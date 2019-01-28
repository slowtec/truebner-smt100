use super::*;

use std::cell::Cell;
use std::io::{Error, ErrorKind};
use std::time::{Duration, Instant};

use tokio::timer::Delay;
use tokio::util::FutureExt;

pub struct Proxy {
    temperature: Temperature,
    water_content: VolumetricWaterContent,
    permittivity: RelativePermittivity,
    raw_counts: usize,
    delay: Duration,
    next_error: Cell<Option<Error>>,
}

pub trait Driver {
    fn set_delay(&mut self, delay: Duration);

    fn set_next_error(&mut self, next_error: Option<Error>);

    fn set_temperature(&mut self, temperature: Temperature);

    fn set_water_content(&mut self, water_content: VolumetricWaterContent);

    fn set_permittivity(&mut self, permittivity: RelativePermittivity);

    fn set_raw_counts(&mut self, raw_counts: usize);
}

impl Proxy {
    pub fn default_temperature() -> Temperature {
        Temperature::from(20.0)
    }

    pub fn default_water_content() -> VolumetricWaterContent {
        VolumetricWaterContent::from_percent(30.0)
    }

    pub fn default_permittivity() -> RelativePermittivity {
        RelativePermittivity::min()
    }

    pub const fn default_raw_counts() -> usize {
        0
    }

    fn read_value<T>(&self, value: T, timeout: Duration) -> impl Future<Item = T, Error = Error>
    where
        T: 'static,
    {
        let deadline = Instant::now() + self.delay;
        let next_error = self.next_error.replace(None);
        let result = if let Some(error) = next_error {
            Err(error)
        } else {
            Ok(value)
        };
        Delay::new(deadline)
            .then(move |_| result)
            .map_err(|err| Error::new(ErrorKind::Other, format!("reading value failed: {}", err)))
            .timeout(timeout)
            .map_err(move |err| {
                err.into_inner().unwrap_or_else(|| {
                    Error::new(ErrorKind::TimedOut, String::from("reading value timed out"))
                })
            })
    }

    /// Implementation of Capabilities::read_temperature()
    pub fn read_temperature(
        &self,
        timeout: Duration,
    ) -> impl Future<Item = Temperature, Error = Error> {
        self.read_value(self.temperature, timeout)
    }

    /// Implementation of Capabilities::read_water_content()
    pub fn read_water_content(
        &self,
        timeout: Duration,
    ) -> impl Future<Item = VolumetricWaterContent, Error = Error> {
        self.read_value(self.water_content, timeout)
    }

    /// Implementation of Capabilities::read_permittivity()
    pub fn read_permittivity(
        &self,
        timeout: Duration,
    ) -> impl Future<Item = RelativePermittivity, Error = Error> {
        self.read_value(self.permittivity, timeout)
    }

    /// Implementation of Capabilities::read_raw_counts()
    pub fn read_raw_counts(&self, timeout: Duration) -> impl Future<Item = usize, Error = Error> {
        self.read_value(self.raw_counts, timeout)
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

    fn set_raw_counts(&mut self, raw_counts: usize) {
        self.raw_counts = raw_counts;
    }
}

impl Capabilities for Proxy {
    fn read_temperature(
        &self,
        timeout: Duration,
    ) -> Box<Future<Item = Temperature, Error = Error>> {
        Box::new(self.read_temperature(timeout))
    }

    fn read_water_content(
        &self,
        timeout: Duration,
    ) -> Box<Future<Item = VolumetricWaterContent, Error = Error>> {
        Box::new(self.read_water_content(timeout))
    }

    fn read_permittivity(
        &self,
        timeout: Duration,
    ) -> Box<Future<Item = RelativePermittivity, Error = Error>> {
        Box::new(self.read_permittivity(timeout))
    }

    fn read_raw_counts(&self, timeout: Duration) -> Box<Future<Item = usize, Error = Error>> {
        Box::new(self.read_raw_counts(timeout))
    }
}
