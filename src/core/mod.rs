#[cfg(feature = "modbus")]
pub mod modbus;

use core::{fmt, result::Result, time::Duration};

/// (Thermodynamic) Temperature.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Temperature(f64);

impl Temperature {
    pub const fn from_degree_celsius(degree_celsius: f64) -> Self {
        Self(degree_celsius)
    }

    pub const fn to_degree_celsius(self) -> f64 {
        self.0
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} °C", self.to_degree_celsius())
    }
}

/// Volumetric water content (VWC).
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct VolumetricWaterContent(f64);

impl VolumetricWaterContent {
    pub const fn from_percent(percent: f64) -> Self {
        Self(percent)
    }

    pub const fn to_percent(self) -> f64 {
        self.0
    }

    pub const fn min_percent() -> f64 {
        0.0
    }

    pub const fn max_percent() -> f64 {
        100.0
    }

    pub const fn min() -> Self {
        Self::from_percent(Self::min_percent())
    }

    pub const fn max() -> Self {
        Self::from_percent(Self::max_percent())
    }

    pub fn is_valid(self) -> bool {
        self >= Self::min() && self <= Self::max()
    }
}

impl fmt::Display for VolumetricWaterContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} %", self.to_percent())
    }
}

/// Relative permittivity or dielectric constant (DK).
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct RelativePermittivity(f64);

impl RelativePermittivity {
    pub const fn from_ratio(percent: f64) -> Self {
        Self(percent)
    }

    pub const fn to_ratio(self) -> f64 {
        self.0
    }

    pub const fn min_ratio() -> f64 {
        1.0
    }

    pub const fn min() -> Self {
        Self::from_ratio(Self::min_ratio())
    }

    pub fn is_valid(self) -> bool {
        self >= Self::min()
    }
}

impl fmt::Display for RelativePermittivity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} %", self.to_ratio())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RawCounts(u16);

impl From<u16> for RawCounts {
    fn from(from: u16) -> Self {
        RawCounts(from)
    }
}

impl From<RawCounts> for u16 {
    fn from(from: RawCounts) -> Self {
        from.0
    }
}

impl fmt::Display for RawCounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Blocking interface that exposes the generic capabilities of the
/// TRUEBNER SMT100 Soil Moisture Sensor.
pub trait Capabilities {
    type ReadError;

    /// Measure the current temperature in the range from -40°C to +80°C
    /// (analog version from -40°C to +60°C).
    fn read_temperature(&self, timeout: Option<Duration>) -> Result<Temperature, Self::ReadError>;

    /// Measure the current water content of the medium (soil) around the sensor
    /// in the range from 0% to 60% (up to 100% with limited accuracy).
    fn read_water_content(
        &self,
        timeout: Option<Duration>,
    ) -> Result<VolumetricWaterContent, Self::ReadError>;

    /// Measure the current (relative) permittivity of the medium around the sensor.
    fn read_permittivity(
        &self,
        timeout: Option<Duration>,
    ) -> Result<RelativePermittivity, Self::ReadError>;

    /// Retrieve the current raw and uncalibrated signal of the sensor.
    fn read_raw_counts(&self, timeout: Option<Duration>) -> Result<RawCounts, Self::ReadError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::float_cmp)]
    fn water_content_percent() {
        for i in 0..=100 {
            let vwc = VolumetricWaterContent::from_percent(f64::from(i));
            assert!(vwc.is_valid());
            assert_eq!(vwc.to_percent(), f64::from(i));
        }
        assert!(!VolumetricWaterContent::from_percent(-0.5).is_valid());
        assert!(!VolumetricWaterContent::from_percent(100.01).is_valid());
    }
}
