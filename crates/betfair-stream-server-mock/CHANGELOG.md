# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-stream-server-mock-v0.1.1...betfair-stream-server-mock-v0.1.2) - 2024-12-24

### Other

- clean up stream server mock

## [0.1.1](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-stream-server-mock-v0.1.0...betfair-stream-server-mock-v0.1.1) - 2024-12-24

### Added

- stream api will not crash if certificate cell is initialized
- proper handling for tls connector
- refactored stream api
- created handicap type
- fix auto-reconnection
- wrapper/helper for subscribing to markets

### Fixed

- :bug: Fix for the default feature of integration-test (#9)
- fix resolve eyre::Result handling in rust type parsing & warnings fixes ([#6](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/6))
- misbehaving heartbeat strategy

### Other

- add description to all packages
- update cargo toml for all crates
- use arcs for wrapping strings (temp)
- Update README.md ([#5](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/5))
- new ci
- remove circular dependencies
- fixing clippy lints
- remove unused dependencies
- stream api folder structure
- cargo fmt & cargo clippy
- cargo make fmt
- fix broken tests
- readme update
- cleanup
- update readme, docs and templates
- root for custom Betfair adapter
