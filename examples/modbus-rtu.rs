#[cfg(feature = "modbus-rtu")]
pub fn main() {
    use futures::Future;
    use tokio_core::reactor::Core;
    use tokio_modbus::prelude::*;

    use truebner_smt100::modbus::Device as ModbusDevice;
    use truebner_smt100::*;

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let tty_path = "/dev/ttyUSB0";
    let device_id = DeviceId::min_slave();

    // On Unix you might disable the `exclusive` flag:
    // port.set_exclusive(false)
    //     .expect("Unable to set serial port exlusive");

    let task = modbus::rtu::connect_path(&handle, tty_path)
        .and_then(move |ctx| {
            println!("Changing Modbus slave address to {:?}", device_id);
            ctx.change_device_id(device_id)
                .and_then(move |rsp| Ok((ctx, rsp)))
        })
        .and_then(move |(mut ctx, device_id)| {
            println!("Changed Modbus slave address to {:?}", device_id);
            ctx.switch_device(device_id);
            println!("Reading temperature");
            ctx.read_temperature().and_then(move |rsp| Ok((ctx, rsp)))
        })
        .and_then(|(ctx, rsp)| {
            println!("Current temperature is {:?}", rsp);
            println!("Reading soil moisture");
            ctx.read_soil_moisture().and_then(move |rsp| Ok((ctx, rsp)))
        })
        .and_then(|(_, rsp)| {
            println!("Current soil moisture is {:?}", rsp);
            Ok(())
        });

    core.run(task).unwrap();
}

#[cfg(not(feature = "modbus-rtu"))]
pub fn main() {
    println!("feature `modbus-rtu` is required to run this example");
    std::process::exit(1);
}
