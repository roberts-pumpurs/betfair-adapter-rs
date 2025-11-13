# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.5](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-types-v0.6.4...betfair-types-v0.6.5) - 2025-11-13

### Added

- accept strings such as Infinity when deserializing f64 ([#93](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/93))

## [0.6.4](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-types-v0.6.3...betfair-types-v0.6.4) - 2025-11-08

### Added

- drop null values when deserializing to HashMaps ([#91](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/91))

### Fixed

- use bitwise equality for F64Ord ([#90](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/90))

## [0.6.3](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-types-v0.6.2...betfair-types-v0.6.3) - 2025-11-03

### Other

- remove rust_decimal feature, only use floats ([#86](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/86))

## [0.6.2](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-types-v0.6.1...betfair-types-v0.6.2) - 2025-11-01

### Added

- allow using either f64 or Decimal depending on feature flag ([#83](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/83))

### Other

- dep updates ([#85](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/85))

## [0.6.1](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-types-v0.6.0...betfair-types-v0.6.1) - 2025-10-24

### Added

- implement Display for CustomerStrategyRef ([#77](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/77))

### Fixed

- stream deserialization issues and expose them to tests ([#80](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/80))

## [0.6.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-types-v0.5.2...betfair-types-v0.6.0) - 2025-09-20

### Added

- make `Price`, `Handicap` and `Position` all Copy ([#64](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/64))

### Other

- `cargo xtask fmt` updates and reduce warnings ([#63](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/63))

## [0.5.2](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-types-v0.5.1...betfair-types-v0.5.2) - 2025-09-18

### Fixed

- allow price of 1000 ([#60](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/60))

## [0.3.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-types-v0.2.1...betfair-types-v0.3.0) - 2025-04-20

### Added

- stream api rewrite ([#31](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/31))

## [0.2.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-types-v0.1.2...betfair-types-v0.2.0) - 2025-04-06

### Other

- remove unused deps & switch to stable

## [0.1.2](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-types-v0.1.1...betfair-types-v0.1.2) - 2024-12-24

### Other

- remove unnecessary lints

## [0.1.1](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-types-v0.1.0...betfair-types-v0.1.1) - 2024-12-24

### Other

- use local readme
