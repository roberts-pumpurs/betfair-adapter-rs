//! A Rust library for interacting with the Betfair API.

//! This crate provides modules for configuration, error handling, provider functionality,
//! secret management, and URL handling, along with types and utilities for making
//! authenticated requests to the Betfair API.  It aims to simplify the process of
//! integrating with Betfair's services, handling authentication, and making various
//! types of requests.

// Example usage (helps users understand how to get started)
//! ```
//! // Example of creating a configuration
//! use betfair_api::BetfairConfigBuilder;
//!
//! let config = BetfairConfigBuilder::new()
//!     .app_key("YOUR_APP_KEY")
//!     .username("YOUR_USERNAME")
//!     .password("YOUR_PASSWORD")
//!     .build()
//!     .unwrap();
//! ```

#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

/// Module for configuration
mod config;

/// Module for error handling.
mod error;

/// Module for provider-related functionality.
mod provider;

/// Module for secret management.
mod secret;

/// Module for URL handling.
mod urls;

/// Main types used in the Betfair API.
pub use betfair_types;
/// Decimal type used in financial calculations.
pub use betfair_types::rust_decimal;
/// Builder for Betfair configuration.
pub use config::BetfairConfigBuilder;
/// Error type for API interactions.
pub use error::ApiError;
/// Struct for authenticated Betfair requests.
pub use provider::authenticated::{BetfairRequest, BetfairResponse};
/// Provider for authenticated Betfair RPC interactions.
pub use provider::{AuthenticatedBetfairRpcProvider, UnauthenticatedBetfairRpcProvider};
/// Struct for managing application keys.
pub use secret::{ApplicationKey, Identity, Password, SecretProvider, SessionToken, Username};
/// URL utilities for Betfair API interactions.
pub use urls::{
    jurisdiction, BetfairUrl, BotLogin, InteractiveLogin, KeepAlive, Logout, RestBase, RetrieveUrl,
    Stream,
};
