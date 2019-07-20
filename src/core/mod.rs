#[cfg(feature = "modbus")]
pub mod modbus;

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
