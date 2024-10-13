//! # Betfair API Types Crate
//!
//! This crate defines the request and response types for interacting with the Betfair API.
//! It provides a strongly-typed interface for constructing API requests and parsing API responses.
//!
//! ## Features
//!
//! - Comprehensive type definitions for Betfair API requests and responses
//! - Strict compiler warnings and denials to ensure code quality
//! - Separate modules for request and response types
//!
//! ## Modules
//!
//! - [`request`]: Contains types and structures for constructing API requests
//! - [`response`]: Contains types and structures for parsing API responses
//!
//! ## Usage
//!
//! Import the necessary types from the appropriate modules:
//!
//! ```rust
//! use betfair_api_types::request::SomeRequestType;
//! use betfair_api_types::response::SomeResponseType;
//! ```
//!
//! ## Compiler Attributes
//!
//! This crate uses several compiler attributes to enforce code quality:
//!
//! - Warnings for missing documentation, unreachable public items, and unused crate dependencies
//! - Denials for unused `must_use` results and non-idiomatic Rust 2018 code
//! - Test attributes to ensure high-quality documentation examples
//!
//! ## Documentation Tests
//!
//! The documentation tests in this crate are configured to:
//!
//! - Not inject the crate into the test's scope
//! - Deny warnings and non-idiomatic Rust 2018 code
//! - Allow dead code and unused variables (useful for concise examples)
//!
//! These settings help ensure that the documentation remains accurate and up-to-date.

#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

/// Module for request types.
pub mod request;
/// Module for response types.
pub mod response;
