#[cfg(feature = "rtu")]
pub mod rtu;

use crate::{core::modbus::*, *};
use async_trait::async_trait;
use std::{
    cell::RefCell,
    io::{ErrorKind, Result},
    rc::Rc,
    time::Duration,
};
use tokio::time;
use tokio_modbus::{
    client::util::{reconnect_shared_context, SharedContext},
    prelude::*,
};

impl From<DecodeError> for Error {
    fn from(from: DecodeError) -> Self {
        use DecodeError::*;
        match from {
            InsufficientInput | InvalidInput => Self::new(ErrorKind::InvalidInput, from),
            InvalidData => Self::new(ErrorKind::InvalidData, from),
        }
    }
}

/// The fixed broadcast address of all sensors that cannot be altered.
///
/// Warning: This address should only be used for configuration purposes,
/// i.e. for initially setting the Modbus slave address of each connected
/// device. All other requests to this address are answered with the
/// slave address 0 (= broadcast) and might be rejected by _tokio-modbus_!
pub const BROADCAST_SLAVE: Slave = Slave(BROADCAST_SLAVE_ADDR);

/// Switch the Modbus slave address of all connected devices.
pub async fn broadcast_slave(context: &mut client::Context, slave: Slave) -> Result<()> {
    context.set_slave(BROADCAST_SLAVE);
    let slave_id: SlaveId = slave.into();
    context
        .write_single_register(BROADCAST_REG_ADDR, u16::from(slave_id))
        .await
}

pub async fn read_temperature(context: &mut client::Context) -> Result<Temperature> {
    context
        .read_holding_registers(TEMPERATURE_REG_START, TEMPERATURE_REG_COUNT)
        .await
        .and_then(|rsp| {
            if let [raw] = rsp[..] {
                decode_temperature_from_u16(raw).map_err(Into::into)
            } else {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("unexpected temperature data: {:?}", rsp),
                ))
            }
        })
}

pub async fn read_temperature_with_timeout(
    context: &mut client::Context,
    timeout: Duration,
) -> Result<Temperature> {
    time::timeout(timeout, read_temperature(context))
        .await
        .map_err(move |_| {
            Error::new(
                ErrorKind::TimedOut,
                String::from("reading temperature timed out"),
            )
        })?
}

pub async fn read_water_content(context: &mut client::Context) -> Result<VolumetricWaterContent> {
    context
        .read_holding_registers(WATER_CONTENT_REG_START, WATER_CONTENT_REG_COUNT)
        .await
        .and_then(|rsp| {
            if let [reg] = rsp[..] {
                decode_water_content_from_u16(reg).map_err(Into::into)
            } else {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("unexpected water content data: {:?}", rsp),
                ))
            }
        })
}

pub async fn read_water_content_with_timeout(
    context: &mut client::Context,
    timeout: Duration,
) -> Result<VolumetricWaterContent> {
    time::timeout(timeout, read_water_content(context))
        .await
        .map_err(move |_| {
            Error::new(
                ErrorKind::TimedOut,
                String::from("reading water content timed out"),
            )
        })?
}

pub async fn read_permittivity(context: &mut client::Context) -> Result<RelativePermittivity> {
    context
        .read_holding_registers(PERMITTIVITY_REG_START, PERMITTIVITY_REG_COUNT)
        .await
        .and_then(|rsp| {
            if let [reg] = rsp[..] {
                decode_permittivity_from_u16(reg).map_err(Into::into)
            } else {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("unexpected permittivity data: {:?}", rsp),
                ))
            }
        })
}

pub async fn read_permittivity_with_timeout(
    context: &mut client::Context,
    timeout: Duration,
) -> Result<RelativePermittivity> {
    time::timeout(timeout, read_permittivity(context))
        .await
        .map_err(move |_| {
            Error::new(
                ErrorKind::TimedOut,
                String::from("reading permittivity timed out"),
            )
        })?
}

pub async fn read_raw_counts(context: &mut client::Context) -> Result<RawCounts> {
    context
        .read_holding_registers(RAW_COUNTS_REG_START, RAW_COUNTS_REG_COUNT)
        .await
        .and_then(|rsp| {
            if let [reg] = rsp[..] {
                decode_raw_counts_from_u16(reg).map_err(Into::into)
            } else {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("unexpected raw counts data: {:?}", rsp),
                ))
            }
        })
}

