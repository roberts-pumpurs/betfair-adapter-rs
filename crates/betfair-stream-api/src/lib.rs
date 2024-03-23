#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

mod error;
mod provider;
mod specialization;
pub use betfair_stream_types as types;
pub use error::StreamError;
pub use provider::{ExternalUpdates, HeartbeatStrategy, Status, StreamListener};
pub use specialization::MarketSubscriber;
