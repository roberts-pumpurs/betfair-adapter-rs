# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-rpc-server-mock-v0.4.0...betfair-rpc-server-mock-v0.4.1) - 2025-04-21

### Other

- typos fix
- fix keep alive test

## [0.4.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-rpc-server-mock-v0.3.0...betfair-rpc-server-mock-v0.4.0) - 2025-04-20

### Added

- rpc agent will take care of keep-alive loop

## [0.3.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-rpc-server-mock-v0.2.1...betfair-rpc-server-mock-v0.3.0) - 2025-04-20

### Added

- stream api rewrite ([#31](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/31))

## [0.2.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-rpc-server-mock-v0.1.2...betfair-rpc-server-mock-v0.2.0) - 2025-04-06

### Other

- update Cargo.toml dependencies

## [0.1.1](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-rpc-server-mock-v0.1.0...betfair-rpc-server-mock-v0.1.1) - 2024-12-24

### Added

- refactored stream api
- can instantiate stream api by just importing a trait
- rpc server mock separated out

### Fixed

- fix resolve eyre::Result handling in rust type parsing & warnings fixes ([#6](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/6))

### Other

- add description to all packages
- update cargo toml for all crates
- use arcs for wrapping strings (temp)
- remove unnecessary `async` keywords
- fix broken test
- Update README.md ([#5](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/5))
- new ci
- add utilities to model error responses
- remove circular dependencies
- rpc mock will validate login credentials
- applying clippy lints
- fixing clippy lints
- remove unused dependencies
- fix broken test
- added utilities for wiring up mocks
- added utilities for wiring up mocks
- added utilities for wiring up mocks
- readme update
- cleanup
- update readme, docs and templates
- root for custom Betfair adapter