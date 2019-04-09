#[cfg(feature = "modbus-rtu")]
pub mod rtu;

use super::*;

use byteorder::{BigEndian, ReadBytesExt};
use futures::Future;
use std::{
    cell::RefCell,
    io::{Cursor, Error, ErrorKind, Result},
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
pub const BROADCAST_SLAVE: Slave = Slave(0xFD);

/// Switch the Modbus slave address of all connected devices.
pub fn broadcast_slave(
    context: &mut client::Context,
    slave: Slave,
) -> impl Future<Item = (), Error = Error> {
    context.set_slave(rtu::BROADCAST_SLAVE);
    let slave_id: SlaveId = slave.into();
    context.write_single_register(0x0004, u16::from(slave_id))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TemperatureRaw(u16);

impl From<TemperatureRaw> for Temperature {
    fn from(from: TemperatureRaw) -> Self {
        let degree_celsius = f64::from(i32::from(from.0) - 10000i32) / 100f64;
        Self::from_degree_celsius(degree_celsius)
    }
}

pub fn decode_temperature_bytes(bytes: &[u8]) -> Result<Temperature> {
    let mut rdr = Cursor::new(bytes);
    let raw = rdr.read_u16::<BigEndian>()?;
    Ok(TemperatureRaw(raw).into())
}

pub fn read_temperature(
    context: &mut client::Context,
    timeout: Duration,
) -> impl Future<Item = Temperature, Error = Error> {
    context
        .read_holding_registers(0x0000, 0x0001)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct VolumetricWaterContentRaw(u16);

impl From<VolumetricWaterContentRaw> for VolumetricWaterContent {
    fn from(from: VolumetricWaterContentRaw) -> Self {
        let percent = f64::from(from.0) / 100f64;
        Self::from_percent(percent)
    }
}

pub fn decode_water_content_bytes(bytes: &[u8]) -> Result<VolumetricWaterContent> {
    let mut rdr = Cursor::new(bytes);
    let raw = rdr.read_u16::<BigEndian>()?;
    let res: VolumetricWaterContent = VolumetricWaterContentRaw(raw).into();
    if res.is_valid() {
        Ok(res)
    } else {
        Err(Error::new(
            ErrorKind::InvalidData,
            format!("Water content out of range: {:?}", res),
        ))
    }
}

pub fn read_water_content(
    context: &mut client::Context,
    timeout: Duration,
) -> impl Future<Item = VolumetricWaterContent, Error = Error> {
    context
        .read_holding_registers(0x0001, 0x0001)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RelativePermittivityRaw(u16);

impl From<RelativePermittivityRaw> for RelativePermittivity {
    fn from(from: RelativePermittivityRaw) -> Self {
        let ratio = f64::from(from.0) / 100f64;
        Self::from_ratio(ratio)
    }
}

pub fn decode_permittivity_bytes(bytes: &[u8]) -> Result<RelativePermittivity> {
    let mut rdr = Cursor::new(bytes);
    let raw = rdr.read_u16::<BigEndian>()?;
    let res: RelativePermittivity = RelativePermittivityRaw(raw).into();
    if res.is_valid() {
        Ok(res)
    } else {
        Err(Error::new(
            ErrorKind::InvalidData,
            format!("Relative permittivity out of range: {:?}", res),
        ))
    }
}

pub fn read_permittivity(
    context: &mut client::Context,
    timeout: Duration,
) -> impl Future<Item = RelativePermittivity, Error = Error> {
    context
        .read_holding_registers(0x0002, 0x0001)
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
        .read_holding_registers(0x0003, 0x0001)
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

    pub fn reconnect(self) -> impl Future<Item = Self, Error = (Error, Self)> {
        reconnect_shared_context(&self.shared_context).then(move |res| match res {
            Ok(()) => Ok(self),
            Err(err) => Err((err, self)),
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_temperature() {
        assert_eq!(
            Temperature::from_degree_celsius(-40.0),
            decode_temperature_bytes(&[0x17, 0x70]).unwrap()
        );
        assert_eq!(
            Temperature::from_degree_celsius(0.0),
            decode_temperature_bytes(&[0x27, 0x10]).unwrap()
        );
        assert_eq!(
            Temperature::from_degree_celsius(27.97),
            decode_temperature_bytes(&[0x31, 0xFD]).unwrap()
        );
        assert_eq!(
            Temperature::from_degree_celsius(60.0),
            decode_temperature_bytes(&[0x3E, 0x80]).unwrap()
        );
        assert_eq!(
            Temperature::from_degree_celsius(80.0),
            decode_temperature_bytes(&[0x46, 0x50]).unwrap()
        );
    }

    #[test]
    fn decode_water_content() {
        // Valid range
        assert_eq!(
            VolumetricWaterContent::from_percent(0.0),
            decode_water_content_bytes(&[0x00, 0x00]).unwrap()
        );
        assert_eq!(
            VolumetricWaterContent::from_percent(34.4),
            decode_water_content_bytes(&[0x0D, 0x70]).unwrap()
        );
        assert_eq!(
            VolumetricWaterContent::from_percent(100.0),
            decode_water_content_bytes(&[0x27, 0x10]).unwrap()
        );
        // Invalid range
        assert!(decode_water_content_bytes(&[0x27, 0x11]).is_err());
        assert!(decode_water_content_bytes(&[0xFF, 0xFF]).is_err());
    }

    #[test]
    fn decode_permittivity() {
        // Valid range
        assert_eq!(
            RelativePermittivity::from_ratio(1.0),
            decode_permittivity_bytes(&[0x00, 0x64]).unwrap()
        );
        assert_eq!(
            RelativePermittivity::from_ratio(15.2),
            decode_permittivity_bytes(&[0x05, 0xF0]).unwrap()
        );
        // Invalid range
        assert!(decode_permittivity_bytes(&[0x00, 0x00]).is_err());
        assert!(decode_permittivity_bytes(&[0x00, 0x63]).is_err());
    }
}
