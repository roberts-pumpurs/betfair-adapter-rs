mod raw_stream_connection;
mod stream_listener;

use std::sync::Arc;

use futures::Future;
pub use stream_listener::*;

use crate::StreamError;

#[trait_variant::make(Send)]
pub trait BetfairProviderExt {
    async fn connect_to_stream(
        &self,
    ) -> Result<
        (
            Arc<tokio::sync::RwLock<StreamListener>>,
            impl Future<Output = ()> + Send + 'static,
            tokio::sync::broadcast::Receiver<ExternalUpdates>,
        ),
        StreamError,
    >;

    async fn connect_to_stream_with_hb(
        &self,
        hb: HeartbeatStrategy,
    ) -> Result<
        (
            Arc<tokio::sync::RwLock<StreamListener>>,
            impl Future<Output = ()> + Send + 'static,
            tokio::sync::broadcast::Receiver<ExternalUpdates>,
        ),
        StreamError,
    >;
}

impl BetfairProviderExt for betfair_adapter::AuthenticatedBetfairRpcProvider {
    async fn connect_to_stream(
        &self,
    ) -> Result<
        (
            Arc<tokio::sync::RwLock<StreamListener>>,
            impl Future<Output = ()> + Send + 'static,
            tokio::sync::broadcast::Receiver<ExternalUpdates>,
        ),
        StreamError,
    > {
        self.connect_to_stream_with_hb(HeartbeatStrategy::None)
            .await
    }

    async fn connect_to_stream_with_hb(
        &self,
        hb: HeartbeatStrategy,
    ) -> Result<
        (
            Arc<tokio::sync::RwLock<StreamListener>>,
            impl Future<Output = ()> + Send + 'static,
            tokio::sync::broadcast::Receiver<ExternalUpdates>,
        ),
        StreamError,
    > {
        let base = self.base();
        let application_key = base.secret_provider.application_key.clone();
        let url = base.stream.clone();

        let session_token = self.session_token().clone();
        StreamListener::new(application_key, session_token, url, hb).await
    }
}
