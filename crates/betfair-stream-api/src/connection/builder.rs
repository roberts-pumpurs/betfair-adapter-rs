use std::convert::Infallible as Never;
use std::time::Duration;

use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::response::ResponseMessage;
use tokio::runtime::Handle;
use tokio::task::JoinSet;

use super::cron::{self, FatalError};
use super::StreamApiConnection;
use crate::{CacheEnabledMessages, ExternalUpdates};

#[derive(Debug, Clone)]
pub enum HeartbeatStrategy {
    None,
    Interval(Duration),
}

#[derive(Debug)]
pub struct StreamApiBuilder {
    /// Send data to the underlying stream
    command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
    command_reader: tokio::sync::broadcast::Receiver<RequestMessage>,
    /// Betfair provider
    provider: betfair_adapter::UnauthenticatedBetfairRpcProvider,
    /// heartbeat strategy
    hb: HeartbeatStrategy,
}

impl StreamApiBuilder {
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

    pub fn run_with_default_runtime(&mut self) -> StreamApiConnection<ResponseMessage> {
        self.run(&Handle::current())
    }

    pub fn run(
        &mut self,
        rt_handle: &tokio::runtime::Handle,
    ) -> StreamApiConnection<ResponseMessage> {
        let (join_set, data_feed) = self.run_internal(rt_handle);
        StreamApiConnection::new(
            join_set,
            data_feed,
            self.command_sender.clone(),
            rt_handle.clone(),
        )
    }

    pub fn run_with_cache(
        &mut self,
        rt_handle: &tokio::runtime::Handle,
    ) -> StreamApiConnection<CacheEnabledMessages> {
        let (mut join_set, data_feed) = self.run_internal(rt_handle);
        let output_queue_reader_post_cache =
            wrap_with_cache_layer(&mut join_set, data_feed, rt_handle);
        StreamApiConnection::new(
            join_set,
            output_queue_reader_post_cache,
            self.command_sender.clone(),
            rt_handle.clone(),
        )
    }

    fn run_internal(
        &mut self,
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
                    cron::StreamConnectioProcessor {
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

pub fn wrap_with_cache_layer(
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
