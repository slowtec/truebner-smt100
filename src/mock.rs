use super::*;

use futures::future;
use std::cell::Cell;
use std::io::{Error, ErrorKind};
use std::time::{Duration, Instant};

use tokio::prelude::*;
use tokio::timer::Delay;

pub struct Proxy {
    temperature: Temperature,
    water_content: VolumetricWaterContent,
    permittivity: RelativePermittivity,
    raw_counts: usize,
    delay: Duration,
    next_error: Cell<Option<Error>>,
}

impl Proxy {
    pub fn set_delay(&mut self, delay: Duration) {
        self.delay = delay;
    }

    pub fn set_next_error(&mut self, next_error: Option<Error>) {
        self.next_error.set(next_error);
    }

    pub fn default_temperature() -> Temperature {
        Temperature::from_degree_celsius(20.0)
    }

    pub fn set_temperature(&mut self, temperature: Temperature) {
        self.temperature = temperature;
    }

    pub fn default_water_content() -> VolumetricWaterContent {
        VolumetricWaterContent::from_percent(30.0)
    }

    pub fn set_water_content(&mut self, water_content: VolumetricWaterContent) {
        self.water_content = water_content;
    }

    pub fn default_permittivity() -> RelativePermittivity {
        RelativePermittivity::min()
    }

    pub fn set_permittivity(&mut self, permittivity: RelativePermittivity) {
        self.permittivity = permittivity;
    }

    pub const fn default_raw_counts() -> usize {
        0
    }

    pub fn set_raw_counts(&mut self, raw_counts: usize) {
        self.raw_counts = raw_counts;
    }

    fn read_value<T>(&self, value: T, timeout: Duration) -> Box<Future<Item = T, Error = Error>>
    where
        T: 'static,
    {
        let deadline = Instant::now() + self.delay;
        let next_error = self.next_error.replace(None);
        if let Some(error) = next_error {
            Box::new(
                Delay::new(deadline)
                    .then(move |_| future::err(error))
                    .map_err(|err| {
                        Error::new(ErrorKind::Other, format!("reading value failed: {}", err))
                    })
                    .timeout(timeout)
                    .map_err(|err| {
                        Error::new(
                            ErrorKind::TimedOut,
                            format!("reading value timed out: {}", err),
                        )
                    }),
            )
        } else {
            Box::new(
                Delay::new(deadline)
                    .then(move |_| future::ok(value))
                    .map_err(|()| {
                        Error::new(
                            ErrorKind::Other,
                            format!("reading value failed unexpectedly"),
                        )
                    })
                    .timeout(timeout)
                    .map_err(|err| {
                        Error::new(
                            ErrorKind::TimedOut,
                            format!("reading value timed out: {}", err),
                        )
                    }),
            )
        }
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
