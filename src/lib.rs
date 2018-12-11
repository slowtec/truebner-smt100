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
pub struct SoilMoisture {
    pub percent: f64,
}

impl SoilMoisture {
    pub fn is_valid(self) -> bool {
        self.percent >= 0.0 && self.percent <= 100.0
    }
}

/// Asynchronous generic device driver for the TRUEBNER SMT100 Soil Moisture Sensor.
pub trait Device {
    /// Measure the current temperature in the range from -40°C to +80°C
    /// (analog version from -40°C to +60°C).
    fn read_temperature(&self) -> Box<Future<Item = Temperature, Error = Error>>;

    /// Measure the current soil moisture in the range from 0% to 60%
    /// (up to 100% with limited accuracy).
    fn read_soil_moisture(&self) -> Box<Future<Item = SoilMoisture, Error = Error>>;
}
