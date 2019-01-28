#[cfg(any(feature = "modbus-rtu"))]
pub mod modbus;

#[cfg(any(feature = "mock"))]
pub mod mock;

use futures::Future;
use std::{fmt, io::Error, time::Duration};

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

impl From<f64> for Temperature {
    fn from(from: f64) -> Self {
        Temperature(from)
    }
}

impl From<Temperature> for f64 {
    fn from(from: Temperature) -> Self {
        from.0
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_ratio())
    }
}

/// Asynchronous interface that exposes the generic capabilities of the
/// TRUEBNER SMT100 Soil Moisture Sensor.
pub trait Capabilities {
    /// Measure the current temperature in the range from -40°C to +80°C
    /// (analog version from -40°C to +60°C).
    fn read_temperature(&self, timeout: Duration)
        -> Box<Future<Item = Temperature, Error = Error>>;

    /// Measure the current water content of the medium (soil) around the sensor
    /// in the range from 0% to 60% (up to 100% with limited accuracy).
    fn read_water_content(
        &self,
        timeout: Duration,
    ) -> Box<Future<Item = VolumetricWaterContent, Error = Error>>;

    /// Measure the current (relative) permittivity of the medium around the sensor.
    fn read_permittivity(
        &self,
        timeout: Duration,
    ) -> Box<Future<Item = RelativePermittivity, Error = Error>>;

    /// Retrieve the current raw and uncalibrated signal of the sensor.
    fn read_raw_counts(&self, timeout: Duration) -> Box<Future<Item = usize, Error = Error>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_content_percent() {
        for i in 0..=100 {
            let vwc = VolumetricWaterContent::from_percent(i as f64);
            assert!(vwc.is_valid());
            assert_eq!(vwc.to_percent(), i as f64);
        }
        assert!(!VolumetricWaterContent::from_percent(-0.5).is_valid());
        assert!(!VolumetricWaterContent::from_percent(100.01).is_valid());
    }
}
