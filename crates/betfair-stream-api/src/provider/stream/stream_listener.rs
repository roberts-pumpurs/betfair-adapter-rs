use std::convert::Infallible as Never;
use std::marker::PhantomData;
use std::time::Duration;

use betfair_adapter::{ApplicationKey, BetfairUrl, SessionToken};
use betfair_stream_types::request::{authentication_message, RequestMessage};
use betfair_stream_types::response::connection_message::ConnectionMessage;
use betfair_stream_types::response::market_change_message::MarketChangeMessage;
use betfair_stream_types::response::order_change_message::OrderChangeMessage;
use betfair_stream_types::response::status_message::StatusMessage;
use betfair_stream_types::response::{self, ResponseMessage};
use futures::{Future, FutureExt, Stream, TryFutureExt, TryStreamExt};
use futures_concurrency::prelude::*;
use tokio::runtime::Handle;
use tokio::task::JoinSet;
use tokio_stream::wrappers::{BroadcastStream, ReceiverStream};
use tokio_stream::StreamExt;

use super::raw_stream_connection;
use crate::cache::primitives::{MarketBookCache, OrderBookCache};
use crate::cache::tracker::{IncomingMessage, StreamStateTracker};
use crate::StreamError;

#[derive(Debug, Clone)]
pub enum HeartbeatStrategy {
    None,
    Interval(Duration),
}

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

#[derive(Debug, Clone)]
pub enum ExternalUpdates<T> {
    Layer(T),
    Metadata(MetadataUpdates),
}

#[derive(Debug, Clone)]
pub enum BaseLayer {
    MarketChangeMessage(MarketChangeMessage),
    OrderChangeMessage(OrderChangeMessage),
    StatusMessage(StatusMessage),
    ConnectionMessage(ConnectionMessage),
}

#[derive(Debug, Clone)]
pub enum MetadataUpdates {
    Disconnected,
    TcpConnected,
    FailedToConnect,
    Authenticated {
        connections_available: i32,
        connection_id: String,
    },
    FailedToAuthenticate,
}

#[derive(Debug, Clone)]
enum AsyncTaskStopReason {
    FatalError,
    NeedsRestart,
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

    pub fn run_with_default_runtime(&mut self) -> StreamApiConnection<PostAuthMessages> {
        self.run(&Handle::current())
    }

    pub fn run(
        &mut self,
        rt_handle: &tokio::runtime::Handle,
    ) -> StreamApiConnection<PostAuthMessages> {
        let (join_set, data_feed) = self.run_internal(&rt_handle);
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
        let (mut join_set, data_feed) = self.run_internal(&rt_handle);
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
        tokio::sync::mpsc::Receiver<ExternalUpdates<PostAuthMessages>>,
    ) {
        let (output_queue_sender, output_queue_reader) = tokio::sync::mpsc::channel(3);
        let (updates_sender, updates_receiver) = tokio::sync::broadcast::channel(3);

        let mut join_set = JoinSet::new();
        join_set.spawn_on(
            broadcast_internal_updates(updates_receiver.resubscribe(), output_queue_sender.clone()),
            &rt_handle,
        );
        join_set.spawn_on(
            connect_and_process(
                self.url.clone(),
                output_queue_sender,
                self.command_reader.resubscribe(),
                updates_sender.clone(),
            ),
            &rt_handle,
        );
        join_set.spawn_on(
            heartbeat_loop(
                self.hb.clone(),
                updates_receiver.resubscribe(),
                self.command_sender.clone(),
            ),
            &rt_handle,
        );

        let (output_queue_sender_post_auth, output_queue_reader_post_auth) =
            tokio::sync::mpsc::channel(3);

        join_set.spawn_on(
            authentication_loop(
                output_queue_reader,
                output_queue_sender_post_auth,
                self.command_sender.clone(),
                updates_sender.clone(),
                self.session_token.clone(),
                self.application_key.clone(),
            ),
            &rt_handle,
        );
        (join_set, output_queue_reader_post_auth)
    }
}

