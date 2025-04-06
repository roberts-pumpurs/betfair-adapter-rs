# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-stream-api-v0.1.2...betfair-stream-api-v0.2.0) - 2025-04-06

### Other

- remove unused deps & switch to stable

## [0.1.2](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-stream-api-v0.1.1...betfair-stream-api-v0.1.2) - 2024-12-24

### Other

- clean up stream server mock

## [0.1.1](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-stream-api-v0.1.0...betfair-stream-api-v0.1.1) - 2024-12-24

### Added

- can build local cache from data
- expose cache primitives
- getter for runners and market_id in OrderBookCache
- using `backoff` crate for retrying the connection to the stream
- utilities for generating certificates for Betfair
- failure to authenticate will trigger a metadata update
- proper handling for tls connector
- example on how to connnect to betfair
- refactored stream api
- can instantiate stream api by just importing a trait
- rpc server mock separated out
- order subscriber specialization
- re-expose internal cache types
- created handicap type
- fix auto-reconnection
- wrapper/helper for subscribing to markets
- cache for stream listener
- init order book cache
- stream market book cache
- available cache structure
- stream API can do a handshake
- stream api can connect
- stream api tls and non-tls connections
- initial stream api processor
- new empty crate for betfair stream api

### Fixed

- fix resolve eyre::Result handling in rust type parsing & warnings fixes ([#6](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/6))
- misbehaving heartbeat strategy

### Other

- add description to all packages
- update cargo toml for all crates
- use arcs for wrapping strings (temp)
- expose the available book data
- minor cleanup
- bump deps
- cargo xtask fmt
- Update README.md ([#5](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/5))
- typo fix
- new ci
- remove circular dependencies
- cargo xtask fmt
- orderbok cache refactor
- using pin-project crate
- remove unnecessary async
- apply clippy suggestions
- applying clippy lints
- fixing clippy lints
- fix typos
- remove unused dependencies
- cargo xtask fmt
- remove redundant broadcast channel
- add tests for stream api connection
- cleaned up cache loop
- improved error handling and added sesseion token refresh logic
- handshake handled by a custom stream impl
- stream api folder structure
- cargo fmt & cargo clippy
- tls stream will no longer be multiplexed
- added utilities for wiring up mocks
- exposing order cache
- exposing order cache
- init background reconnection structure
- fix broken tests
- type renaming
- simplified `raw_stream` code
- tests for market book cache
- wip subscribe
- cargo make fmt
- fix broken tests
- wip
- readme update
- cleanup
- update readme, docs and templates
- root for custom Betfair adapter
