use crate::{core::modbus::rtu::*, modbus::*};
use std::{path::Path, time::Duration};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_modbus::client::{rtu::connect_slave, Context as ClientContext};
use tokio_serial::{Serial, SerialPortSettings};

pub const SERIAL_PORT_SETTINGS: SerialPortSettings = SerialPortSettings {
    baud_rate: BAUD_RATE,
    data_bits: DATA_BITS,
    stop_bits: STOP_BITS,
    parity: PARITY,
    flow_control: FLOW_CONTROL,
    // A timeout is currently not supported and ignored by tokio-serial
    // See also: https://github.com/berkowski/tokio-serial/issues/15
    timeout: Duration::from_secs(0),
};

pub async fn connect<T>(transport: T) -> Result<ClientContext>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    connect_slave(transport, BROADCAST_SLAVE).await
}

pub async fn connect_path(path: impl AsRef<Path>) -> Result<ClientContext> {
    log::info!("Connecting to serial port {}", path.as_ref().display());
    match Serial::from_path(path, &SERIAL_PORT_SETTINGS) {
        Ok(serial) => connect(serial).await,
        Err(err) => Err(err),
    }
}
