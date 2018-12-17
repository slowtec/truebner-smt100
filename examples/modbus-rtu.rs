#[cfg(feature = "modbus-rtu")]
pub fn main() {
    use futures::Future;
    use tokio_core::reactor::Core;
    use tokio_modbus::prelude::*;

    use truebner_smt100::modbus;

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // TODO: Parse TTY path and and Modbus slave address
    // from command-line arguments
    let tty_path = "/dev/ttyUSB0";
    let slave = Slave::min_device();

    let task = modbus::rtu::connect_path(&handle, tty_path)
        .and_then(move |ctx| {
            println!("Resetting Modbus slave address to {:?}", slave);
            ctx.init_slave(slave).and_then(move |rsp| Ok((ctx, rsp)))
        })
        .and_then(move |(ctx, slave)| {
            println!("Reset Modbus slave address to {:?}", slave);
            let proxy = modbus::SlaveProxy::from_context(ctx, slave);
            println!("Reading (thermodynamic) temperature...");
            proxy.read_temperature().and_then(move |rsp| Ok((proxy, rsp)))
        })
        .and_then(|(proxy, rsp)| {
            println!("Current (thermodynamic) temperature is {}", rsp);
            println!("Reading (volumetric) water content...");
            proxy.read_water_content().and_then(move |rsp| Ok((proxy, rsp)))
        })
        .and_then(|(proxy, rsp)| {
            println!("Current (volumetric) water content is {}", rsp);
            println!("Reading (relative) permittivity");
            proxy.read_permittivity().and_then(move |rsp| Ok((proxy, rsp)))
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
