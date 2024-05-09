use std::convert::Infallible as Never;
use std::pin::Pin;
use std::task::Poll;
use std::time::Duration;

use betfair_adapter::{ApplicationKey, BetfairUrl, SessionToken};
use betfair_stream_types::request::{authentication_message, RequestMessage};
use betfair_stream_types::response::connection_message::ConnectionMessage;
use betfair_stream_types::response::market_change_message::MarketChangeMessage;
use betfair_stream_types::response::order_change_message::OrderChangeMessage;
use betfair_stream_types::response::status_message::{StatusCode, StatusMessage};
use betfair_stream_types::response::ResponseMessage;
use futures::task::SpawnExt;
use futures::{pin_mut, FutureExt, SinkExt, Stream, TryFutureExt, TryStreamExt};
use futures_concurrency::prelude::*;
use tokio::runtime::Handle;
use tokio::task::JoinSet;
use tokio_stream::StreamExt;

use super::cron::{self, AsyncTaskStopReason};
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
    /// Application key
    application_key: ApplicationKey,
    /// Session token
    session_token: SessionToken,
    /// Stream URL
    url: BetfairUrl<betfair_adapter::Stream>,
    hb: HeartbeatStrategy,
}

impl StreamApiBuilder {
    pub fn new(
        application_key: ApplicationKey,
        session_token: SessionToken,
        url: BetfairUrl<betfair_adapter::Stream>,
        hb: HeartbeatStrategy,
    ) -> Self {
        let (command_sender, command_reader) = tokio::sync::broadcast::channel(3);

        Self {
            command_sender,
            command_reader,
            application_key,
            session_token,
            url,
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
        JoinSet<Result<Never, AsyncTaskStopReason>>,
        tokio::sync::mpsc::Receiver<ExternalUpdates<ResponseMessage>>,
    ) {
        let (output_queue_sender, output_queue_reader) = tokio::sync::mpsc::channel(3);
        let (updates_sender, updates_receiver) = tokio::sync::broadcast::channel(3);

        let mut join_set = JoinSet::new();
        join_set.spawn_on(
            cron::broadcast_internal_updates(
                updates_receiver.resubscribe(),
                output_queue_sender.clone(),
            ),
            rt_handle,
        );
        join_set.spawn_on(
            cron::connect_and_process(
                self.url.clone(),
                output_queue_sender,
                self.command_reader.resubscribe(),
                self.command_sender.clone(),
                updates_sender.clone(),
                self.session_token.clone(),
                self.application_key.clone(),
                rt_handle.clone(),
                self.hb.clone(),
            ),
            rt_handle,
        );

        (join_set, output_queue_reader)
    }
}

pub fn wrap_with_cache_layer(
    join_set: &mut JoinSet<Result<Never, AsyncTaskStopReason>>,
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
