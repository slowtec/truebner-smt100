# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Removed

## [0.2.0] - 2019-04-dd

### Added

- Added functions for interacting directly with a Modbus client context
  without the need for a `SlaveProxy`

### Changed

- Upgrade to tokio-modbus 0.3.2
- Use a shared, reconnectable Modbus RTU environment for multiple devices
- Breaking change: Switching from an owned to a shared Modbus connection
  requires to provide `tokio_modbus::client::util::SharedEnvironment` when
  creating a `SlaveProxy`!

### Removed

## [0.1.0] - 2019-04-08

### Added

- Initial public release

[Unreleased]: https://github.com/slowtec/truebner-smt100/compare/v0.2.0...master
[0.2.0]: https://github.com/slowtec/truebner-smt100/releases/v0.2.0
[0.1.0]: https://github.com/slowtec/truebner-smt100/releases/v0.1.0