fn wrap_with_cache_layer(
    join_set: &mut JoinSet<Result<Never, AsyncTaskStopReason>>,
    data_feed: tokio::sync::mpsc::Receiver<ExternalUpdates<PostAuthMessages>>,
    rt_handle: &tokio::runtime::Handle,
) -> tokio::sync::mpsc::Receiver<ExternalUpdates<CacheEnabledMessages>> {
    let (output_queue_sender_post_cache, output_queue_reader_post_cache) =
        tokio::sync::mpsc::channel(3);
    join_set.spawn_on(
        cache_loop(data_feed, output_queue_sender_post_cache),
        &rt_handle,
    );
    output_queue_reader_post_cache
}

#[derive(Debug)]
pub struct StreamApiConnection<T> {
    join_set: JoinSet<Result<Never, AsyncTaskStopReason>>,
    rt_handle: tokio::runtime::Handle,
    completed: bool,
    is_shutting_down: bool,
    data_feed: tokio::sync::mpsc::Receiver<ExternalUpdates<T>>,
    command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
}

impl<T> StreamApiConnection<T> {
    pub(crate) fn new(
        join_set: JoinSet<Result<Never, AsyncTaskStopReason>>,
        data_feed: tokio::sync::mpsc::Receiver<ExternalUpdates<T>>,
        command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
        rt_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            completed: false,
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

impl StreamApiConnection<PostAuthMessages> {
    pub async fn enable_cache(mut self) -> StreamApiConnection<CacheEnabledMessages> {
        let output_queue_reader_post_cache =
            wrap_with_cache_layer(&mut self.join_set, self.data_feed, &self.rt_handle);
        StreamApiConnection {
            join_set: self.join_set,
            completed: self.completed,
            rt_handle: self.rt_handle,
            is_shutting_down: self.is_shutting_down,
            data_feed: output_queue_reader_post_cache,
            command_sender: self.command_sender,
        }
    }
}

impl<T> Stream for StreamApiConnection<T> {
    type Item = ExternalUpdates<T>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if self.join_set.is_empty() && self.is_shutting_down {
            self.completed = true;
            return std::task::Poll::Ready(None);
        }

        // poll the join set to see if any child tasks have completed
        let response = self.join_set.try_join_next();
        if let Some(_task_return) = response {
            self.join_set.abort_all();
            self.is_shutting_down = true;
            return std::task::Poll::Pending;
        }

        // poll the data feed
        self.data_feed.poll_recv(cx)
    }
}

impl<T> Unpin for StreamApiConnection<T> {}

async fn broadcast_internal_updates(
    mut updates_receiver: tokio::sync::broadcast::Receiver<MetadataUpdates>,
    output_queue_sender: tokio::sync::mpsc::Sender<ExternalUpdates<BaseLayer>>,
) -> Result<Never, AsyncTaskStopReason> {
    while let Ok(msg) = updates_receiver.recv().await {
        output_queue_sender
            .send(ExternalUpdates::Metadata(msg))
            .await
            .map_err(|_| AsyncTaskStopReason::FatalError)?;
    }

    Err(AsyncTaskStopReason::FatalError)
}

async fn connect_and_process(
    url: BetfairUrl<betfair_adapter::Stream>,
    sender: tokio::sync::mpsc::Sender<ExternalUpdates<BaseLayer>>,
    command_reader: tokio::sync::broadcast::Receiver<RequestMessage>,
    updates_sender: tokio::sync::broadcast::Sender<MetadataUpdates>,
) -> Result<Never, AsyncTaskStopReason> {
    loop {
        let connection = raw_stream_connection::connect(
            url.clone(),
            BroadcastStream::new(command_reader.resubscribe())
                .map_err(|_| eyre::eyre!("Failed to read command")),
        )
        .await;

        updates_sender
            .send(MetadataUpdates::TcpConnected)
            .map_err(|_| AsyncTaskStopReason::FatalError)?;

        let Ok((write_to_socket, read)) = connection else {
            let _res = updates_sender
                .send(MetadataUpdates::FailedToConnect)
                .map_err(|_| AsyncTaskStopReason::FatalError)?;
            continue;
        };

        let mut h1 = tokio::spawn(write_to_socket.map_err(|_x| AsyncTaskStopReason::NeedsRestart));
        let mut h2 = tokio::spawn(parse_stream_api_messagse(sender.clone(), read).map(|_| Ok(())));

        let result = (&mut h1).race(&mut h2).await;
        h1.abort();
        h2.abort();
        if let Ok(Err(AsyncTaskStopReason::FatalError)) = result {
            return Err(AsyncTaskStopReason::FatalError);
        }
        updates_sender
            .send(MetadataUpdates::Disconnected)
            .map_err(|_| AsyncTaskStopReason::FatalError)?;
    }
}

/// Read from the Betfair stream and process the messages, send them away!
async fn parse_stream_api_messagse(
    sender: tokio::sync::mpsc::Sender<ExternalUpdates<BaseLayer>>,
    read: impl Stream<Item = Result<ResponseMessage, StreamError>>,
) -> Result<Never, AsyncTaskStopReason> {
    tokio::pin!(read);
    while let Some(msg) = read.next().await {
        match msg {
            Ok(msg) => {
                tracing::debug!(msg = ?msg, "Received message");
                let msg = match msg {
                    ResponseMessage::Connection(msg) => BaseLayer::ConnectionMessage(msg),
                    ResponseMessage::MarketChange(msg) => BaseLayer::MarketChangeMessage(msg),
                    ResponseMessage::OrderChange(msg) => BaseLayer::OrderChangeMessage(msg),
                    ResponseMessage::StatusMessage(msg) => BaseLayer::StatusMessage(msg),
                };
                sender
                    .send(ExternalUpdates::Layer(msg))
                    .await
                    .map_err(|_| AsyncTaskStopReason::FatalError)?;
            }
            Err(err) => {
                tracing::error!(err = ?err, "Error reading from stream!");
                break;
            }
        }
    }

    Err(AsyncTaskStopReason::NeedsRestart)
}

async fn heartbeat_loop(
    hb: HeartbeatStrategy,
    mut updates_receiver: tokio::sync::broadcast::Receiver<MetadataUpdates>,
    command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
) -> Result<Never, AsyncTaskStopReason> {
    match hb {
        HeartbeatStrategy::None => loop {
            std::future::pending::<()>().await;
        },
        HeartbeatStrategy::Interval(period) => {
            let mut interval = tokio::time::interval(period);
            interval.reset();
            let mut id: i32 = 0;
            let mut is_connected = false;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        id = id.wrapping_add(1);
                        if !is_connected {continue}
                        command_sender
                            .send(RequestMessage::Heartbeat(
                                betfair_stream_types::request::heartbeat_message::HeartbeatMessage {
                                    id: Some(id),
                                },
                            ))
                            .map_err(|_| AsyncTaskStopReason::FatalError)?;
                    }
                    Ok(msg) = updates_receiver.recv() => {
                        if matches!(msg, MetadataUpdates::Authenticated {connections_available: _, connection_id: _,}) {
                            is_connected = true;
                        }
                        if matches!(msg, MetadataUpdates::Disconnected) {
                            is_connected = false;
                        }
                    }
                    else => {
                        break;
                    }
                }
            }
        }
    };

    Err(AsyncTaskStopReason::FatalError)
}

