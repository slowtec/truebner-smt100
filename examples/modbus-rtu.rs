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
        .and_then(move |(mut ctx, slave)| {
            println!("Reset Modbus slave address to {:?}", slave);
            ctx.set_slave(slave);
            println!("Reading temperature");
            ctx.read_temperature().and_then(move |rsp| Ok((ctx, rsp)))
        })
        .and_then(|(ctx, rsp)| {
            println!("Current temperature is {} °C", rsp.celsius);
            println!("Reading water content");
            ctx.read_water_content().and_then(move |rsp| Ok((ctx, rsp)))
        })
        .and_then(|(ctx, rsp)| {
            println!("Current water content is {} %", rsp.percent);
            println!("Reading (relative) permittivity");
            ctx.read_permittivity().and_then(move |rsp| Ok((ctx, rsp)))
        })
        .and_then(|(_, rsp)| {
            println!("Current (relative) permittivity is {} K", rsp.ratio);
            Ok(())
        });

    core.run(task).unwrap();
}

#[cfg(not(feature = "modbus-rtu"))]
pub fn main() {
    println!("feature `modbus-rtu` is required to run this example");
    std::process::exit(1);
}
