/// Module for managing the connection to the Betfair streaming API.
pub(crate) mod builder;
pub(crate) mod cron;
pub(crate) mod handshake;

use core::convert::Infallible as Never;
use core::task::Poll;

use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::response::connection_message::ConnectionMessage;
use betfair_stream_types::response::status_message::StatusMessage;
use betfair_stream_types::response::ResponseMessage;
use futures::Stream;
use tokio::task::JoinSet;

use self::builder::wrap_with_cache_layer;
use self::cron::FatalError;
use crate::cache::primitives::{MarketBookCache, OrderBookCache};

/// Represents the streaming API connection.
#[derive(Debug)]
#[pin_project::pin_project]
pub struct StreamApi<T> {
    join_set: JoinSet<Result<Never, FatalError>>,
    rt_handle: tokio::runtime::Handle,
    is_shutting_down: bool,
    data_feed: tokio::sync::mpsc::Receiver<ExternalUpdates<T>>,
    command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
}

impl<T> StreamApi<T> {
    /// Creates a new instance of `StreamApi`.
    pub(crate) const fn new(
        join_set: JoinSet<Result<Never, FatalError>>,
        data_feed: tokio::sync::mpsc::Receiver<ExternalUpdates<T>>,
        command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
        rt_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            is_shutting_down: false,
            join_set,
            rt_handle,
            data_feed,
            command_sender,
        }
    }

    /// Returns a reference to the command sender.
    #[must_use]
    pub const fn command_sender(&self) -> &tokio::sync::broadcast::Sender<RequestMessage> {
        &self.command_sender
    }
}

impl StreamApi<ResponseMessage> {
    /// Enables caching for the stream API.
    #[must_use]
    pub fn enable_cache(mut self) -> StreamApi<CacheEnabledMessages> {
        let output_queue_reader_post_cache =
            wrap_with_cache_layer(&mut self.join_set, self.data_feed, &self.rt_handle);
        StreamApi {
            join_set: self.join_set,
            rt_handle: self.rt_handle,
            is_shutting_down: self.is_shutting_down,
            data_feed: output_queue_reader_post_cache,
            command_sender: self.command_sender,
        }
    }
}

/// Represents external updates received from the data feed.
#[derive(Debug, Clone)]
pub enum ExternalUpdates<T> {
    /// Represents a layer of data.
    Layer(T),
    /// Represents metadata updates.
    Metadata(MetadataUpdates),
}

/// Represents messages that have caching enabled.
#[derive(Debug, Clone)]
pub enum CacheEnabledMessages {
    /// Represents market changes.
    MarketChange(Vec<MarketBookCache>),
    /// Represents order changes.
    OrderChange(Vec<OrderBookCache>),
    /// Represents connection messages.
    Connection(ConnectionMessage),
    /// Represents status messages.
    Status(StatusMessage),
}

/// Represents various metadata updates related to the connection state.
#[derive(Debug, Clone)]
pub enum MetadataUpdates {
    /// Indicates disconnection.
    Disconnected,
    /// Indicates TCP connection established.
    TcpConnected,
    /// Indicates failure to connect.
    FailedToConnect,
    /// Indicates authentication message sent.
    AuthenticationMessageSent,
    /// Indicates successful authentication.
    Authenticated {
        /// Number of available connections.
        connections_available: i32,
        /// Optional connection ID.
        connection_id: Option<String>,
    },
    /// Indicates failure to authenticate.
    FailedToAuthenticate,
}

impl<T> Stream for StreamApi<T> {
    type Item = ExternalUpdates<T>;

