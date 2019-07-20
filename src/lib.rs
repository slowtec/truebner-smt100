pub mod core;

pub use self::core::*;

#[cfg(feature = "modbus-rtu")]
pub mod modbus;

#[cfg(feature = "mock")]
pub mod mock;

use futures::Future;
use std::{fmt, io::Error, time::Duration};

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} °C", self.to_degree_celsius())
    }
}

impl fmt::Display for VolumetricWaterContent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} %", self.to_percent())
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
        -> Box<dyn Future<Item = Temperature, Error = Error>>;

    /// Measure the current water content of the medium (soil) around the sensor
    /// in the range from 0% to 60% (up to 100% with limited accuracy).
    fn read_water_content(
        &self,
        timeout: Duration,
    ) -> Box<dyn Future<Item = VolumetricWaterContent, Error = Error>>;

    /// Measure the current (relative) permittivity of the medium around the sensor.
    fn read_permittivity(
        &self,
        timeout: Duration,
    ) -> Box<dyn Future<Item = RelativePermittivity, Error = Error>>;

    /// Retrieve the current raw and uncalibrated signal of the sensor.
    fn read_raw_counts(&self, timeout: Duration) -> Box<dyn Future<Item = usize, Error = Error>>;
}
