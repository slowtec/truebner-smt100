use super::*;

use futures::future;

pub struct Proxy {
    temperature: Temperature,
    water_content: VolumetricWaterContent,
    permittivity: RelativePermittivity,
    counts: usize,
}

impl Proxy {
    pub fn set_temperature(&mut self, temperature: Temperature) {
        self.temperature = temperature;
    }

    pub fn set_water_content(&mut self, water_content: VolumetricWaterContent) {
        self.water_content = water_content;
    }

    pub fn set_permittivity(&mut self, permittivity: RelativePermittivity) {
        self.permittivity = permittivity;
    }

    pub fn set_counts(&mut self, counts: usize) {
        self.counts = counts;
    }

    /// Implementation of Capabilities::read_temperature()
    pub fn read_temperature(&self) -> impl Future<Item = Temperature, Error = Error> {
        future::ok(self.temperature)
    }

    /// Implementation of Capabilities::read_water_content()
    pub fn read_water_content(&self) -> impl Future<Item = VolumetricWaterContent, Error = Error> {
        future::ok(self.water_content)
    }

    /// Implementation of Capabilities::read_permittivity()
    pub fn read_permittivity(&self) -> impl Future<Item = RelativePermittivity, Error = Error> {
        future::ok(self.permittivity)
    }

    /// Implementation of Capabilities::read_counts()
    pub fn read_counts(&self) -> impl Future<Item = usize, Error = Error> {
        future::ok(self.counts)
    }
}

impl Capabilities for Proxy {
    fn read_temperature(&self) -> Box<Future<Item = Temperature, Error = Error>> {
        Box::new(self.read_temperature())
    }

    fn read_water_content(&self) -> Box<Future<Item = VolumetricWaterContent, Error = Error>> {
        Box::new(self.read_water_content())
    }

    fn read_permittivity(&self) -> Box<Future<Item = RelativePermittivity, Error = Error>> {
        Box::new(self.read_permittivity())
    }

    fn read_counts(&self) -> Box<Future<Item = usize, Error = Error>> {
        Box::new(self.read_counts())
    }
}
