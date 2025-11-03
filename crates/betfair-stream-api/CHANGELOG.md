# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.3](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-stream-api-v0.6.2...betfair-stream-api-v0.6.3) - 2025-11-03

### Other

- remove rust_decimal feature, only use floats ([#86](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/86))

## [0.6.2](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-stream-api-v0.6.1...betfair-stream-api-v0.6.2) - 2025-11-01

### Added

- allow using either f64 or Decimal depending on feature flag ([#83](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/83))

## [0.6.1](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-stream-api-v0.6.0...betfair-stream-api-v0.6.1) - 2025-10-24

### Added

- expose stream trackers and states ([#79](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/79))

### Fixed

- expose last change in OrderBookCache and update strategy_matches correctly ([#81](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/81))

## [0.6.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-stream-api-v0.5.2...betfair-stream-api-v0.6.0) - 2025-09-20

### Added

- make `Price`, `Handicap` and `Position` all Copy ([#64](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/64))

### Other

- `cargo xtask fmt` updates and reduce warnings ([#63](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/63))

## [0.5.1](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-stream-api-v0.5.0...betfair-stream-api-v0.5.1) - 2025-09-18

### Added

- getter for total_matched in MarketBookCache ([#59](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/59))

### Other

- update hb params for stream

## [0.5.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-stream-api-v0.4.1...betfair-stream-api-v0.5.0) - 2025-04-25

### Added

- the spawn function will be generic

## [0.4.1](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-stream-api-v0.4.0...betfair-stream-api-v0.4.1) - 2025-04-21

### Added

- add extra derive macros

### Other

- typos fix

## [0.3.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-stream-api-v0.2.1...betfair-stream-api-v0.3.0) - 2025-04-20

### Added

- stream api rewrite ([#31](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/31))
