pub mod builder;
pub mod cron;
pub mod handshake;

use std::convert::Infallible as Never;
use std::task::Poll;

use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::response::connection_message::ConnectionMessage;
use betfair_stream_types::response::status_message::StatusMessage;
use betfair_stream_types::response::ResponseMessage;
use futures::Stream;
use tokio::task::JoinSet;

use self::builder::wrap_with_cache_layer;
use self::cron::FatalError;
use crate::cache::primitives::{MarketBookCache, OrderBookCache};

#[derive(Debug)]
pub struct StreamApiConnection<T> {
    join_set: JoinSet<Result<Never, FatalError>>,
    rt_handle: tokio::runtime::Handle,
    is_shutting_down: bool,
    data_feed: tokio::sync::mpsc::Receiver<ExternalUpdates<T>>,
    command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
}

impl<T> StreamApiConnection<T> {
    pub(crate) fn new(
        join_set: JoinSet<Result<Never, FatalError>>,
        data_feed: tokio::sync::mpsc::Receiver<ExternalUpdates<T>>,
        command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
        rt_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            is_shutting_down: false,
            join_set,
            rt_handle: rt_handle.clone(),
            data_feed,
            command_sender,
        }
    }

    pub fn command_sender(&self) -> &tokio::sync::broadcast::Sender<RequestMessage> {
        &self.command_sender
    }
}

impl StreamApiConnection<ResponseMessage> {
    pub async fn enable_cache(mut self) -> StreamApiConnection<CacheEnabledMessages> {
        let output_queue_reader_post_cache =
            wrap_with_cache_layer(&mut self.join_set, self.data_feed, &self.rt_handle);
        StreamApiConnection {
            join_set: self.join_set,
            rt_handle: self.rt_handle,
            is_shutting_down: self.is_shutting_down,
            data_feed: output_queue_reader_post_cache,
            command_sender: self.command_sender,
        }
    }
}

impl<T> Stream for StreamApiConnection<T> {
    type Item = ExternalUpdates<T>;

    // todo write unittests for this, because therere are edge cases where we hang up
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // only return None if we are shutting down and there are no tasks left
        if self.join_set.is_empty() && self.is_shutting_down {
            tracing::warn!("StreamApiConnection: No tasks remaining, shutting down.");
            return Poll::Ready(None);
        }

        // Poll the join set to check if any child tasks have completed
        match self.join_set.poll_join_next(cx) {
            Poll::Ready(Some(Ok(Err(e)))) => {
                tracing::error!("Error returned by a task: {:?}", e);
                self.join_set.abort_all();
                self.is_shutting_down = true;
                cx.waker().wake_by_ref();
            }
            Poll::Ready(Some(Ok(Ok(_e)))) => {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            Poll::Ready(Some(Err(e))) => {
                tracing::error!("Error in join_set: {:?}", e);
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

impl<T> Unpin for StreamApiConnection<T> {}

#[derive(Debug, Clone)]
pub enum ExternalUpdates<T> {
    Layer(T),
    Metadata(MetadataUpdates),
}

#[derive(Debug, Clone)]
pub enum CacheEnabledMessages {
    MarketChangeMessage(Vec<MarketBookCache>),
    OrderChangeMessage(Vec<OrderBookCache>),
    ConnectionMessage(ConnectionMessage),
    StatusMessage(StatusMessage),
}

#[derive(Debug, Clone)]
pub enum MetadataUpdates {
    Disconnected,
    TcpConnected,
    FailedToConnect,
    AuthenticationMessageSent,
    Authenticated {
        connections_available: i32,
        connection_id: Option<String>,
    },
    FailedToAuthenticate,
}
