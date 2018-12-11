#[cfg(any(feature = "modbus-rtu"))]
pub mod modbus;

use futures::Future;
use std::io::Error;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
/// Temperature (°C).
pub struct Temperature {
    pub celsius: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
/// Soil volumetric water content (VWC).
pub struct WaterContent {
    pub percent: f64,
}

impl WaterContent {
    pub const fn min() -> Self {
        Self { percent: 0.0 }
    }

    pub const fn max() -> Self {
        Self { percent: 100.0 }
    }

    pub fn is_valid(self) -> bool {
        self >= Self::min() && self <= Self::max()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
/// Relative permittivity or dielectric constant (DK).
pub struct RelativePermittivity {
    pub ratio: f64,
}

impl RelativePermittivity {
    pub const fn min() -> Self {
        Self { ratio: 1.0 }
    }

    pub fn is_valid(self) -> bool {
        self >= Self::min()
    }
}

/// Asynchronous generic device driver for the TRUEBNER SMT100 Soil Moisture Sensor.
pub trait Device {
    /// Measure the current temperature in the range from -40°C to +80°C
    /// (analog version from -40°C to +60°C).
    fn read_temperature(&self) -> Box<Future<Item = Temperature, Error = Error>>;

    /// Measure the current water content of the medium (soil) around the sensor
    /// in the range from 0% to 60% (up to 100% with limited accuracy).
    fn read_water_content(&self) -> Box<Future<Item = WaterContent, Error = Error>>;

    /// Measure the current (relative) permittivity of the medium around the sensor.
    fn read_permittivity(&self) -> Box<Future<Item = RelativePermittivity, Error = Error>>;

    /// Retrieve the current raw and uncalibrated signal of the sensor.
    fn read_counts(&self) -> Box<Future<Item = usize, Error = Error>>;
}
