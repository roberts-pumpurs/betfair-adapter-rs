mod cache;
mod connection;
mod error;
mod tls_sream;

pub use betfair_stream_types as types;
pub use cache::market_subscriber::MarketSubscriber;
pub use cache::order_subscriber::OrderSubscriber;
pub use connection::builder::{HeartbeatStrategy, StreamApiBuilder};
pub use connection::{CacheEnabledMessages, ExternalUpdates, MetadataUpdates, StreamApiConnection};
pub use error::StreamError;
pub use futures::StreamExt;

#[trait_variant::make(Send)]
pub trait BetfairProviderExt {
    async fn connect_to_stream(&self) -> StreamApiBuilder;

    async fn connect_to_stream_with_hb(&self, hb: HeartbeatStrategy) -> StreamApiBuilder;
}

impl BetfairProviderExt for betfair_adapter::UnauthenticatedBetfairRpcProvider {
    async fn connect_to_stream(&self) -> StreamApiBuilder {
        self.connect_to_stream_with_hb(HeartbeatStrategy::None)
            .await
    }

    async fn connect_to_stream_with_hb(&self, hb: HeartbeatStrategy) -> StreamApiBuilder {
        StreamApiBuilder::new(self.clone(), hb)
    }
}

#[cfg(feature = "integration-test")]
/// allows the `betfair-stream-server-mock` to set a custom certificate to be used by the server and
/// client on a mock TLS connection
pub static CERTIFICATE: std::sync::OnceLock<String> = std::sync::OnceLock::<String>::new();
