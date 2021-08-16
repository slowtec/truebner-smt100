#![cfg_attr(not(feature = "std"), no_std)]

/// The no_std enclave
pub mod core;

pub use self::core::*;

#[cfg(feature = "tokio-modbus-rtu")]
pub mod modbus;

#[cfg(feature = "tokio-mock")]
pub mod mock;

#[cfg(feature = "std")]
use std::{io::Error, time::Duration};

#[cfg(feature = "std")]
use async_trait::async_trait;

#[cfg(feature = "std")]
type Result<T> = std::result::Result<T, Error>;

/// Asynchronous interface that exposes the generic capabilities of the
/// TRUEBNER SMT100 Soil Moisture Sensor.
#[cfg(feature = "std")]
#[async_trait(?Send)]
pub trait Capabilities {
    /// Measure the current temperature in the range from -40°C to +80°C
    /// (analog version from -40°C to +60°C).
    async fn read_temperature(&self, timeout: Option<Duration>) -> Result<Temperature>;

    /// Measure the current water content of the medium (soil) around the sensor
    /// in the range from 0% to 60% (up to 100% with limited accuracy).
    async fn read_water_content(&self, timeout: Option<Duration>)
        -> Result<VolumetricWaterContent>;

    /// Measure the current (relative) permittivity of the medium around the sensor.
    async fn read_permittivity(&self, timeout: Option<Duration>) -> Result<RelativePermittivity>;

    /// Retrieve the current raw and uncalibrated signal of the sensor.
    async fn read_raw_counts(&self, timeout: Option<Duration>) -> Result<RawCounts>;
}
