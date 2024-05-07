#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

mod error;
mod provider;
mod specialization;
pub use error::StreamError;
pub use provider::{
    cache, BetfairData, BetfairProviderExt, CacheEnabledMessages, ExternalUpdates,
    HeartbeatStrategy, MetadataUpdates, StreamApiBuilder, StreamApiConnection,
};
pub use specialization::{MarketSubscriber, OrderSubscriber};
pub use {betfair_adapter, betfair_stream_types as types, futures};
