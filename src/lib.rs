#[cfg(any(feature = "modbus-rtu"))]
pub mod modbus;

use futures::Future;
use std::{fmt, io::Error};
use uom::si::f64;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
/// /Thermodynamic) Temperature.
pub struct Temperature(f64::ThermodynamicTemperature);

impl Temperature {
    pub /*const*/ fn from_degree_celsius(degree_celsius: f64) -> Self {
        Temperature(f64::ThermodynamicTemperature::new::<uom::si::thermodynamic_temperature::degree_celsius>(degree_celsius))
    }

    /// The value in degree celsius with a precision of 2 decimal places.
    pub fn degree_celsius(self) -> f64 {
        self.0.get::<uom::si::thermodynamic_temperature::degree_celsius>()
    }


}

impl From<f64::ThermodynamicTemperature> for Temperature {
    fn from(from: f64::ThermodynamicTemperature) -> Self {
        Temperature(from)
    }
}

impl From<Temperature> for f64::ThermodynamicTemperature {
    fn from(from: Temperature) -> Self {
        from.0
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}°C", self.degree_celsius())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
/// Soil volumetric water content (VWC).
pub struct VolumetricWaterContent(f64::Ratio);

impl VolumetricWaterContent {
    pub /*const*/ fn from_percent(percent: f64) -> Self {
        VolumetricWaterContent(f64::Ratio::new::<uom::si::ratio::percent>(percent))
    }

    /// The value in precent with a precision of 2 decimal places.
    pub fn percent(self) -> f64 {
        self.0.get::<uom::si::ratio::percent>()
    }

    pub /*const*/ fn min() -> Self {
        Self::from_percent(0.0)
    }

    pub /*const*/ fn max() -> Self {
        Self::from_percent(100.0)
    }

    pub fn is_valid(self) -> bool {
        self >= Self::min() && self <= Self::max()
    }
}

impl From<f64::Ratio> for VolumetricWaterContent {
    fn from(from: f64::Ratio) -> Self {
        VolumetricWaterContent(from)
    }
}

impl From<VolumetricWaterContent> for f64::Ratio {
    fn from(from: VolumetricWaterContent) -> Self {
        from.0
    }
}

impl fmt::Display for VolumetricWaterContent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}%", self.percent())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
/// Relative permittivity or dielectric constant (DK).
pub struct RelativePermittivity(f64::Ratio);

impl RelativePermittivity {
    pub /*const*/ fn from_ratio(ratio: f64) -> Self {
        RelativePermittivity(f64::Ratio::new::<uom::si::ratio::ratio>(ratio))
    }

    /// The ration with a precision of 2 decimal places.
    pub fn ratio(self) -> f64 {
        self.0.get::<uom::si::ratio::ratio>()
    }

    pub /*const*/ fn min() -> Self {
        Self::from_ratio(1.0)
    }

    pub fn is_valid(self) -> bool {
        self >= Self::min()
    }
}

impl From<f64::Ratio> for RelativePermittivity {
    fn from(from: f64::Ratio) -> Self {
        RelativePermittivity(from)
    }
}

impl From<RelativePermittivity> for f64::Ratio {
    fn from(from: RelativePermittivity) -> Self {
        from.0
    }
}

impl fmt::Display for RelativePermittivity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ratio())
    }
}

/// Asynchronous generic device driver interface for the TRUEBNER SMT100 Soil Moisture Sensor.
pub trait Sensor {
    /// Measure the current temperature in the range from -40°C to +80°C
    /// (analog version from -40°C to +60°C).
    fn read_temperature(&self) -> Box<Future<Item = Temperature, Error = Error>>;

    /// Measure the current water content of the medium (soil) around the sensor
    /// in the range from 0% to 60% (up to 100% with limited accuracy).
    fn read_water_content(&self) -> Box<Future<Item = VolumetricWaterContent, Error = Error>>;

    /// Measure the current (relative) permittivity of the medium around the sensor.
    fn read_permittivity(&self) -> Box<Future<Item = RelativePermittivity, Error = Error>>;

    /// Retrieve the current raw and uncalibrated signal of the sensor.
    fn read_counts(&self) -> Box<Future<Item = usize, Error = Error>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_content_percent() {
        for i in 0..=100 {
            assert!(VolumetricWaterContent::from_percent(i as f64).is_valid());
            assert!(
                (i as f64 - VolumetricWaterContent::from_percent(i as f64).percent()).abs() < 0.000001,
            );
        }
        assert!(!VolumetricWaterContent::from_percent(-0.5).is_valid());
        assert!(!VolumetricWaterContent::from_percent(100.01).is_valid());
    }
}