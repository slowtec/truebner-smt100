#[cfg(any(feature = "modbus-rtu"))]
pub mod modbus;

#[cfg(any(feature = "mock"))]
pub mod mock;

use futures::Future;
use std::{fmt, io::Error, time::Duration};

/// (Thermodynamic) TemperatureDegreeCelsius.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct TemperatureDegreeCelsius(f64);

impl From<f64> for TemperatureDegreeCelsius {
    fn from(from: f64) -> Self {
        TemperatureDegreeCelsius(from)
    }
}

impl From<TemperatureDegreeCelsius> for f64 {
    fn from(from: TemperatureDegreeCelsius) -> Self {
        from.0
    }
}

impl fmt::Display for TemperatureDegreeCelsius {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} °C", f64::from(*self))
    }
}

/// Volumetric water content (VWC).
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct VolumetricWaterContentPercent(f64);

impl VolumetricWaterContentPercent {
    pub const fn min() -> Self {
        Self(0.0)
    }

    pub const fn max() -> Self {
        Self(100.0)
    }

    pub fn is_valid(self) -> bool {
        self >= Self::min() && self <= Self::max()
    }
}

impl From<f64> for VolumetricWaterContentPercent {
    fn from(from: f64) -> Self {
        VolumetricWaterContentPercent(from)
    }
}

impl From<VolumetricWaterContentPercent> for f64 {
    fn from(from: VolumetricWaterContentPercent) -> Self {
        from.0
    }
}

impl fmt::Display for VolumetricWaterContentPercent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} %", f64::from(*self))
    }
}

/// Relative permittivity or dielectric constant (DK).
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct RelativePermittivityRatio(f64);

impl RelativePermittivityRatio {
    pub fn min() -> Self {
        Self(1.0)
    }

    pub fn is_valid(self) -> bool {
        self >= Self::min()
    }
}

impl From<f64> for RelativePermittivityRatio {
    fn from(from: f64) -> Self {
        RelativePermittivityRatio(from)
    }
}

impl From<RelativePermittivityRatio> for f64 {
    fn from(from: RelativePermittivityRatio) -> Self {
        from.0
    }
}

impl fmt::Display for RelativePermittivityRatio {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", f64::from(*self))
    }
}

/// Asynchronous interface that exposes the generic capabilities of the
/// TRUEBNER SMT100 Soil Moisture Sensor.
pub trait Capabilities {
    /// Measure the current temperature in the range from -40°C to +80°C
    /// (analog version from -40°C to +60°C).
    fn read_temperature(&self, timeout: Duration)
        -> Box<Future<Item = TemperatureDegreeCelsius, Error = Error>>;

    /// Measure the current water content of the medium (soil) around the sensor
    /// in the range from 0% to 60% (up to 100% with limited accuracy).
    fn read_water_content(
        &self,
        timeout: Duration,
    ) -> Box<Future<Item = VolumetricWaterContentPercent, Error = Error>>;

    /// Measure the current (relative) permittivity of the medium around the sensor.
    fn read_permittivity(
        &self,
        timeout: Duration,
    ) -> Box<Future<Item = RelativePermittivityRatio, Error = Error>>;

    /// Retrieve the current raw and uncalibrated signal of the sensor.
    fn read_raw_counts(&self, timeout: Duration) -> Box<Future<Item = usize, Error = Error>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_content_percent() {
        for i in 0..=100 {
            let wvc = VolumetricWaterContentPercent::from(i as f64);
            assert!(wvc.is_valid());
            assert_eq!(f64::from(wvc), i as f64);
        }
        assert!(!VolumetricWaterContentPercent::from(-0.5).is_valid());
        assert!(!VolumetricWaterContentPercent::from(100.01).is_valid());
    }
}
