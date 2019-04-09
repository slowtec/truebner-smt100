#[cfg(feature = "modbus-rtu")]
pub fn main() {
    use chrono::{DateTime, Utc};
    use env_logger::Builder as LoggerBuilder;
    use futures::{future::Either, Future, Stream};
    use std::{cell::RefCell, env, io::Error, rc::Rc, time::Duration};
    use stream_cancel::{StreamExt, Tripwire};
    use tokio::timer::Interval;
    use tokio_core::reactor::{Core, Handle};
    use tokio_modbus::prelude::{*, client::util::*};

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
    struct ContextConfig {
        handle: Handle,
        tty_path: String,
    };

    impl NewContext for ContextConfig {
        fn new_context(&self) -> Box<dyn Future<Item = client::Context, Error = Error>> {
            Box::new(modbus::rtu::connect_path(&self.handle, &self.tty_path))
        }
    }

    #[derive(Debug, Clone)]
    struct SlaveConfig {
        slave: Slave,
        cycle_time: Duration,
        timeout: Duration,
    };

    // TODO: Parse parameters and options from command-line arguments
    let context_config = ContextConfig {
        handle: core.handle(),
        tty_path: "/dev/ttyUSB0".to_owned(),
    };
    let slave_config = SlaveConfig {
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

    // Only a single slave sensor is used for demonstration purposes here.
    // A typical application will use multiple slaves that all share
    // the same Modbus environment, RTU client context and bus wiring,
    // i.e. multiple sensors and actuators are all connected to a single
    // serial port.
    struct ControlLoop {
        // Only shared with the single proxy and otherwise unused within the
        // control loop. Just to demonstrate how to share the Modbus context
        // and how to recover from communication errors by reconnecting.
        shared_context: Rc<RefCell<SharedContext>>,

        config: SlaveConfig,
        proxy: modbus::SlaveProxy,
        measurements: Measurements,
    };

    impl ControlLoop {
        pub fn new(config: SlaveConfig, new_context: Box<dyn NewContext>) -> Self {
            let shared_context = Rc::new(RefCell::new(SharedContext::new(None, new_context)));
            let proxy = modbus::SlaveProxy::new(config.slave, Rc::clone(&shared_context));
            Self {
                shared_context,
                config,
                proxy,
                measurements: Default::default(),
            }
        }

        fn reconnect(self) -> impl Future<Item = Self, Error = (Error, Self)> {
            let shared_context = self.shared_context;
            let config = self.config;
            let measurements = self.measurements;
            self.proxy.reconnect().then(move |res| {
                match res {
                    Ok(proxy) => {
                        let this = Self {
                            shared_context,
                            config,
                            proxy,
                            measurements,
                        };
                        Ok(this)
                    }
                    Err((err, proxy)) => {
                        let this = Self {
                            shared_context,
                            config,
                            proxy,
                            measurements,
                        };
                        Err((err, this))
                    }
                }
            })
        }

        pub fn measure_temperature(mut self) -> impl Future<Item = Self, Error = (Error, Self)> {
            self
                .proxy
                .read_temperature(self.config.timeout)
                .then(move |res| match res {
                    Ok(val) => {
                        self.measurements.temperature = Some(Measurement::new(val));
                        Ok(self)
                    }
                    Err(err) => Err((err, self)),
                })
        }

        pub fn measure_water_content(mut self) -> impl Future<Item = Self, Error = (Error, Self)> {
            self
                .proxy
                .read_water_content(self.config.timeout)
                .then(move |res| match res {
                    Ok(val) => {
                        self.measurements.water_content = Some(Measurement::new(val));
                        Ok(self)
                    }
                    Err(err) => Err((err, self)),
                })
        }

        pub fn measure_permittivity(mut self) -> impl Future<Item = Self, Error = (Error, Self)> {
            self
                .proxy
                .read_permittivity(self.config.timeout)
                .then(move |res| match res {
                    Ok(val) => {
                        self.measurements.permittivity = Some(Measurement::new(val));
                        Ok(self)
                    }
                    Err(err) => Err((err, self)),
                })
        }

        pub fn recover_after_error(self, err: &Error) -> impl Future<Item = Self, Error = ()> {
            log::warn!(
                "Reconnecting after error: {}",
                err
            );
            self.reconnect().then(move |res| {
                let this = match res {
                    Ok(this) => this,
                    Err((err, this)) => {
                        log::error!(
                            "Failed to reconnect: {}",
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
            self.proxy.broadcast_slave()
        }
    }

    log::info!("Connecting: {:?}", context_config);
    let ctrl_loop = ControlLoop::new(slave_config, Box::new(context_config));
    let ctrl_loop = core.run(ctrl_loop.reconnect()).map_err(|(err, _)| err).unwrap();

    let broadcast_slave = false;
    if broadcast_slave {
        log::info!(
            "Resetting Modbus slave address to {:?}",
            ctrl_loop.proxy.slave()
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
                        log::info!("{:?}", ctrl_loop.measurements);
                        Either::A(futures::future::ok(ctrl_loop))
                    }
                    Err((err, ctrl_loop)) => {
                        log::info!("{:?}", ctrl_loop.measurements);
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
