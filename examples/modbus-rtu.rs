#[cfg(feature = "modbus-rtu")]
pub fn main() {
    use futures::Future;
    use std::time::Duration;
    use tokio_core::reactor::Core;
    use tokio_modbus::prelude::*;

    use truebner_smt100::modbus;

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let timeout = Duration::from_millis(500);

    // TODO: Parse TTY path and and Modbus slave address
    // from command-line arguments
    let tty_path = "/dev/ttyUSB0";
    let slave = Slave::min_device();

    let task = modbus::rtu::connect_path(&handle, tty_path)
        .and_then(move |context| {
            println!("Resetting Modbus slave address to {:?}", slave);
            context
                .init_slave(slave)
                .and_then(move |rsp| Ok((context, rsp)))
        })
        .and_then(move |(context, slave)| {
            println!("Reset Modbus slave address to {:?}", slave);
            let proxy = modbus::SlaveProxy::from_context(context, slave);
            println!("Reading (thermodynamic) temperature...");
            proxy
                .read_temperature(timeout)
                .and_then(move |rsp| Ok((proxy, rsp)))
        })
        .and_then(|(proxy, rsp)| {
            println!("Current (thermodynamic) temperature is {}", rsp);
            println!("Reading (volumetric) water content...");
            proxy
                .read_water_content(timeout)
                .and_then(move |rsp| Ok((proxy, rsp)))
        })
        .and_then(|(proxy, rsp)| {
            println!("Current (volumetric) water content is {}", rsp);
            println!("Reading (relative) permittivity");
            proxy
                .read_permittivity(timeout)
                .and_then(move |rsp| Ok((proxy, rsp)))
        })
        .and_then(|(_, rsp)| {
            println!("Current (relative) permittivity is {}", rsp);
            Ok(())
        });

    core.run(task).unwrap();
}

#[cfg(not(feature = "modbus-rtu"))]
pub fn main() {
    println!("feature `modbus-rtu` is required to run this example");
    std::process::exit(1);
}
