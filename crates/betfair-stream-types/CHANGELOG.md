# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/roberts-pumpurs/betfair-adapter-rs/compare/betfair-stream-types-v0.1.0...betfair-stream-types-v0.1.1) - 2024-12-24

### Added

- created handicap type
- wrapper/helper for subscribing to markets
- cache for stream listener
- init order book cache
- stream market book cache
- custom types for some specific betfair types
- initial stream api processor
- streaming API types

### Fixed

- fix resolve eyre::Result handling in rust type parsing & warnings fixes ([#6](https://github.com/roberts-pumpurs/betfair-adapter-rs/pull/6))
- :construction: Fix linting warnings in the codebase (#4)

### Other

- use local readme
- add description to all packages
- update cargo toml for all crates
- use arcs for wrapping strings (temp)
- cargo xtask fmt
- new ci
- cargo xtask fmt
- apply clippy suggestions
- applying clippy lints
- fixing clippy lints
- fix typos
- remove unused dependencies
- cleaned up cache loop
- stream api folder structure
- order change message data field renaming
- order change message data field renaming
- order place date will be a proper date time object
- type renaming
- tests for market book cache
- wip subscribe
- wip
- rearrange the stream types
- rpc adapter cleanup
- cleanup stream API data types
