use super::*;

#[cfg(feature = "rtu")]
pub mod rtu;

use crate::core::modbus::*;

use futures::Future;
use std::{
    cell::RefCell,
    io::{Error, ErrorKind, Result},
    rc::Rc,
    time::Duration,
};
use tokio::prelude::*;

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
pub fn broadcast_slave(
    context: &mut client::Context,
    slave: Slave,
) -> impl Future<Item = (), Error = Error> {
    context.set_slave(BROADCAST_SLAVE);
    let slave_id: SlaveId = slave.into();
    context.write_single_register(BROADCAST_REG_ADDR, u16::from(slave_id))
}

pub fn read_temperature(
    context: &mut client::Context,
) -> impl Future<Item = Temperature, Error = Error> {
    context
        .read_holding_registers(TEMPERATURE_REG_START, TEMPERATURE_REG_COUNT)
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

pub fn read_temperature_with_timeout(
    context: &mut client::Context,
    timeout: Duration,
) -> impl Future<Item = Temperature, Error = Error> {
    read_temperature(context).timeout(timeout)
        .map_err(move |err| {
            err.into_inner().unwrap_or_else(|| {
                Error::new(
                    ErrorKind::TimedOut,
                    String::from("reading temperature timed out"),
                )
            })
        })
}

pub fn read_water_content(
    context: &mut client::Context,
) -> impl Future<Item = VolumetricWaterContent, Error = Error> {
    context
        .read_holding_registers(WATER_CONTENT_REG_START, WATER_CONTENT_REG_COUNT)
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

pub fn read_water_content_with_timeout(
    context: &mut client::Context,
    timeout: Duration,
) -> impl Future<Item = VolumetricWaterContent, Error = Error> {
    read_water_content(context)
        .timeout(timeout)
        .map_err(move |err| {
            err.into_inner().unwrap_or_else(|| {
                Error::new(
                    ErrorKind::TimedOut,
                    String::from("reading water content timed out"),
                )
            })
        })
}

pub fn read_permittivity(
    context: &mut client::Context,
) -> impl Future<Item = RelativePermittivity, Error = Error> {
    context
        .read_holding_registers(PERMITTIVITY_REG_START, PERMITTIVITY_REG_COUNT)
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

pub fn read_permittivity_with_timeout(
    context: &mut client::Context,
    timeout: Duration,
) -> impl Future<Item = RelativePermittivity, Error = Error> {
    read_permittivity(context)
        .timeout(timeout)
        .map_err(move |err| {
            err.into_inner().unwrap_or_else(|| {
                Error::new(
                    ErrorKind::TimedOut,
                    String::from("reading permittivity timed out"),
                )
            })
        })
}

pub fn read_raw_counts(
    context: &mut client::Context,
) -> impl Future<Item = RawCounts, Error = Error> {
    context
        .read_holding_registers(RAW_COUNTS_REG_START, RAW_COUNTS_REG_COUNT)
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

pub fn read_raw_counts_with_timeout(
    context: &mut client::Context,
    timeout: Duration,
) -> impl Future<Item = RawCounts, Error = Error> {
    read_raw_counts(context)
        .timeout(timeout)
        .map_err(move |err| {
            err.into_inner().unwrap_or_else(|| {
                Error::new(
                    ErrorKind::TimedOut,
                    String::from("reading raw counts timed out"),
                )
            })
        })
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
    pub fn reconnect(&self) -> impl Future<Item = (), Error = Error> {
        reconnect_shared_context(&self.shared_context)
    }

    fn shared_context(&self) -> Result<Rc<RefCell<client::Context>>> {
        if let Some(context) = self.shared_context.borrow().share_context() {
            Ok(context)
        } else {
            Err(Error::new(ErrorKind::NotConnected, "No shared context"))
        }
    }

    /// Switch the Modbus slave address of all connected devices.
    pub fn broadcast_slave(&self) -> impl Future<Item = (), Error = Error> {
        match self.shared_context() {
            Ok(shared_context) => future::Either::A(self::broadcast_slave(
                &mut shared_context.borrow_mut(),
                self.slave,
            )),
            Err(err) => future::Either::B(future::err(err)),
        }
    }

    pub fn read_temperature(
        &self,
        timeout: Option<Duration>,
    ) -> impl Future<Item = Temperature, Error = Error> {
        match self.shared_context() {
            Ok(shared_context) => {
                let mut context = shared_context.borrow_mut();
                context.set_slave(self.slave);
                future::Either::A(if let Some(timeout) = timeout {
                    future::Either::A(read_temperature_with_timeout(&mut context, timeout))
                } else {
                    future::Either::B(read_temperature(&mut context))
                })
            }
            Err(err) => future::Either::B(future::err(err)),
        }
    }

    pub fn read_water_content(
        &self,
        timeout: Option<Duration>,
    ) -> impl Future<Item = VolumetricWaterContent, Error = Error> {
        match self.shared_context() {
            Ok(shared_context) => {
                let mut context = shared_context.borrow_mut();
                context.set_slave(self.slave);
                future::Either::A(if let Some(timeout) = timeout {
                    future::Either::A(read_water_content_with_timeout(&mut context, timeout))
                } else {
                    future::Either::B(read_water_content(&mut context))
                })
            }
            Err(err) => future::Either::B(future::err(err)),
        }
    }

    pub fn read_permittivity(
        &self,
        timeout: Option<Duration>,
    ) -> impl Future<Item = RelativePermittivity, Error = Error> {
        match self.shared_context() {
            Ok(shared_context) => {
                let mut context = shared_context.borrow_mut();
                context.set_slave(self.slave);
                future::Either::A(if let Some(timeout) = timeout {
                    future::Either::A(read_permittivity_with_timeout(&mut context, timeout))
                } else {
                    future::Either::B(read_permittivity(&mut context))
                })
            }
            Err(err) => future::Either::B(future::err(err)),
        }
    }

    pub fn read_raw_counts(&self, timeout: Option<Duration>) -> impl Future<Item = RawCounts, Error = Error> {
        match self.shared_context() {
            Ok(shared_context) => {
                let mut context = shared_context.borrow_mut();
                context.set_slave(self.slave);
                future::Either::A(if let Some(timeout) = timeout {
                    future::Either::A(read_raw_counts_with_timeout(&mut context, timeout))
                } else {
                    future::Either::B(read_raw_counts(&mut context))
                })
            }
            Err(err) => future::Either::B(future::err(err)),
        }
    }
}

impl Capabilities for SlaveProxy {
    fn read_temperature(
        &self,
        timeout: Option<Duration>,
    ) -> Box<dyn Future<Item = Temperature, Error = Error>> {
        Box::new(self.read_temperature(timeout))
    }

    fn read_water_content(
        &self,
        timeout: Option<Duration>,
    ) -> Box<dyn Future<Item = VolumetricWaterContent, Error = Error>> {
        Box::new(self.read_water_content(timeout))
    }

    fn read_permittivity(
        &self,
        timeout: Option<Duration>,
    ) -> Box<dyn Future<Item = RelativePermittivity, Error = Error>> {
        Box::new(self.read_permittivity(timeout))
    }

    fn read_raw_counts(&self, timeout: Option<Duration>) -> Box<dyn Future<Item = RawCounts, Error = Error>> {
        Box::new(self.read_raw_counts(timeout))
    }
}
