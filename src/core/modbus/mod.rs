use super::*;

#[cfg(feature = "rtu")]
pub mod rtu;

use core::{fmt, mem, convert::TryInto};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    InsufficientInput,
    InvalidInput,
    InvalidData,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DecodeError::*;
        match self {
            InsufficientInput => write!(f, "Insufficient input"),
            InvalidInput => write!(f, "Invalid input"),
            InvalidData => write!(f, "Invalid data"),
        }

    }
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}

pub type DecodeResult<T> = Result<T, DecodeError>;

fn decode_be_u16_from_bytes(input: &[u8]) -> DecodeResult<(u16, &[u8])> {
    if input.len() < mem::size_of::<u16>() {
        return Err(DecodeError::InsufficientInput);
    }
    let (head, rest) = input.split_at(mem::size_of::<u16>());
    if let Ok(bytes) = head.try_into() {
        Ok((u16::from_be_bytes(bytes), rest))
    } else {
        Err(DecodeError::InvalidInput)
    }
}

pub const TEMPERATURE_REG_START: u16 = 0x0000;
pub const TEMPERATURE_REG_COUNT: u16 = 0x0001;

pub fn decode_temperature_from_u16(input: u16) -> DecodeResult<Temperature> {
    let degree_celsius = f64::from(i32::from(input) - 10000i32) / 100f64;
    Ok(Temperature::from_degree_celsius(degree_celsius))
}

pub fn decode_temperature_from_bytes(input: &[u8]) -> DecodeResult<(Temperature, &[u8])> {
    decode_be_u16_from_bytes(input).and_then(|(val, rest)| Ok((decode_temperature_from_u16(val)?, rest)))
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

pub fn decode_water_content_from_u16(input: u16) -> DecodeResult<VolumetricWaterContent> {
    let percent = f64::from(input) / 100f64;
    let res = VolumetricWaterContent::from_percent(percent);
    if res.is_valid() {
        Ok(res)
    } else {
        Err(DecodeError::InvalidData)
    }
}

pub fn decode_water_content_from_bytes(input: &[u8]) -> DecodeResult<(VolumetricWaterContent, &[u8])> {
    decode_be_u16_from_bytes(input).and_then(|(val, rest)| Ok((decode_water_content_from_u16(val)?, rest)))
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

pub fn decode_permittivity_from_u16(input: u16) -> DecodeResult<RelativePermittivity> {
    let ratio = f64::from(input) / 100f64;
    let res = RelativePermittivity::from_ratio(ratio);
    if res.is_valid() {
        Ok(res)
    } else {
        Err(DecodeError::InvalidData)
    }
}

pub fn decode_permittivity_from_bytes(input: &[u8]) -> DecodeResult<(RelativePermittivity, &[u8])> {
    decode_be_u16_from_bytes(input).and_then(|(val, rest)| Ok((decode_permittivity_from_u16(val)?, rest)))
}

pub const RAW_COUNTS_REG_START: u16 = 0x0003;
pub const RAW_COUNTS_REG_COUNT: u16 = 0x0001;

#[inline]
pub fn decode_raw_counts_from_u16(input: u16) -> DecodeResult<RawCounts> {
    Ok(input.into())
}

#[inline]
pub fn decode_raw_counts_from_bytes(input: &[u8]) -> DecodeResult<(RawCounts, &[u8])> {
    decode_be_u16_from_bytes(input).and_then(|(val, rest)| Ok((decode_raw_counts_from_u16(val)?, rest)))
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
            decode_temperature_from_bytes(&[0x17, 0x70]).unwrap().0
        );
        assert_eq!(
            Temperature::from_degree_celsius(0.0),
            decode_temperature_from_bytes(&[0x27, 0x10]).unwrap().0
        );
        assert_eq!(
            Temperature::from_degree_celsius(27.97),
            decode_temperature_from_bytes(&[0x31, 0xFD]).unwrap().0
        );
        assert_eq!(
            Temperature::from_degree_celsius(60.0),
            decode_temperature_from_bytes(&[0x3E, 0x80]).unwrap().0
        );
        assert_eq!(
            Temperature::from_degree_celsius(80.0),
            decode_temperature_from_bytes(&[0x46, 0x50]).unwrap().0
        );
    }

    #[test]
    fn decode_water_content() {
        // Valid range
        assert_eq!(
            VolumetricWaterContent::from_percent(0.0),
            decode_water_content_from_bytes(&[0x00, 0x00]).unwrap().0
        );
        assert_eq!(
            VolumetricWaterContent::from_percent(34.4),
            decode_water_content_from_bytes(&[0x0D, 0x70]).unwrap().0
        );
        assert_eq!(
            VolumetricWaterContent::from_percent(100.0),
            decode_water_content_from_bytes(&[0x27, 0x10]).unwrap().0
        );
        // Invalid range
        assert!(decode_water_content_from_bytes(&[0x27, 0x11]).is_err());
        assert!(decode_water_content_from_bytes(&[0xFF, 0xFF]).is_err());
    }

    #[test]
    fn decode_permittivity() {
        // Valid range
        assert_eq!(
            RelativePermittivity::from_ratio(1.0),
            decode_permittivity_from_bytes(&[0x00, 0x64]).unwrap().0
        );
        assert_eq!(
            RelativePermittivity::from_ratio(15.2),
            decode_permittivity_from_bytes(&[0x05, 0xF0]).unwrap().0
        );
        // Invalid range
        assert!(decode_permittivity_from_bytes(&[0x00, 0x00]).is_err());
        assert!(decode_permittivity_from_bytes(&[0x00, 0x63]).is_err());
    }
}
