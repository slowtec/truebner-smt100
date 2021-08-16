use crate::*;
use std::{
    cell::Cell,
    io::{Error, ErrorKind, Result},
    time::Duration,
};
use tokio::time::delay_until;

pub struct Proxy {
    temperature: Temperature,
    water_content: VolumetricWaterContent,
    permittivity: RelativePermittivity,
    raw_counts: RawCounts,
    delay: Duration,
    next_error: Cell<Option<Error>>,
}

pub trait Driver {
    fn set_delay(&mut self, delay: Duration);

    fn set_next_error(&mut self, next_error: Option<Error>);

    fn set_temperature(&mut self, temperature: Temperature);

    fn set_water_content(&mut self, water_content: VolumetricWaterContent);

    fn set_permittivity(&mut self, permittivity: RelativePermittivity);

    fn set_raw_counts(&mut self, raw_counts: RawCounts);
}

impl Proxy {
    pub fn default_temperature() -> Temperature {
        Temperature::from_degree_celsius(20.0)
    }

    pub fn default_water_content() -> VolumetricWaterContent {
        VolumetricWaterContent::from_percent(30.0)
    }

    pub fn default_permittivity() -> RelativePermittivity {
        RelativePermittivity::min()
    }

    pub fn default_raw_counts() -> RawCounts {
        Default::default()
    }

    async fn delay_value<T>(&self, value: T, timeout: Duration) -> Result<T>
    where
        T: 'static,
    {
        let deadline = tokio::time::Instant::now() + self.delay;
        let next_error = self.next_error.replace(None);
        let result = if let Some(error) = next_error {
            Err(error)
        } else {
            Ok(value)
        };
        let delay = delay_until(deadline);
        tokio::time::timeout(timeout, delay)
            .await
            .map_err(move |_| {
                Error::new(ErrorKind::TimedOut, String::from("reading value timed out"))
            })
            .and(result)
            .map_err(|err| Error::new(ErrorKind::Other, format!("reading value failed: {}", err)))
    }

    /// Implementation of Capabilities::read_temperature()
    pub async fn read_temperature(&self, timeout: Option<Duration>) -> Result<Temperature> {
        if let Some(timeout) = timeout {
            self.delay_value(self.temperature, timeout).await
        } else {
            Ok(self.temperature)
        }
    }

    /// Implementation of Capabilities::read_water_content()
    pub async fn read_water_content(
        &self,
        timeout: Option<Duration>,
    ) -> Result<VolumetricWaterContent> {
        if let Some(timeout) = timeout {
            self.delay_value(self.water_content, timeout).await
        } else {
            Ok(self.water_content)
        }
    }

    /// Implementation of Capabilities::read_permittivity()
    pub async fn read_permittivity(
        &self,
        timeout: Option<Duration>,
    ) -> Result<RelativePermittivity> {
        if let Some(timeout) = timeout {
            self.delay_value(self.permittivity, timeout).await
        } else {
            Ok(self.permittivity)
        }
    }

    /// Implementation of Capabilities::read_raw_counts()
    pub async fn read_raw_counts(&self, timeout: Option<Duration>) -> Result<RawCounts> {
        if let Some(timeout) = timeout {
            self.delay_value(self.raw_counts, timeout).await
        } else {
            Ok(self.raw_counts)
        }
    }
}

impl Default for Proxy {
    fn default() -> Self {
        Self {
            temperature: Self::default_temperature(),
            water_content: Self::default_water_content(),
            permittivity: Self::default_permittivity(),
            raw_counts: Self::default_raw_counts(),
            delay: Duration::default(),
            next_error: Cell::new(None),
        }
    }
}

impl Driver for Proxy {
    fn set_delay(&mut self, delay: Duration) {
        self.delay = delay;
    }

    fn set_next_error(&mut self, next_error: Option<Error>) {
        self.next_error.set(next_error);
    }

    fn set_temperature(&mut self, temperature: Temperature) {
        self.temperature = temperature;
    }

    fn set_water_content(&mut self, water_content: VolumetricWaterContent) {
        self.water_content = water_content;
    }

    fn set_permittivity(&mut self, permittivity: RelativePermittivity) {
        self.permittivity = permittivity;
    }

    fn set_raw_counts(&mut self, raw_counts: RawCounts) {
        self.raw_counts = raw_counts;
    }
}

#[async_trait(?Send)]
impl crate::Capabilities for Proxy {
    async fn read_temperature(&self, timeout: Option<Duration>) -> Result<Temperature> {
        self.read_temperature(timeout).await
    }

    async fn read_water_content(
        &self,
        timeout: Option<Duration>,
    ) -> Result<VolumetricWaterContent> {
        self.read_water_content(timeout).await
    }

    async fn read_permittivity(&self, timeout: Option<Duration>) -> Result<RelativePermittivity> {
        self.read_permittivity(timeout).await
    }

    async fn read_raw_counts(&self, timeout: Option<Duration>) -> Result<RawCounts> {
        self.read_raw_counts(timeout).await
    }
}
