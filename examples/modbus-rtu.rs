#[cfg(feature = "modbus-rtu")]
pub fn main() {
    use chrono::{DateTime, Utc};
    use env_logger::Builder as LoggerBuilder;
    use futures::{future::Either, Future, Stream};
    use std::{cell::RefCell, env, rc::Rc, time::Duration};
    use stream_cancel::{StreamExt, Tripwire};
    use tokio::timer::Interval;
    use tokio_core::reactor::{Core, Handle};
    use tokio_modbus::prelude::*;

    use truebner_smt100::{modbus, *};

    let mut logger_builder = LoggerBuilder::new();
    if env::var("RUST_LOG").is_ok() {
        let rust_log_var = &env::var("RUST_LOG").unwrap();
        println!("Parsing RUST_LOG={}", rust_log_var);
        logger_builder.parse_filters(rust_log_var);
    }
    logger_builder.init();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // TODO: Parse parameters and options from command-line arguments
    let tty_path = "/dev/ttyUSB0";
    let slave = Slave::min_device();
    let broadcast_slave = false;
    let cycle_time = Duration::from_millis(1000);
    let timeout = Duration::from_millis(500);

    let context_task = modbus::rtu::connect_path(&handle, tty_path);
    let context = Rc::new(RefCell::new(core.run(context_task).unwrap()));

    log::info!("Creating Modbus slave proxy for {:?}", slave);
    let proxy = modbus::SlaveProxy::new(Rc::clone(&context), slave);

    if broadcast_slave {
        log::info!("Resetting Modbus slave address to {:?}", proxy.slave());
        let broadcast_task = proxy.broadcast_slave();
        core.run(broadcast_task).unwrap();
    }

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
        temp: Option<Measurement<Temperature>>,
        vwc: Option<Measurement<VolumetricWaterContent>>,
        perm: Option<Measurement<RelativePermittivity>>,
    }

    struct ControlLoopState {
        handle: Handle,
        tty_path: String,
        proxy: modbus::SlaveProxy,
        measurements: Measurements,
    };

    let (_trigger, tripwire) = Tripwire::new();
    let state = ControlLoopState {
        handle,
        tty_path: tty_path.to_owned(),
        proxy,
        measurements: Default::default(),
    };
    let cycle_interval = Interval::new_interval(cycle_time);
    let control_loop_task = cycle_interval
        .map_err(|err| {
            log::error!("Aborting control loop after timer error: {:?}", err);
        })
        .take_until(tripwire)
        .fold(state, |mut state, _event| {
            // Asynchronous chain of measurements
            state
                .proxy
                .read_temperature(timeout)
                .then(move |res| match res {
                    Ok(val) => {
                        state.measurements.temp = Some(Measurement::new(val));
                        Ok(state)
                    }
                    Err(err) => Err((state, err)),
                })
                .and_then(|mut state| {
                    state
                        .proxy
                        .read_water_content(timeout)
                        .then(move |res| match res {
                            Ok(val) => {
                                state.measurements.vwc = Some(Measurement::new(val));
                                Ok(state)
                            }
                            Err(err) => Err((state, err)),
                        })
                })
                .and_then(|mut state| {
                    state
                        .proxy
                        .read_permittivity(timeout)
                        .then(move |res| match res {
                            Ok(val) => {
                                state.measurements.perm = Some(Measurement::new(val));
                                Ok(state)
                            }
                            Err(err) => Err((state, err)),
                        })
                })
                .then(|res| {
                    match res {
                        Ok(state) => {
                            log::info!("{:?}", state.measurements);
                            Either::A(futures::future::ok(state))
                        }
                        Err((mut state, err)) => {
                            log::warn!(
                                "Reconnecting serial port {} after error: {}",
                                state.tty_path,
                                err
                            );
                            let reconnected_state = modbus::rtu::connect_path(
                                &state.handle,
                                &state.tty_path,
                            )
                            .then(move |res| {
                                match res {
                                    Ok(context) => {
                                        state.proxy.reset_context(Rc::new(RefCell::new(context)));
                                    }
                                    Err(err) => {
                                        log::error!(
                                            "Failed to reconnect serial port {}: {}",
                                            state.tty_path,
                                            err
                                        );
                                        // Continue and don't leave/terminate the control loop!
                                    }
                                };
                                Ok(state)
                            });
                            Either::B(reconnected_state)
                        }
                    }
                })
        });

    core.run(control_loop_task).unwrap();
}

#[cfg(not(feature = "modbus-rtu"))]
pub fn main() {
    println!("feature `modbus-rtu` is required to run this example");
    std::process::exit(1);
}
