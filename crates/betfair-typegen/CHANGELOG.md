# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-typegen-v0.6.8...betfair-typegen-v0.7.0) - 2026-02-14

### Fixed

- allow certain fields to be missing as witnessed in live data ([#108](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/108))

### Other

- remove NumericPrimitives as no longer needed ([#110](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/110))

## [0.6.8](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-typegen-v0.6.7...betfair-typegen-v0.6.8) - 2025-12-07

### Fixed

- size cancelled and matcheddate not mandatory ([#104](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/104))

## [0.6.5](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-typegen-v0.6.4...betfair-typegen-v0.6.5) - 2025-11-13

### Added

- accept strings such as Infinity when deserializing f64 ([#93](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/93))

## [0.6.4](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-typegen-v0.6.3...betfair-typegen-v0.6.4) - 2025-11-08

### Added

- drop null values when deserializing to HashMaps ([#91](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/91))

## [0.6.3](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-typegen-v0.6.2...betfair-typegen-v0.6.3) - 2025-11-03

### Other

- remove rust_decimal feature, only use floats ([#86](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/86))

## [0.6.2](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-typegen-v0.6.1...betfair-typegen-v0.6.2) - 2025-11-01

### Added

- allow using either f64 or Decimal depending on feature flag ([#83](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/83))

## [0.6.1](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-typegen-v0.6.0...betfair-typegen-v0.6.1) - 2025-10-24

### Fixed

- ignore bad doctests which should not run ([#78](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/78))

## [0.6.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-typegen-v0.5.2...betfair-typegen-v0.6.0) - 2025-09-20

### Other

- `cargo xtask fmt` updates and reduce warnings ([#63](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/63))

## [0.5.2](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-typegen-v0.5.1...betfair-typegen-v0.5.2) - 2025-09-18

### Added

- make SelectionId and other i64 newtypes Copy ([#61](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/61))

## [0.4.1](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-typegen-v0.4.0...betfair-typegen-v0.4.1) - 2025-04-21

### Other

- typos fix

## [0.3.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-typegen-v0.2.1...betfair-typegen-v0.3.0) - 2025-04-20

### Added

- stream api rewrite ([#31](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/31))

## [0.2.1](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-typegen-v0.2.0...betfair-typegen-v0.2.1) - 2025-04-06

### Added

- add extra missing error param to sportsaping

## [0.2.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-typegen-v0.1.2...betfair-typegen-v0.2.0) - 2025-04-06

### Fixed

- naming
- inconsistency in exec report error code
