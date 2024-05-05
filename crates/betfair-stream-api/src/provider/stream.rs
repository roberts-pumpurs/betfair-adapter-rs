mod raw_stream_connection;
mod stream_listener;

pub use stream_listener::*;

#[trait_variant::make(Send)]
pub trait BetfairProviderExt {
    async fn connect_to_stream(&self) -> StreamApiBuilder;

    async fn connect_to_stream_with_hb(&self, hb: HeartbeatStrategy) -> StreamApiBuilder;
}

impl BetfairProviderExt for betfair_adapter::AuthenticatedBetfairRpcProvider {
    async fn connect_to_stream(&self) -> StreamApiBuilder {
        self.connect_to_stream_with_hb(HeartbeatStrategy::None)
            .await
    }

    async fn connect_to_stream_with_hb(&self, hb: HeartbeatStrategy) -> StreamApiBuilder {
        let base = self.base();
        let application_key = base.secret_provider.application_key.clone();
        let url = base.stream.clone();

        let session_token = self.session_token().clone();
        StreamApiBuilder::new(application_key, session_token, url, hb)
    }
}
