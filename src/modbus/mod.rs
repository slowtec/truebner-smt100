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
use tokio_modbus::prelude::*;

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

pub struct Context {
    context: client::Context,
}

impl Context {
    /// Implementation of Capabilities::read_temperature()
    pub fn read_temperature(
        &self,
        timeout: Duration,
    ) -> impl Future<Item = Temperature, Error = Error> {
        self.context
            .read_holding_registers(0x0000, 0x0001)
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("reading temperature failed: {}", err),
                )
            })
            .timeout(timeout)
            .map_err(|err| {
                Error::new(
                    ErrorKind::TimedOut,
                    format!("reading temperature timed out: {}", err),
                )
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

    /// Implementation of Capabilities::read_water_content()
    pub fn read_water_content(
        &self,
        timeout: Duration,
    ) -> impl Future<Item = VolumetricWaterContent, Error = Error> {
        self.context
            .read_holding_registers(0x0001, 0x0001)
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("reading water content failed: {}", err),
                )
            })
            .timeout(timeout)
            .map_err(|err| {
                Error::new(
                    ErrorKind::TimedOut,
                    format!("reading water content timed out: {}", err),
                )
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

    /// Implementation of Capabilities::read_permittivity()
    pub fn read_permittivity(
        &self,
        timeout: Duration,
    ) -> impl Future<Item = RelativePermittivity, Error = Error> {
        self.context
            .read_holding_registers(0x0002, 0x0001)
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("reading permittivity failed: {}", err),
                )
            })
            .timeout(timeout)
            .map_err(|err| {
                Error::new(
                    ErrorKind::TimedOut,
                    format!("reading permittivity timed out: {}", err),
                )
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

    /// Implementation of Capabilities::read_raw_counts()
    pub fn read_raw_counts(&self, timeout: Duration) -> impl Future<Item = usize, Error = Error> {
        self.context
            .read_holding_registers(0x0003, 0x0001)
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("reading raw counts failed: {}", err),
                )
            })
            .timeout(timeout)
            .map_err(|err| {
                Error::new(
                    ErrorKind::TimedOut,
                    format!("reading raw counts timed out: {}", err),
                )
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

    /// Permanently change the Modbus slave address/id of the device.
    pub fn init_slave(&self, slave: Slave) -> impl Future<Item = Slave, Error = Error> {
        let slave_id: SlaveId = slave.into();
        self.context
            .write_single_register(0x0004, u16::from(slave_id))
            .map(move |()| slave)
    }
}

impl SlaveContext for Context {
    fn set_slave(&mut self, slave: Slave) {
        self.context.set_slave(slave)
    }
}

impl Capabilities for Context {
    fn read_temperature(
        &self,
        timeout: Duration,
    ) -> Box<Future<Item = Temperature, Error = Error>> {
        Box::new(self.read_temperature(timeout))
    }

    fn read_water_content(
        &self,
        timeout: Duration,
    ) -> Box<Future<Item = VolumetricWaterContent, Error = Error>> {
        Box::new(self.read_water_content(timeout))
    }

    fn read_permittivity(
        &self,
        timeout: Duration,
    ) -> Box<Future<Item = RelativePermittivity, Error = Error>> {
        Box::new(self.read_permittivity(timeout))
    }

    fn read_raw_counts(&self, timeout: Duration) -> Box<Future<Item = usize, Error = Error>> {
        Box::new(self.read_raw_counts(timeout))
    }
}

pub struct SlaveProxy {
    context: Rc<RefCell<Context>>,
    slave: Slave,
}

impl SlaveProxy {
    pub fn new(context: Rc<RefCell<Context>>, slave: Slave) -> Self {
        Self { context, slave }
    }

    pub fn from_context(context: Context, slave: Slave) -> Self {
        let context = Rc::new(RefCell::new(context));
        Self::new(context, slave)
    }

    pub fn context(&self) -> Rc<RefCell<Context>> {
        Rc::clone(&self.context)
    }

    pub fn read_temperature(
        &self,
        timeout: Duration,
    ) -> impl Future<Item = Temperature, Error = Error> {
        let mut context = self.context.borrow_mut();
        context.set_slave(self.slave);
        context.read_temperature(timeout)
    }

    pub fn read_water_content(
        &self,
        timeout: Duration,
    ) -> impl Future<Item = VolumetricWaterContent, Error = Error> {
        let mut context = self.context.borrow_mut();
        context.set_slave(self.slave);
        context.read_water_content(timeout)
    }

    pub fn read_permittivity(
        &self,
        timeout: Duration,
    ) -> impl Future<Item = RelativePermittivity, Error = Error> {
        let mut context = self.context.borrow_mut();
        context.set_slave(self.slave);
        context.read_permittivity(timeout)
    }

    pub fn read_raw_counts(&self, timeout: Duration) -> impl Future<Item = usize, Error = Error> {
        let mut context = self.context.borrow_mut();
        context.set_slave(self.slave);
        context.read_raw_counts(timeout)
    }
}

impl Capabilities for SlaveProxy {
    fn read_temperature(
        &self,
        timeout: Duration,
    ) -> Box<Future<Item = Temperature, Error = Error>> {
        Box::new(self.read_temperature(timeout))
    }

    fn read_water_content(
        &self,
        timeout: Duration,
    ) -> Box<Future<Item = VolumetricWaterContent, Error = Error>> {
        Box::new(self.read_water_content(timeout))
    }

    fn read_permittivity(
        &self,
        timeout: Duration,
    ) -> Box<Future<Item = RelativePermittivity, Error = Error>> {
        Box::new(self.read_permittivity(timeout))
    }

    fn read_raw_counts(&self, timeout: Duration) -> Box<Future<Item = usize, Error = Error>> {
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
