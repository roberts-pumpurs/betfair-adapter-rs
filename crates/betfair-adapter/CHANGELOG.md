# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-adapter-v0.4.0...betfair-adapter-v0.4.1) - 2025-04-21

### Other

- typos fix

## [0.4.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-adapter-v0.3.0...betfair-adapter-v0.4.0) - 2025-04-20

### Added

- rpc agent will take care of keep-alive loop

## [0.3.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-adapter-v0.2.1...betfair-adapter-v0.3.0) - 2025-04-20

### Added

- stream api rewrite ([#31](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/31))

## [0.2.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-adapter-v0.1.2...betfair-adapter-v0.2.0) - 2025-04-06

### Other

- update Cargo.toml dependencies

## [0.1.1](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-adapter-v0.1.0...betfair-adapter-v0.1.1) - 2024-12-24

### Added

- betfair requests can be made concurrently via request objects
- :art: Added xtask subscribe to market command (#7)
- example on how to connect to betfair
- refactored stream api
- can instantiate stream api by just importing a trait
- rpc server mock separated out
- created handicap type
- wrapper/helper for subscribing to markets
- stream API can do a handshake
- stream api can connect
- custom types for some specific betfair types
- stream api tls and non-tls connections
- initial stream api processor

### Fixed

- re-export internal request types
- fix resolve eyre::Result handling in rust type parsing & warnings fixes ([#6](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/6))
- misbehaving heartbeat strategy

### Other

- add description to all packages
- update cargo toml for all crates
- use arcs for wrapping strings (temp)
- remove unnecessary `async` keywords
- `build_request` is no longer async
- cargo xtask fmt
- Update README.md ([#5](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/5))
- new ci
- identity parsing tests
- remove circular dependencies
- cargo xtask fmt
- betfair urls have default jurisdiction
- applying clippy lints
- fixing clippy lints
- remove unused dependencies
- improved error handling and added sesseion token refresh logic
- cargo fmt & cargo clippy
- added utilities for wiring up mocks
- added utilities for wiring up mocks
- fix broken tests
- rename "rpc adapter" to just "adapter"
- readme update
- cleanup
- update readme, docs and templates
- root for custom Betfair adapter
