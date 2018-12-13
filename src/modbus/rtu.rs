use super::*;

use futures::{future, Future};
use std::{io::Error, path::Path, time::Duration};
use tokio_core::reactor::Handle;
use tokio_modbus::client::rtu::connect_slave;
use tokio_serial::{DataBits, FlowControl, Parity, Serial, SerialPortSettings, StopBits};

/// The fixed broadcast address of all sensors that cannot be altered.
///
/// Warning: This address should only be used for configuration purposes,
/// i.e. for initially setting the Modbus slave address of each connected
/// device. All other requests to this address are answered with the
/// slave address 0 (= broadcast) and might be rejected by _tokio-modbus_!
pub const BROADCAST_SLAVE: Slave = Slave(0xFD);

pub const SERIAL_PORT_SETTINGS: SerialPortSettings = SerialPortSettings {
    baud_rate: 9600,
    data_bits: DataBits::Eight,
    parity: Parity::Even,
    stop_bits: StopBits::One,
    flow_control: FlowControl::None,
    // A timeout is currently not supported and ignored by tokio-serial
    // See also: https://github.com/berkowski/tokio-serial/issues/15
    timeout: Duration::from_secs(0),
};

pub fn connect_path<P>(
    handle: &Handle,
    tty_path: P,
) -> Box<Future<Item = super::Context, Error = Error>>
where
    P: AsRef<Path>,
{
    match Serial::from_path_with_handle(tty_path, &SERIAL_PORT_SETTINGS, &handle.new_tokio_handle())
    {
        Ok(port) => Box::new(
            connect_slave(handle, port, BROADCAST_SLAVE)
                .and_then(|ctx| Ok(super::Context::new(ctx))),
        ),
        Err(err) => Box::new(future::err(err)),
    }
}

pub trait SlaveDevice {
    fn init_slave_device(&self, slave: Slave) -> Box<Future<Item = Slave, Error = Error>>;
}

impl SlaveDevice for super::Context {
    fn init_slave_device(&self, slave: Slave) -> Box<Future<Item = Slave, Error = Error>> {
        let req_adr: u16 = 0x0004;
        let slave_id: SlaveId = slave.into();
        let req_reg: u16 = slave_id.into();
        let req = Request::WriteSingleRegister(req_adr, req_reg);
        Box::new(self.ctx.call(req).and_then(move |rsp| {
            if let Response::WriteSingleRegister(rsp_adr, rsp_reg) = rsp {
                if (req_adr, req_reg) == (rsp_adr, rsp_reg) {
                    return Ok(slave);
                }
            }
            Err(Error::new(ErrorKind::InvalidData, "Invalid response"))
        }))
    }
}
