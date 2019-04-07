#[cfg(feature = "modbus-rtu")]
pub fn main() {
    use chrono::{DateTime, Utc};
    use env_logger::Builder as LoggerBuilder;
    use futures::{future::Either, Future, Stream};
    use std::{cell::RefCell, env, io::Error, rc::Rc, time::Duration};
    use stream_cancel::{StreamExt, Tripwire};
    use tokio::timer::Interval;
    use tokio_core::reactor::{Core, Handle};
    use tokio_modbus::prelude::{client::Context as ModbusContext, *};

    use truebner_smt100::{modbus, *};

    let mut logger_builder = LoggerBuilder::new();
    if env::var("RUST_LOG").is_ok() {
        let rust_log_var = &env::var("RUST_LOG").unwrap();
        println!("Parsing RUST_LOG={}", rust_log_var);
        logger_builder.parse_filters(rust_log_var);
    }
    logger_builder.init();

    let mut core = Core::new().unwrap();

    #[derive(Debug, Clone)]
    struct Config {
        handle: Handle,
        tty_path: String,
        slave: Slave,
        cycle_time: Duration,
        timeout: Duration,
    };

    impl Config {
        fn connect(&self) -> impl Future<Item = ModbusContext, Error = Error> {
            modbus::rtu::connect_path(&self.handle, &self.tty_path)
        }
    }

    // TODO: Parse parameters and options from command-line arguments
    let config = Config {
        handle: core.handle(),
        tty_path: "/dev/ttyUSB0".to_owned(),
        slave: Slave::min_device(),
        cycle_time: Duration::from_millis(1000),
        timeout: Duration::from_millis(500),
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Measurement<T> {
        ts: DateTime<Utc>,
        val: T,
    }

    impl<T> Measurement<T> {
        pub fn new(val: T) -> Self {
            Self {
                ts: Utc::now(),
                val,
            }
        }
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq)]
    struct Measurements {
        temperature: Option<Measurement<Temperature>>,
        water_content: Option<Measurement<VolumetricWaterContent>>,
        permittivity: Option<Measurement<RelativePermittivity>>,
    }

    struct State {
        proxy: modbus::SlaveProxy,
        measurements: Measurements,
    };

    impl State {
        pub fn new(context: &Rc<RefCell<ModbusContext>>, slave: Slave) -> Self {
            Self {
                proxy: modbus::SlaveProxy::new(Rc::clone(context), slave),
                measurements: Default::default(),
            }
        }

        pub fn reset_context(&mut self, context: &Rc<RefCell<ModbusContext>>) {
            self.proxy.reset_context(Rc::clone(context));
        }
    }

    struct ControlLoop {
        config: Config,
        state: State,
    };

    impl ControlLoop {
        pub fn connect(config: Config) -> impl Future<Item = Self, Error = Error> {
            config.connect().map(move |context| {
                let state = State::new(&Rc::new(RefCell::new(context)), config.slave);
                Self { config, state }
            })
        }

        fn reconnect(mut self) -> impl Future<Item = Self, Error = (Self, Error)> {
            self.config.connect().then(move |res| match res {
                Ok(context) => {
                    self.state.reset_context(&Rc::new(RefCell::new(context)));
                    Ok(self)
                }
                Err(err) => Err((self, err)),
            })
        }

        pub fn measure_temperature(mut self) -> impl Future<Item = Self, Error = (Self, Error)> {
            self.state
                .proxy
                .read_temperature(self.config.timeout)
                .then(move |res| match res {
                    Ok(val) => {
                        self.state.measurements.temperature = Some(Measurement::new(val));
                        Ok(self)
                    }
                    Err(err) => Err((self, err)),
                })
        }

        pub fn measure_water_content(mut self) -> impl Future<Item = Self, Error = (Self, Error)> {
            self.state
                .proxy
                .read_water_content(self.config.timeout)
                .then(move |res| match res {
                    Ok(val) => {
                        self.state.measurements.water_content = Some(Measurement::new(val));
                        Ok(self)
                    }
                    Err(err) => Err((self, err)),
                })
        }

        pub fn measure_permittivity(mut self) -> impl Future<Item = Self, Error = (Self, Error)> {
            self.state
                .proxy
                .read_permittivity(self.config.timeout)
                .then(move |res| match res {
                    Ok(val) => {
                        self.state.measurements.permittivity = Some(Measurement::new(val));
                        Ok(self)
                    }
                    Err(err) => Err((self, err)),
                })
        }

        pub fn recover_after_error(self, err: &Error) -> impl Future<Item = Self, Error = ()> {
            log::warn!(
                "Reconnecting serial port {} after error: {}",
                self.config.tty_path,
                err
            );
            self.reconnect().then(move |res| {
                let this = match res {
                    Ok(this) => this,
                    Err((this, err)) => {
                        log::error!(
                            "Failed to reconnect serial port {}: {}",
                            this.config.tty_path,
                            err
                        );
                        // Continue and don't leave/terminate the control loop!
                        this
                    }
                };
                Ok(this)
            })
        }

        pub fn broadcast_slave(&self) -> impl Future<Item = (), Error = Error> {
            self.state.proxy.broadcast_slave()
        }
    }

    log::info!("Connecting: {:?}", config);
    let ctrl_loop = core.run(ControlLoop::connect(config)).unwrap();

    let broadcast_slave = false;
    if broadcast_slave {
        log::info!(
            "Resetting Modbus slave address to {:?}",
            ctrl_loop.state.proxy.slave()
        );
        core.run(ctrl_loop.broadcast_slave()).unwrap();
    }

    let (_trigger, tripwire) = Tripwire::new();
    let cycle_interval = Interval::new_interval(ctrl_loop.config.cycle_time);
    let ctrl_loop_task = cycle_interval
        .map_err(|err| {
            log::error!("Aborting control loop after timer error: {:?}", err);
        })
        .take_until(tripwire)
        .fold(ctrl_loop, |ctrl_loop, _event| {
            // Asynchronous chain of measurements
            futures::future::ok(ctrl_loop)
                .and_then(ControlLoop::measure_temperature)
                .and_then(ControlLoop::measure_water_content)
                .and_then(ControlLoop::measure_permittivity)
                .then(|res| match res {
                    Ok(ctrl_loop) => {
                        log::info!("{:?}", ctrl_loop.state.measurements);
                        Either::A(futures::future::ok(ctrl_loop))
                    }
                    Err((ctrl_loop, err)) => {
                        log::info!("{:?}", ctrl_loop.state.measurements);
                        Either::B(ctrl_loop.recover_after_error(&err))
                    }
                })
        });

    core.run(ctrl_loop_task).unwrap();
}

#[cfg(not(feature = "modbus-rtu"))]
pub fn main() {
    println!("feature `modbus-rtu` is required to run this example");
    std::process::exit(1);
}