pub async fn read_raw_counts_with_timeout(
    context: &mut client::Context,
    timeout: Duration,
) -> Result<RawCounts> {
    time::timeout(timeout, read_raw_counts(context))
        .await
        .map_err(move |_| {
            Error::new(
                ErrorKind::TimedOut,
                String::from("reading raw counts timed out"),
            )
        })?
}

pub struct SlaveProxy {
    slave: Slave,
    shared_context: Rc<RefCell<SharedContext>>,
}

impl SlaveProxy {
    pub fn new(slave: Slave, shared_context: Rc<RefCell<SharedContext>>) -> Self {
        Self {
            slave,
            shared_context,
        }
    }

    pub fn slave(&self) -> Slave {
        self.slave
    }

    /// Reconnect a new, shared Modbus context to recover from communication errors.
    pub async fn reconnect(&self) -> Result<()> {
        reconnect_shared_context(&self.shared_context).await
    }

    fn shared_context(&self) -> Result<Rc<RefCell<client::Context>>> {
        if let Some(context) = self.shared_context.borrow().share_context() {
            Ok(context)
        } else {
            Err(Error::new(ErrorKind::NotConnected, "No shared context"))
        }
    }

    /// Switch the Modbus slave address of all connected devices.
    pub async fn broadcast_slave(&self) -> Result<()> {
        match self.shared_context() {
            Ok(shared_context) => {
                self::broadcast_slave(&mut shared_context.borrow_mut(), self.slave).await
            }
            Err(err) => Err(err),
        }
    }

    pub async fn read_temperature(&self, timeout: Option<Duration>) -> Result<Temperature> {
        match self.shared_context() {
            Ok(shared_context) => {
                let mut context = shared_context.borrow_mut();
                context.set_slave(self.slave);
                if let Some(timeout) = timeout {
                    read_temperature_with_timeout(&mut context, timeout).await
                } else {
                    read_temperature(&mut context).await
                }
            }
            Err(err) => Err(err),
        }
    }

    pub async fn read_water_content(
        &self,
        timeout: Option<Duration>,
    ) -> Result<VolumetricWaterContent> {
        match self.shared_context() {
            Ok(shared_context) => {
                let mut context = shared_context.borrow_mut();
                context.set_slave(self.slave);
                if let Some(timeout) = timeout {
                    read_water_content_with_timeout(&mut context, timeout).await
                } else {
                    read_water_content(&mut context).await
                }
            }
            Err(err) => Err(err),
        }
    }

    pub async fn read_permittivity(
        &self,
        timeout: Option<Duration>,
    ) -> Result<RelativePermittivity> {
        match self.shared_context() {
            Ok(shared_context) => {
                let mut context = shared_context.borrow_mut();
                context.set_slave(self.slave);
                if let Some(timeout) = timeout {
                    read_permittivity_with_timeout(&mut context, timeout).await
                } else {
                    read_permittivity(&mut context).await
                }
            }
            Err(err) => Err(err),
        }
    }

    pub async fn read_raw_counts(&self, timeout: Option<Duration>) -> Result<RawCounts> {
        match self.shared_context() {
            Ok(shared_context) => {
                let mut context = shared_context.borrow_mut();
                context.set_slave(self.slave);
                if let Some(timeout) = timeout {
                    read_raw_counts_with_timeout(&mut context, timeout).await
                } else {
                    read_raw_counts(&mut context).await
                }
            }
            Err(err) => Err(err),
        }
    }
}

#[async_trait(?Send)]
impl crate::Capabilities for SlaveProxy {
    async fn read_temperature(&self, timeout: Option<Duration>) -> Result<Temperature> {
        self.read_temperature(timeout).await
    }

    async fn read_water_content(
        &self,
        timeout: Option<Duration>,
    ) -> Result<VolumetricWaterContent> {
        self.read_water_content(timeout).await
    }

    async fn read_permittivity(&self, timeout: Option<Duration>) -> Result<RelativePermittivity> {
        self.read_permittivity(timeout).await
    }

    async fn read_raw_counts(&self, timeout: Option<Duration>) -> Result<RawCounts> {
        self.read_raw_counts(timeout).await
    }
}