#[derive(Debug, Clone)]
pub enum PostAuthMessages {
    MarketChangeMessage(MarketChangeMessage),
    OrderChangeMessage(OrderChangeMessage),
    ConnectionMessage(ConnectionMessage),
}

async fn authentication_loop(
    mut receiver: tokio::sync::mpsc::Receiver<ExternalUpdates<BaseLayer>>,
    external_sender: tokio::sync::mpsc::Sender<ExternalUpdates<PostAuthMessages>>,
    command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
    updates_sender: tokio::sync::broadcast::Sender<MetadataUpdates>,
    session_token: SessionToken,
    application_key: ApplicationKey,
) -> Result<Never, AsyncTaskStopReason> {
    let mut id = 1;
    while let Some(msg) = receiver.recv().await {
        let processed_msg = if let ExternalUpdates::Layer(msg) = msg {
            let post_auth_msg =
                match msg {
                    BaseLayer::StatusMessage(msg) => {
                        let span = tracing::info_span!("received status message", msg =? msg);
                        let _guard = span.enter();

                        let could_not_authenticate = || {
                            updates_sender
                                .send(MetadataUpdates::FailedToAuthenticate)
                                .map_err(|_| AsyncTaskStopReason::FatalError)
                        };
                        if matches!(msg.connection_closed, Some(true)) ||
                            msg.error_code.is_some() ||
                            matches!(msg.status_code, Some(
                            betfair_stream_types::response::status_message::StatusCode::Failure,
                        )) {
                            tracing::warn!("error code present or connection closed");
                            could_not_authenticate()?;
                            continue;
                        }

                        let Some(connections_available) = msg.connections_available else {
                            tracing::warn!("connections not available");
                            could_not_authenticate()?;
                            continue;
                        };
                        let Some(connection_id) = msg.connection_id.clone() else {
                            tracing::warn!("connections id not available");
                            could_not_authenticate()?;
                            continue;
                        };

                        updates_sender
                            .send(MetadataUpdates::Authenticated {
                                connections_available,
                                connection_id,
                            })
                            .map_err(|_| AsyncTaskStopReason::FatalError)?;

                        None
                    }
                    BaseLayer::ConnectionMessage(msg) => {
                        let authorization_message = authentication_message::AuthenticationMessage {
                            id: Some(id),
                            session: session_token.0.expose_secret().clone(),
                            app_key: application_key.0.expose_secret().clone(),
                        };
                        id = id.wrapping_add(1);
                        command_sender
                            .send(RequestMessage::Authentication(authorization_message))
                            .map_err(|_| AsyncTaskStopReason::FatalError)?;

                        Some(PostAuthMessages::ConnectionMessage(msg))
                    }
                    BaseLayer::MarketChangeMessage(msg) => {
                        Some(PostAuthMessages::MarketChangeMessage(msg))
                    }
                    BaseLayer::OrderChangeMessage(msg) => {
                        Some(PostAuthMessages::OrderChangeMessage(msg))
                    }
                };

            post_auth_msg.map(ExternalUpdates::Layer)
        } else {
            map_update(msg)
        };

        if let Some(processed_msg) = processed_msg {
            external_sender
                .send(processed_msg)
                .await
                .map_err(|_| AsyncTaskStopReason::FatalError)?;
        }
    }
    Err(AsyncTaskStopReason::FatalError)
}

