//! # Betfair Stream API Crate
//!
//! This crate provides functionality to interact with Betfair's streaming API,
//! offering tools for market and order data subscriptions, caching, and connection management.
//!
//! ## Features
//!
//! - Connect to Betfair's streaming API
//! - Market and order data subscription
//! - Caching mechanisms for market and order books
//! - Configurable heartbeat strategies
//! - TLS stream support
//! - Error handling specific to stream operations
//!
//! ## Main Components
//!
//! - `StreamApiBuilder`: For building and configuring stream connections
//! - `StreamApi`: The main interface for interacting with the Betfair stream
//! - `MarketSubscriber` and `OrderSubscriber`: For subscribing to market and order updates
//! - `MarketBookCache` and `OrderBookCache`: For caching market and order book data
//! - `BetfairProviderExt`: A trait extending Betfair provider functionality for stream connections
//!
//! ## Usage
//!
//! Users can connect to the Betfair stream using the `BetfairProviderExt` trait methods:
//!
//! ```rust
//! use betfair_stream::BetfairProviderExt;
//!
//! let provider = betfair_adapter::UnauthenticatedBetfairRpcProvider::new();
//! let stream_builder = provider.connect_to_stream();
//! // Configure and build the stream...
//! ```
//!
//! ## Error Handling
//!
//! The crate provides a custom `StreamError` type for handling stream-specific errors.
//!
//! ## Integration Testing
//!
//! When the `integration-test` feature is enabled, the crate provides functionality
//! for setting a custom certificate for mock TLS connections in testing scenarios.
mod cache;
mod connection;
mod error;
mod tls_sream;

extern crate alloc;

pub use betfair_stream_types as types;
pub use cache::market_subscriber::MarketSubscriber;
pub use cache::order_subscriber::OrderSubscriber;
pub use cache::primitives::{MarketBookCache, OrderBookCache};
pub use connection::builder::{HeartbeatStrategy, StreamApiBuilder};
pub use connection::{CacheEnabledMessages, ExternalUpdates, MetadataUpdates, StreamApi};
pub use error::StreamError;
pub use futures::StreamExt;

/// A trait for extending Betfair provider functionality for stream connections.
pub trait BetfairProviderExt {
    /// Connects to the stream using the default heartbeat strategy.
    ///
    /// # Returns
    /// A `StreamApiBuilder` instance configured for the stream connection.
    fn connect_to_stream(&self) -> StreamApiBuilder;

    /// Connects to the stream with a specified heartbeat strategy.
    ///
    /// # Parameters
    /// - `hb`: The heartbeat strategy to use for the connection.
    ///
    /// # Returns
    /// A `StreamApiBuilder` instance configured for the stream connection with the specified
    /// heartbeat strategy.
    fn connect_to_stream_with_hb(&self, hb: HeartbeatStrategy) -> StreamApiBuilder;
}

impl BetfairProviderExt for betfair_adapter::UnauthenticatedBetfairRpcProvider {
    fn connect_to_stream(&self) -> StreamApiBuilder {
        self.connect_to_stream_with_hb(HeartbeatStrategy::None)
    }

    fn connect_to_stream_with_hb(&self, hb: HeartbeatStrategy) -> StreamApiBuilder {
        StreamApiBuilder::new(self.clone(), hb)
    }
}

#[cfg(feature = "integration-test")]
/// allows the `betfair-stream-server-mock` to set a custom certificate to be used by the server and
/// client on a mock TLS connection
pub static CERTIFICATE: std::sync::OnceLock<String> = std::sync::OnceLock::<String>::new();
