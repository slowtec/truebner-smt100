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
                .and_then(|context| Ok(super::Context { context })),
        ),
        Err(err) => Box::new(future::err(err)),
    }
}
