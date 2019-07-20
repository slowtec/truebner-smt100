use super::*;

#[cfg(feature = "rtu")]
pub mod rtu;

use crate::core::modbus::*;

use futures::Future;
use std::{
    cell::RefCell,
    io::{Error, ErrorKind, Result},
    rc::Rc,
};
use tokio::prelude::*;

use tokio_modbus::{
    client::util::{reconnect_shared_context, SharedContext},
    prelude::*,
};

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
    timeout: Duration,
) -> impl Future<Item = Temperature, Error = Error> {
    context
        .read_holding_registers(TEMPERATURE_REG_START, TEMPERATURE_REG_COUNT)
        .timeout(timeout)
        .map_err(move |err| {
            err.into_inner().unwrap_or_else(|| {
                Error::new(
                    ErrorKind::TimedOut,
                    String::from("reading temperature timed out"),
                )
            })
        })
        .and_then(|rsp| {
            if let [raw] = rsp[..] {
                Ok(TemperatureRaw(raw).into())
            } else {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("unexpected temperature data: {:?}", rsp),
                ))
            }
        })
}

pub fn read_water_content(
    context: &mut client::Context,
    timeout: Duration,
) -> impl Future<Item = VolumetricWaterContent, Error = Error> {
    context
        .read_holding_registers(WATER_CONTENT_REG_START, WATER_CONTENT_REG_COUNT)
        .timeout(timeout)
        .map_err(move |err| {
            err.into_inner().unwrap_or_else(|| {
                Error::new(
                    ErrorKind::TimedOut,
                    String::from("reading water content timed out"),
                )
            })
        })
        .and_then(|rsp| {
            if let [raw] = rsp[..] {
                let val = VolumetricWaterContent::from(VolumetricWaterContentRaw(raw));
                if val.is_valid() {
                    Ok(val)
                } else {
                    Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("invalid water content value: {}", val),
                    ))
                }
            } else {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("unexpected water content data: {:?}", rsp),
                ))
            }
        })
}

pub fn read_permittivity(
    context: &mut client::Context,
    timeout: Duration,
) -> impl Future<Item = RelativePermittivity, Error = Error> {
    context
        .read_holding_registers(PERMITTIVITY_REG_START, PERMITTIVITY_REG_COUNT)
        .timeout(timeout)
        .map_err(move |err| {
            err.into_inner().unwrap_or_else(|| {
                Error::new(
                    ErrorKind::TimedOut,
                    String::from("reading permittivity timed out"),
                )
            })
        })
        .and_then(|rsp| {
            if let [raw] = rsp[..] {
                let val = RelativePermittivity::from(RelativePermittivityRaw(raw));
                if val.is_valid() {
                    Ok(val)
                } else {
                    Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("invalid permittivity value: {}", val),
                    ))
                }
            } else {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("unexpected permittivity data: {:?}", rsp),
                ))
            }
        })
}

pub fn read_raw_counts(
    context: &mut client::Context,
    timeout: Duration,
) -> impl Future<Item = usize, Error = Error> {
    context
        .read_holding_registers(RAW_COUNTS_REG_START, RAW_COUNTS_REG_COUNT)
        .timeout(timeout)
        .map_err(move |err| {
            err.into_inner().unwrap_or_else(|| {
                Error::new(
                    ErrorKind::TimedOut,
                    String::from("reading raw counts timed out"),
                )
            })
        })
        .and_then(|rsp| {
            if let [raw] = rsp[..] {
                Ok(raw.into())
            } else {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("unexpected raw counts data: {:?}", rsp),
                ))
            }
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
        timeout: Duration,
    ) -> impl Future<Item = Temperature, Error = Error> {
        match self.shared_context() {
            Ok(shared_context) => {
                let mut context = shared_context.borrow_mut();
                context.set_slave(self.slave);
                future::Either::A(self::read_temperature(&mut context, timeout))
            }
            Err(err) => future::Either::B(future::err(err)),
        }
    }

    pub fn read_water_content(
        &self,
        timeout: Duration,
    ) -> impl Future<Item = VolumetricWaterContent, Error = Error> {
        match self.shared_context() {
            Ok(shared_context) => {
                let mut context = shared_context.borrow_mut();
                context.set_slave(self.slave);
                future::Either::A(self::read_water_content(&mut context, timeout))
            }
            Err(err) => future::Either::B(future::err(err)),
        }
    }

    pub fn read_permittivity(
        &self,
        timeout: Duration,
    ) -> impl Future<Item = RelativePermittivity, Error = Error> {
        match self.shared_context() {
            Ok(shared_context) => {
                let mut context = shared_context.borrow_mut();
                context.set_slave(self.slave);
                future::Either::A(self::read_permittivity(&mut context, timeout))
            }
            Err(err) => future::Either::B(future::err(err)),
        }
    }

    pub fn read_raw_counts(&self, timeout: Duration) -> impl Future<Item = usize, Error = Error> {
        match self.shared_context() {
            Ok(shared_context) => {
                let mut context = shared_context.borrow_mut();
                context.set_slave(self.slave);
                future::Either::A(self::read_raw_counts(&mut context, timeout))
            }
            Err(err) => future::Either::B(future::err(err)),
        }
    }
}

impl Capabilities for SlaveProxy {
    fn read_temperature(
        &self,
        timeout: Duration,
    ) -> Box<dyn Future<Item = Temperature, Error = Error>> {
        Box::new(self.read_temperature(timeout))
    }

    fn read_water_content(
        &self,
        timeout: Duration,
    ) -> Box<dyn Future<Item = VolumetricWaterContent, Error = Error>> {
        Box::new(self.read_water_content(timeout))
    }

    fn read_permittivity(
        &self,
        timeout: Duration,
    ) -> Box<dyn Future<Item = RelativePermittivity, Error = Error>> {
        Box::new(self.read_permittivity(timeout))
    }

    fn read_raw_counts(&self, timeout: Duration) -> Box<dyn Future<Item = usize, Error = Error>> {
        Box::new(self.read_raw_counts(timeout))
    }
}
