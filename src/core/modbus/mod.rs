use super::*;

#[cfg(feature = "rtu")]
pub mod rtu;

use byteorder::{ByteOrder, BigEndian};
use core::result::Result;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    InvalidInput,
    InvalidData,
}

pub type DecodeResult<T> = Result<T, DecodeError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemperatureRaw(pub u16);

impl From<TemperatureRaw> for Temperature {
    fn from(from: TemperatureRaw) -> Self {
        let degree_celsius = f64::from(i32::from(from.0) - 10000i32) / 100f64;
        Self::from_degree_celsius(degree_celsius)
    }
}

pub const TEMPERATURE_REG_START: u16 = 0x0000;
pub const TEMPERATURE_REG_COUNT: u16 = 0x0001;

const TEMPERATURE_BYTES_LEN: usize = TEMPERATURE_REG_COUNT as usize * 2;

pub fn decode_temperature_bytes(bytes: &[u8]) -> DecodeResult<(Temperature, &[u8])> {
    if bytes.len() < TEMPERATURE_BYTES_LEN {
        return Err(DecodeError::InvalidInput);
    }
    let raw = BigEndian::read_u16(bytes);
    Ok((TemperatureRaw(raw).into(), &bytes[TEMPERATURE_BYTES_LEN..]))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VolumetricWaterContentRaw(pub u16);

impl From<VolumetricWaterContentRaw> for VolumetricWaterContent {
    fn from(from: VolumetricWaterContentRaw) -> Self {
        let percent = f64::from(from.0) / 100f64;
        Self::from_percent(percent)
    }
}

pub const WATER_CONTENT_REG_START: u16 = 0x0001;
pub const WATER_CONTENT_REG_COUNT: u16 = 0x0001;

const WATER_CONTENT_BYTES_LEN: usize = WATER_CONTENT_REG_COUNT as usize * 2;

pub fn decode_water_content_bytes(bytes: &[u8]) -> DecodeResult<(VolumetricWaterContent, &[u8])> {
    if bytes.len() < WATER_CONTENT_BYTES_LEN {
        return Err(DecodeError::InvalidInput);
    }
    let raw = BigEndian::read_u16(bytes);
    let val: VolumetricWaterContent = VolumetricWaterContentRaw(raw).into();
    if !val.is_valid() {
        return Err(DecodeError::InvalidData);
    }
    Ok((val, &bytes[WATER_CONTENT_BYTES_LEN..]))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelativePermittivityRaw(pub u16);

impl From<RelativePermittivityRaw> for RelativePermittivity {
    fn from(from: RelativePermittivityRaw) -> Self {
        let ratio = f64::from(from.0) / 100f64;
        Self::from_ratio(ratio)
    }
}

pub const PERMITTIVITY_REG_START: u16 = 0x0002;
pub const PERMITTIVITY_REG_COUNT: u16 = 0x0001;

const PERMITTIVITY_BYTES_LEN: usize = PERMITTIVITY_REG_COUNT as usize * 2;

pub fn decode_permittivity_bytes(bytes: &[u8]) -> DecodeResult<(RelativePermittivity, &[u8])> {
    if bytes.len() < PERMITTIVITY_BYTES_LEN {
        return Err(DecodeError::InvalidInput);
    }
    let raw = BigEndian::read_u16(bytes);
    let val: RelativePermittivity = RelativePermittivityRaw(raw).into();
    if !val.is_valid() {
        return Err(DecodeError::InvalidData);
    }
    Ok((val, &bytes[PERMITTIVITY_BYTES_LEN..]))
}

pub const RAW_COUNTS_REG_START: u16 = 0x0003;
pub const RAW_COUNTS_REG_COUNT: u16 = 0x0001;

const RAW_COUNTS_BYTES_LEN: usize = RAW_COUNTS_REG_COUNT as usize * 2;

pub fn decode_raw_counts(bytes: &[u8]) -> DecodeResult<(u16, &[u8])> {
    if bytes.len() < RAW_COUNTS_BYTES_LEN {
        return Err(DecodeError::InvalidInput);
    }
    let val = BigEndian::read_u16(bytes);
    Ok((val, &bytes[RAW_COUNTS_BYTES_LEN..]))
}

pub const BROADCAST_SLAVE_ADDR: u8 = 0xFD;
pub const BROADCAST_REG_ADDR: u16 = 0x0004;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_temperature() {
        assert_eq!(
            Temperature::from_degree_celsius(-40.0),
            decode_temperature_bytes(&[0x17, 0x70]).unwrap().0
        );
        assert_eq!(
            Temperature::from_degree_celsius(0.0),
            decode_temperature_bytes(&[0x27, 0x10]).unwrap().0
        );
        assert_eq!(
            Temperature::from_degree_celsius(27.97),
            decode_temperature_bytes(&[0x31, 0xFD]).unwrap().0
        );
        assert_eq!(
            Temperature::from_degree_celsius(60.0),
            decode_temperature_bytes(&[0x3E, 0x80]).unwrap().0
        );
        assert_eq!(
            Temperature::from_degree_celsius(80.0),
            decode_temperature_bytes(&[0x46, 0x50]).unwrap().0
        );
    }

    #[test]
    fn decode_water_content() {
        // Valid range
        assert_eq!(
            VolumetricWaterContent::from_percent(0.0),
            decode_water_content_bytes(&[0x00, 0x00]).unwrap().0
        );
        assert_eq!(
            VolumetricWaterContent::from_percent(34.4),
            decode_water_content_bytes(&[0x0D, 0x70]).unwrap().0
        );
        assert_eq!(
            VolumetricWaterContent::from_percent(100.0),
            decode_water_content_bytes(&[0x27, 0x10]).unwrap().0
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
            decode_permittivity_bytes(&[0x00, 0x64]).unwrap().0
        );
        assert_eq!(
            RelativePermittivity::from_ratio(15.2),
            decode_permittivity_bytes(&[0x05, 0xF0]).unwrap().0
        );
        // Invalid range
        assert!(decode_permittivity_bytes(&[0x00, 0x00]).is_err());
        assert!(decode_permittivity_bytes(&[0x00, 0x63]).is_err());
    }
}
