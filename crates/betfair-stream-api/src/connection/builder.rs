use core::convert::Infallible as Never;
use core::time::Duration;

use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::response::ResponseMessage;
use tokio::runtime::Handle;
use tokio::task::JoinSet;

use super::cron::{self, FatalError};
use super::StreamApi;
use crate::{CacheEnabledMessages, ExternalUpdates};

/// Represents the strategy for sending heartbeat messages.
#[derive(Debug, Clone)]
pub enum HeartbeatStrategy {
    /// No heartbeat strategy.
    None,
    /// Send heartbeat messages at a specified interval.
    Interval(Duration),
}

/// Builder for creating a StreamApi instance.
#[expect(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct StreamApiBuilder {
    /// Sender for sending commands to the underlying stream.
    command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
    /// Receiver for reading commands from the underlying stream.
    command_reader: tokio::sync::broadcast::Receiver<RequestMessage>,
    /// Betfair provider for making requests.
    provider: betfair_adapter::UnauthenticatedBetfairRpcProvider,
    /// Heartbeat strategy for the stream.
    hb: HeartbeatStrategy,
}

impl StreamApiBuilder {
    /// Creates a new StreamApiBuilder with the specified provider and heartbeat strategy.
    #[must_use]
    pub fn new(
        provider: betfair_adapter::UnauthenticatedBetfairRpcProvider,
        hb: HeartbeatStrategy,
    ) -> Self {
        let (command_sender, command_reader) = tokio::sync::broadcast::channel(3);

        Self {
            command_sender,
            command_reader,
            provider,
            hb,
        }
    }

    /// Runs the StreamApi with the default Tokio runtime.
    #[must_use]
    pub fn run_with_default_runtime(&self) -> StreamApi<ResponseMessage> {
        self.run(&Handle::current())
    }

    /// Runs the StreamApi with the specified Tokio runtime handle.
    #[must_use]
    pub fn run(&self, rt_handle: &tokio::runtime::Handle) -> StreamApi<ResponseMessage> {
        let (join_set, data_feed) = self.run_internal(rt_handle);
        StreamApi::new(
            join_set,
            data_feed,
            self.command_sender.clone(),
            rt_handle.clone(),
        )
    }

    /// Runs the StreamApi with caching enabled.
    #[must_use]
    pub fn run_with_cache(
        &self,
        rt_handle: &tokio::runtime::Handle,
    ) -> StreamApi<CacheEnabledMessages> {
        let (mut join_set, data_feed) = self.run_internal(rt_handle);
        let output_queue_reader_post_cache =
            wrap_with_cache_layer(&mut join_set, data_feed, rt_handle);
        StreamApi::new(
            join_set,
            output_queue_reader_post_cache,
            self.command_sender.clone(),
            rt_handle.clone(),
        )
    }

    /// Internal method to run the StreamApi and return a join set and data feed.
    fn run_internal(
        &self,
        rt_handle: &tokio::runtime::Handle,
    ) -> (
        JoinSet<Result<Never, FatalError>>,
        tokio::sync::mpsc::Receiver<ExternalUpdates<ResponseMessage>>,
    ) {
        let (output_queue_sender, output_queue_reader) = tokio::sync::mpsc::channel(3);

        let mut join_set = JoinSet::new();
        join_set.spawn_on(
            {
                let command_reader = self.command_reader.resubscribe();
                let command_sender = self.command_sender.clone();
                let provider = self.provider.clone();
                let runtime_handle = rt_handle.clone();
                let hb = self.hb.clone();
                async move {
                    cron::StreamConnectionProcessor {
                        sender: output_queue_sender,
                        command_reader,
                        command_sender,
                        provider,
                        runtime_handle,
                        hb,
                        last_time_token_refreshed: None,
                    }
                    .connect_and_process_loop()
                    .await
                }
            },
            rt_handle,
        );

        (join_set, output_queue_reader)
    }
}

/// Wraps the data feed with a cache layer and returns the receiver for external updates.
pub(crate) fn wrap_with_cache_layer(
    join_set: &mut JoinSet<Result<Never, FatalError>>,
    data_feed: tokio::sync::mpsc::Receiver<ExternalUpdates<ResponseMessage>>,
    rt_handle: &tokio::runtime::Handle,
) -> tokio::sync::mpsc::Receiver<ExternalUpdates<CacheEnabledMessages>> {
    let (output_queue_sender_post_cache, output_queue_reader_post_cache) =
        tokio::sync::mpsc::channel(3);
    join_set.spawn_on(
        cron::cache_loop(data_feed, output_queue_sender_post_cache),
        rt_handle,
    );
    output_queue_reader_post_cache
}
