#[cfg(feature = "tokio-modbus-rtu")]
#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    use chrono::{DateTime, Utc};
    use env_logger::Builder as LoggerBuilder;
    use futures::prelude::*;
    use std::{cell::RefCell, env, future::Future, io::Error, pin::Pin, rc::Rc, time::Duration};
    use stream_cancel::Tripwire;
    use tokio::time;
    use tokio_modbus::prelude::{client::util::*, *};

    use truebner_smt100::{modbus, *};

    let mut logger_builder = LoggerBuilder::new();
    logger_builder.filter_level(log::LevelFilter::Info);
    if env::var("RUST_LOG").is_ok() {
        let rust_log_var = &env::var("RUST_LOG")?;
        println!("Parsing RUST_LOG={}", rust_log_var);
        logger_builder.parse_filters(rust_log_var);
    }
    logger_builder.init();

    #[derive(Debug, Clone)]
    struct ContextConfig {
        tty_path: String,
    }

    impl NewContext for ContextConfig {
        fn new_context(&self) -> Pin<Box<dyn Future<Output = Result<client::Context, Error>>>> {
            // TODO: Box::pin(modbus::rtu::connect_path(&self.tty_path))
            todo!()
        }
    }

    #[derive(Debug, Clone)]
    struct SlaveConfig {
        slave: Slave,
        cycle_time: Duration,
        timeout: Option<Duration>,
    }

    // TODO: Parse parameters and options from command-line arguments
    let context_config = ContextConfig {
        tty_path: "/dev/ttyUSB0".to_owned(),
    };

    let slave_config = SlaveConfig {
        slave: Slave::min_device(),
        cycle_time: Duration::from_millis(1000),
        timeout: Some(Duration::from_millis(500)),
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
        _shared_context: Rc<RefCell<SharedContext>>,

        config: SlaveConfig,
        proxy: modbus::SlaveProxy,
        measurements: Measurements,
    }

    impl ControlLoop {
        pub fn new(config: SlaveConfig, new_context: Box<dyn NewContext>) -> Self {
            let shared_context = Rc::new(RefCell::new(SharedContext::new(None, new_context)));
            let proxy = modbus::SlaveProxy::new(config.slave, Rc::clone(&shared_context));
            Self {
                _shared_context: shared_context,
                config,
                proxy,
                measurements: Default::default(),
            }
        }

        async fn reconnect(&self) -> Result<(), Error> {
            self.proxy.reconnect().await
        }

        pub async fn measure_temperature(&mut self) -> Result<(), Error> {
            let res = self.proxy.read_temperature(self.config.timeout).await;
            match res {
                Ok(val) => {
                    self.measurements.temperature = Some(Measurement::new(val));
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }

        pub async fn measure_water_content(&mut self) -> Result<(), Error> {
            let res = self.proxy.read_water_content(self.config.timeout).await;
            match res {
                Ok(val) => {
                    self.measurements.water_content = Some(Measurement::new(val));
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }

        pub async fn measure_permittivity(&mut self) -> Result<(), Error> {
            let res = self.proxy.read_permittivity(self.config.timeout).await;
            match res {
                Ok(val) => {
                    self.measurements.permittivity = Some(Measurement::new(val));
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }

        pub async fn recover_after_error(&self, err: &dyn std::fmt::Display) {
            log::warn!("Reconnecting after error: {}", err);
            if let Err(err) = self.reconnect().await {
                log::error!("Failed to reconnect: {}", err);
                // Continue and don't leave/terminate the control loop!
            }
        }

        pub async fn broadcast_slave(&self) -> Result<(), Error> {
            self.proxy.broadcast_slave().await
        }
    }

    log::info!("Connecting: {:?}", context_config);
    let mut ctrl_loop = ControlLoop::new(slave_config, Box::new(context_config));
    //TODO: tokio::spawn(ctrl_loop.reconnect());

    let broadcast_slave = false;
    if broadcast_slave {
        log::info!(
            "Resetting Modbus slave address to {:?}",
            ctrl_loop.proxy.slave()
        );
        //TODO: tokio::spawn(ctrl_loop.broadcast_slave());
    }

    // Asynchronous chain of measurements.

    async fn ctrl_loop_step(ctrl_loop: &mut ControlLoop) -> anyhow::Result<()> {
        ctrl_loop.measure_temperature().await?;
        ctrl_loop.measure_water_content().await?;
        ctrl_loop.measure_permittivity().await?;
        Ok(())
    }

    let (_trigger, tripwire) = Tripwire::new();
    let mut cycle_interval = time::interval(ctrl_loop.config.cycle_time).take_until(tripwire);
    while cycle_interval.next().await.is_some() {
        match ctrl_loop_step(&mut ctrl_loop).await {
            Ok(_) => {
                log::info!("{:?}", ctrl_loop.measurements);
            }
            Err(err) => {
                log::info!("{:?}", ctrl_loop.measurements);
                ctrl_loop.recover_after_error(&err).await;
            }
        }
    }
    Ok(())
}

#[cfg(not(feature = "tokio-modbus-rtu"))]
pub fn main() {
    println!("feature `modbus-rtu` is required to run this example");
    std::process::exit(1);
}