#[derive(Debug, Clone)]
pub enum CacheEnabledMessages {
    MarketChangeMessage(Vec<MarketBookCache>),
    OrderChangeMessage(Vec<OrderBookCache>),
    ConnectionMessage(ConnectionMessage),
}

async fn cache_loop(
    mut receiver: tokio::sync::mpsc::Receiver<ExternalUpdates<PostAuthMessages>>,
    external_sender: tokio::sync::mpsc::Sender<ExternalUpdates<CacheEnabledMessages>>,
) -> Result<Never, AsyncTaskStopReason> {
    let mut state = StreamStateTracker::new();
    while let Some(msg) = receiver.recv().await {
        let mut publish_time = None;
        if let ExternalUpdates::Layer(msg) = msg {
            let updates = match msg {
                PostAuthMessages::MarketChangeMessage(msg) => {
                    publish_time = msg.publish_time;
                    state.calculate_updates(IncomingMessage::Market(msg))
                }
                PostAuthMessages::OrderChangeMessage(msg) => {
                    publish_time = msg.publish_time;
                    state.calculate_updates(IncomingMessage::Order(msg))
                }
                PostAuthMessages::ConnectionMessage(_msg) => None,
            };
            let Some(updates) = updates else {
                continue;
            };
            let update = match updates {
                crate::cache::tracker::Updates::Market(msg) => {
                    CacheEnabledMessages::MarketChangeMessage(msg.into_iter().cloned().collect())
                }
                crate::cache::tracker::Updates::Order(msg) => {
                    CacheEnabledMessages::OrderChangeMessage(msg.into_iter().cloned().collect())
                }
            };

            external_sender
                .send(ExternalUpdates::Layer(update))
                .await
                .map_err(|_| AsyncTaskStopReason::FatalError)?;
        } else {
            if let Some(msg) = map_update(msg) {
                external_sender
                    .send(msg)
                    .await
                    .map_err(|_| AsyncTaskStopReason::FatalError)?;
            }
        }
        if let Some(publish_time) = publish_time {
            state.clear_stale_cache(publish_time);
        }
    }

    Err(AsyncTaskStopReason::FatalError)
}

fn map_update<T, K>(from: ExternalUpdates<T>) -> Option<ExternalUpdates<K>> {
    match from {
        ExternalUpdates::Layer(_old_layer) => None,
        ExternalUpdates::Metadata(data) => Some(ExternalUpdates::Metadata(data)),
    }
}
