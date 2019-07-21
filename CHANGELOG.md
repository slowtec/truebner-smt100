# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added `[no_std]` core module with optional features `modbus` and `rtu`
- Added various `[no_std]` low-level *Modbus* decoding functions
- Added a `[no_std]` blocking `Capabilities` trait

### Changed

- Renamed feature `modbus-rtu` as `tokio-modbus-rtu`
- Renamed feature `mock` as `tokio-mock`
- Read timeout on the non-blocking `Capabilities` trait has become optional

### Removed

- Removed dependency on the `byteorder` crate
- Removed all *Newtypes* for `...Raw` measurements

## [0.2.1] - 2019-05-21

### Changed

- Disabled default features of *tokio-serial* dependency to avoid unused dependency
  on *libudev*

## [0.2.0] - 2019-04-15

### Added

- Added functions for interacting directly with a Modbus client context
  without the need for a `SlaveProxy`

### Changed

- Upgrade to tokio-modbus 0.3.2
- Use a shared, reconnectable Modbus RTU environment for multiple devices
- Breaking change: Switching from an owned to a shared Modbus connection
  requires to provide `tokio_modbus::client::util::SharedContext` when
  creating a `SlaveProxy`!

### Removed

## [0.1.0] - 2019-04-08

### Added

- Initial public release

[Unreleased]: https://github.com/slowtec/truebner-smt100/compare/v0.2.1...master
[0.2.1]: https://github.com/slowtec/truebner-smt100/releases/v0.2.1
[0.2.0]: https://github.com/slowtec/truebner-smt100/releases/v0.2.0
[0.1.0]: https://github.com/slowtec/truebner-smt100/releases/v0.1.0