    fn poll_next(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // only return None if we are shutting down and there are no tasks left
        if self.join_set.is_empty() && self.is_shutting_down {
            tracing::warn!("StreamApiConnection: No tasks remaining, shutting down.");
            return Poll::Ready(None);
        }

        // Poll the join set to check if any child tasks have completed
        match self.join_set.poll_join_next(cx) {
            Poll::Ready(Some(Ok(Err(err)))) => {
                tracing::error!(?err, "Error returned by a task");
                self.join_set.abort_all();
                self.is_shutting_down = true;
                cx.waker().wake_by_ref();
            }
            Poll::Ready(Some(Ok(Ok(_e)))) => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            Poll::Ready(Some(Err(err))) => {
                tracing::error!(?err, "Error in join_set");
                self.join_set.abort_all();
                self.is_shutting_down = true;
                cx.waker().wake_by_ref();
            }
            Poll::Ready(None) => {
                // All tasks have completed; commence shutdown
                self.is_shutting_down = true;
            }
            Poll::Pending => {}
        }

        // Poll the data feed for new items
        match self.data_feed.poll_recv(cx) {
            Poll::Ready(Some(update)) => Poll::Ready(Some(update)),
            Poll::Ready(None) => {
                // No more data, initiate shutdown
                tracing::warn!("StreamApiConnection: Data feed closed.");
                self.join_set.abort_all();
                self.is_shutting_down = true;
                cx.waker().wake_by_ref();
                Poll::Ready(None)
            }
            Poll::Pending if self.is_shutting_down => {
                // If shutting down and no data available, end the stream
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use futures::stream::StreamExt;
    use tokio::sync::{broadcast, mpsc};
    use tokio::task::JoinSet;
    use tokio::time::timeout;

    use super::*;

    #[tokio::test]
    async fn stream_api_connection_poll_next_shuts_down_on_empty_join_set_when_shutting_down() {
        let (_data_sender, data_receiver) = mpsc::channel(10);
        let (command_sender, _) = broadcast::channel(10);
        let join_set = JoinSet::new();
        let handle = tokio::runtime::Handle::current();

        let mut connection = StreamApi::<()>::new(join_set, data_receiver, command_sender, handle);
        connection.is_shutting_down = true;

        assert!(
            connection.next().await.is_none(),
            "Stream should shut down immediately when no tasks are left"
        );
    }

    #[tokio::test]
    async fn stream_api_connection_poll_next_shuts_down_on_empty_join_set() {
        let (_data_sender, data_receiver) = mpsc::channel(10);
        let (command_sender, _) = broadcast::channel(10);
        let join_set = JoinSet::new();
        let handle = tokio::runtime::Handle::current();

        let mut connection = StreamApi::<()>::new(join_set, data_receiver, command_sender, handle);

        assert!(
            connection.next().await.is_none(),
            "Stream should shut down immediately when no tasks are left"
        );
    }

    #[tokio::test]
    async fn stream_api_connection_receives_updates() {
        let (data_sender, data_receiver) = mpsc::channel(10);
        let (command_sender, _) = broadcast::channel(10);
        let mut join_set = JoinSet::new();
        join_set.spawn(futures::future::pending());
        let handle = tokio::runtime::Handle::current();

        let mut connection = StreamApi::new(join_set, data_receiver, command_sender, handle);

        let expected_update = ExternalUpdates::Layer("Test".to_owned());
        data_sender.send(expected_update.clone()).await.unwrap();

        match connection.next().await {
            Some(update) => match update {
                ExternalUpdates::Layer(content) => assert_eq!(content, "Test"),
                _ => panic!("Unexpected update type"),
            },
            _ => panic!("Expected to receive an update"),
        }

        assert!(
            timeout(Duration::from_millis(100), connection.next())
                .await
                .is_err(),
            "Stream should remain pending after receiving an update"
        );
    }

    #[tokio::test]
    async fn stream_api_connection_receives_updates_then_closes_empty_join_set() {
        let (data_sender, data_receiver) = mpsc::channel(10);
        let (command_sender, _) = broadcast::channel(10);
        let join_set = JoinSet::new();
        let handle = tokio::runtime::Handle::current();

        let mut connection = StreamApi::new(join_set, data_receiver, command_sender, handle);

        let expected_update = ExternalUpdates::Layer("Test".to_owned());
        data_sender.send(expected_update.clone()).await.unwrap();

        match connection.next().await {
            Some(update) => match update {
                ExternalUpdates::Layer(content) => assert_eq!(content, "Test"),
                _ => panic!("Unexpected update type"),
            },
            _ => panic!("Expected to receive an update"),
        }

        assert!(
            connection.next().await.is_none(),
            "Stream should return None after receiving an update and closing"
        );
    }

    #[tokio::test]
    async fn stream_api_connection_closes_after_join_set_returns() {
        let (data_sender, data_receiver) = mpsc::channel(10);
        let (command_sender, _) = broadcast::channel(10);
        let mut join_set = JoinSet::new();
        join_set.spawn(futures::future::ready(Err(FatalError)));
        let handle = tokio::runtime::Handle::current();

        let mut connection = StreamApi::new(join_set, data_receiver, command_sender, handle);

        let expected_update = ExternalUpdates::Layer("Test".to_owned());
        data_sender.send(expected_update.clone()).await.unwrap();

        match connection.next().await {
            Some(update) => match update {
                ExternalUpdates::Layer(content) => assert_eq!(content, "Test"),
                _ => panic!("Unexpected update type"),
            },
            _ => panic!("Expected to receive an update"),
        }

        assert!(
            connection.next().await.is_none(),
            "Stream should return None after receiving an update and closing"
        );
    }

    #[tokio::test]
    async fn stream_api_connection_shuts_down_on_data_feed_close() {
        let (data_sender, data_receiver) = mpsc::channel::<ExternalUpdates<String>>(1);
        let (command_sender, _) = broadcast::channel(10);
        let join_set = JoinSet::new();
        let handle = tokio::runtime::Handle::current();

        let mut connection = StreamApi::new(join_set, data_receiver, command_sender, handle);

        drop(data_sender); // This closes the data_feed channel

        assert!(
            connection.next().await.is_none(),
            "Stream should have ended due to data feed close"
        );
    }
}
