# TRUEBNER SMT100 Soil Moisture Sensor

Device drivers for the [TRUEBNER SMT100 Soil Moisture Sensor](http://www.truebner.de/smt100).

[![Build Status](https://travis-ci.org/slowtec/truebner-smt100.svg?branch=master)](https://travis-ci.org/slowtec/truebner-smt100)

Sensor values are accessible through the generic `Capabilities` trait independent of
the actual connection and protocol. Proxy objects provide concrete implementations of
this trait:

- Modbus RTU
- Mock (only for testing and simulation)

## Resources

- [SMT100 Product Page](http://www.truebner.de/en/smt100)
- [AN002: SMT100 Modbus Quickstart Guide](http://www.truebner.de/sites/default/files/AN002.pdf)
- [AN005: SMT100 ASCII Text Command Guide](http://www.truebner.de/sites/default/files/AN005.pdf)

## License

Copyright (c) 2018 - 2019, [slowtec GmbH](https://www.slowtec.de)

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `truebner-smt100` by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
