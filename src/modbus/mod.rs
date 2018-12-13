#[cfg(feature = "modbus-rtu")]
pub mod rtu;

use super::*;

use byteorder::{BigEndian, ReadBytesExt};
use futures::Future;
use std::io::{Cursor, Error, ErrorKind, Result};
use tokio_modbus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TemperatureRaw(pub u16);

impl From<TemperatureRaw> for Temperature {
    fn from(from: TemperatureRaw) -> Self {
        let celsius = f64::from(i32::from(from.0) - 10000i32) / 100f64;
        Self { celsius }
    }
}

pub fn decode_temperature_bytes(bytes: &[u8]) -> Result<Temperature> {
    let mut rdr = Cursor::new(bytes);
    let raw = rdr.read_u16::<BigEndian>()?;
    Ok(TemperatureRaw(raw).into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WaterContentRaw(pub u16);

impl From<WaterContentRaw> for WaterContent {
    fn from(from: WaterContentRaw) -> Self {
        let percent = f64::from(from.0) / 100f64;
        Self { percent }
    }
}

pub fn decode_water_content_bytes(bytes: &[u8]) -> Result<WaterContent> {
    let mut rdr = Cursor::new(bytes);
    let raw = rdr.read_u16::<BigEndian>()?;
    let res: WaterContent = WaterContentRaw(raw).into();
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
struct RelativePermittivityRaw(pub u16);

impl From<RelativePermittivityRaw> for RelativePermittivity {
    fn from(from: RelativePermittivityRaw) -> Self {
        let ratio = f64::from(from.0) / 100f64;
        Self { ratio }
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
    ctx: client::Context,
}

impl Context {
    fn new(ctx: client::Context) -> Self {
        Self { ctx }
    }
}

impl SlaveContext for Context {
    fn set_slave(&mut self, slave: Slave) {
        self.ctx.set_slave(slave)
    }
}

impl Sensor for Context {
    fn read_temperature(&self) -> Box<Future<Item = Temperature, Error = Error>> {
        let req = Request::ReadHoldingRegisters(0x0000, 0x0001);
        Box::new(self.ctx.call(req).and_then(|rsp| {
            if let Response::ReadHoldingRegisters(regs) = rsp {
                if let [raw] = regs[..] {
                    return Ok(TemperatureRaw(raw).into());
                }
            }
            Err(Error::new(ErrorKind::InvalidData, "Invalid response"))
        }))
    }

    fn read_water_content(&self) -> Box<Future<Item = WaterContent, Error = Error>> {
        let req = Request::ReadHoldingRegisters(0x0001, 0x0001);
        Box::new(self.ctx.call(req).and_then(|rsp| {
            if let Response::ReadHoldingRegisters(regs) = rsp {
                if let [raw] = regs[..] {
                    return Ok(WaterContentRaw(raw).into());
                }
            }
            Err(Error::new(ErrorKind::InvalidData, "Invalid response"))
        }))
    }

    fn read_permittivity(&self) -> Box<Future<Item = RelativePermittivity, Error = Error>> {
        let req = Request::ReadHoldingRegisters(0x0002, 0x0001);
        Box::new(self.ctx.call(req).and_then(|rsp| {
            if let Response::ReadHoldingRegisters(regs) = rsp {
                if let [raw] = regs[..] {
                    return Ok(RelativePermittivityRaw(raw).into());
                }
            }
            Err(Error::new(ErrorKind::InvalidData, "Invalid response"))
        }))
    }

    fn read_counts(&self) -> Box<Future<Item = usize, Error = Error>> {
        let req = Request::ReadHoldingRegisters(0x0003, 0x0001);
        Box::new(self.ctx.call(req).and_then(|rsp| {
            if let Response::ReadHoldingRegisters(regs) = rsp {
                if let [raw] = regs[..] {
                    return Ok(raw.into());
                }
            }
            Err(Error::new(ErrorKind::InvalidData, "Invalid response"))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_temperature() {
        assert_eq!(
            Temperature { celsius: -40.0 },
            decode_temperature_bytes(&[0x17, 0x70]).unwrap()
        );
        assert_eq!(
            Temperature { celsius: 0.0 },
            decode_temperature_bytes(&[0x27, 0x10]).unwrap()
        );
        assert_eq!(
            Temperature { celsius: 27.97 },
            decode_temperature_bytes(&[0x31, 0xFD]).unwrap()
        );
        assert_eq!(
            Temperature { celsius: 60.0 },
            decode_temperature_bytes(&[0x3E, 0x80]).unwrap()
        );
        assert_eq!(
            Temperature { celsius: 80.0 },
            decode_temperature_bytes(&[0x46, 0x50]).unwrap()
        );
    }

    #[test]
    fn decode_water_content() {
        // Valid range
        assert_eq!(
            WaterContent { percent: 0.00 },
            decode_water_content_bytes(&[0x00, 0x00]).unwrap()
        );
        assert_eq!(
            WaterContent { percent: 34.4 },
            decode_water_content_bytes(&[0x0D, 0x70]).unwrap()
        );
        assert_eq!(
            WaterContent { percent: 100.0 },
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
            RelativePermittivity { ratio: 1.00 },
            decode_permittivity_bytes(&[0x00, 0x64]).unwrap()
        );
        assert_eq!(
            RelativePermittivity { ratio: 15.2 },
            decode_permittivity_bytes(&[0x05, 0xF0]).unwrap()
        );
        // Invalid range
        assert!(decode_permittivity_bytes(&[0x00, 0x00]).is_err());
        assert!(decode_permittivity_bytes(&[0x00, 0x63]).is_err());
    }
}
