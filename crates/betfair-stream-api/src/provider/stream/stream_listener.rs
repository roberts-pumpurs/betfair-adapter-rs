use std::convert::Infallible as Never;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::Poll;
use std::time::Duration;

use betfair_adapter::betfair_types::types::heartbeat_aping::heartbeat;
use betfair_adapter::{ApplicationKey, BetfairUrl, SessionToken};
use betfair_stream_types::request::{authentication_message, RequestMessage};
use betfair_stream_types::response::connection_message::ConnectionMessage;
use betfair_stream_types::response::market_change_message::MarketChangeMessage;
use betfair_stream_types::response::order_change_message::OrderChangeMessage;
use betfair_stream_types::response::status_message::{StatusCode, StatusMessage};
use betfair_stream_types::response::{self, ResponseMessage};
use futures::stream::FuturesUnordered;
use futures::task::SpawnExt;
use futures::{pin_mut, Future, FutureExt, SinkExt, Stream, TryFutureExt, TryStreamExt};
use futures_concurrency::prelude::*;
use tokio::runtime::Handle;
use tokio::task::JoinSet;
use tokio_stream::wrappers::{BroadcastStream, ReceiverStream};
use tokio_stream::StreamExt;

use super::raw_stream_connection::{self, RawStreamApiConnection};
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
pub enum BetfairData {
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
        connection_id: Option<String>,
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

    pub fn run_with_default_runtime(&mut self) -> StreamApiConnection<BetfairData> {
        self.run(&Handle::current())
    }

    pub fn run(&mut self, rt_handle: &tokio::runtime::Handle) -> StreamApiConnection<BetfairData> {
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
        tokio::sync::mpsc::Receiver<ExternalUpdates<BetfairData>>,
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
                self.command_sender.clone(),
                updates_sender.clone(),
                self.session_token.clone(),
                self.application_key.clone(),
                rt_handle.clone(),
                self.hb.clone(),
            ),
            &rt_handle,
        );

        (join_set, output_queue_reader)
    }
}

fn wrap_with_cache_layer(
    join_set: &mut JoinSet<Result<Never, AsyncTaskStopReason>>,
    data_feed: tokio::sync::mpsc::Receiver<ExternalUpdates<BetfairData>>,
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

impl StreamApiConnection<BetfairData> {
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
            }
            Poll::Ready(Some(Ok(Ok(_e)))) => {
                return Poll::Pending;
            }
            Poll::Ready(Some(Err(e))) => {
                tracing::error!("Error in join_set: {:?}", e);
                self.join_set.abort_all();
                self.is_shutting_down = true;
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

async fn broadcast_internal_updates(
    mut updates_receiver: tokio::sync::broadcast::Receiver<MetadataUpdates>,
    output_queue_sender: tokio::sync::mpsc::Sender<ExternalUpdates<BetfairData>>,
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
    sender: tokio::sync::mpsc::Sender<ExternalUpdates<BetfairData>>,
    // todo replace all broadcasts with mpsc
    command_reader: tokio::sync::broadcast::Receiver<RequestMessage>,
    command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
    updates_sender: tokio::sync::broadcast::Sender<MetadataUpdates>,
    session_token: SessionToken,
    application_key: ApplicationKey,
    runtime_handle: tokio::runtime::Handle,
    hb: HeartbeatStrategy,
) -> Result<Never, AsyncTaskStopReason> {
    loop {
        let connection = raw_stream_connection::connect(url.clone()).await;

        updates_sender
            .send(MetadataUpdates::TcpConnected)
            .map_err(|_| AsyncTaskStopReason::FatalError)?;

        let Ok(connection) = connection else {
            let _res = updates_sender
                .send(MetadataUpdates::FailedToConnect)
                .map_err(|_| AsyncTaskStopReason::FatalError)?;
            // continue;
            // connection.err
            tracing::error!(err =? connection.err(), "Failed to connect to the stream, retrying...");
            return Err(AsyncTaskStopReason::FatalError);
        };

        let (auth_sender, auth_recv) = futures::channel::oneshot::channel();
        let mut process = runtime_handle.spawn(handle_stream_connection(
            connection,
            sender.clone(),
            command_reader.resubscribe(),
            session_token.clone(),
            application_key.clone(),
            auth_sender,
        ));

        if auth_recv.await.is_err() {
            process.abort();
            continue;
        }

        let mut heartbeat =
            runtime_handle.spawn(heartbeat_loop(hb.clone(), command_sender.clone()));

        let result = (&mut heartbeat).race(&mut process).await;
        heartbeat.abort();
        process.abort();

        if let Ok(Err(AsyncTaskStopReason::FatalError)) = result {
            return Err(AsyncTaskStopReason::FatalError);
        }
    }
}

async fn handle_stream_connection(
    connection: RawStreamApiConnection,
    sender: tokio::sync::mpsc::Sender<ExternalUpdates<BetfairData>>,
    mut command_reader: tokio::sync::broadcast::Receiver<RequestMessage>,
    session_token: SessionToken,
    application_key: ApplicationKey,
    authenticated: futures::channel::oneshot::Sender<()>,
) -> Result<Never, AsyncTaskStopReason> {
    pin_mut!(connection);

    {
        // do handshake
        match handshake(
            session_token,
            application_key,
            &mut connection,
            sender.clone(),
        )
        .await
        {
            Ok(status_message) => {
                sender
                    .send(ExternalUpdates::Metadata(MetadataUpdates::Authenticated {
                        connections_available: status_message.connections_available.unwrap_or(-1),
                        connection_id: status_message.connection_id,
                    }))
                    .await
                    .map_err(|_| AsyncTaskStopReason::FatalError)?;
            }
            Err(_) => {
                sender
                    .send(ExternalUpdates::Metadata(
                        MetadataUpdates::FailedToAuthenticate,
                    ))
                    .await
                    .map_err(|_| AsyncTaskStopReason::FatalError)?;
                return Err(AsyncTaskStopReason::NeedsRestart);
            }
        }
        authenticated
            .send(())
            .map_err(|_| AsyncTaskStopReason::NeedsRestart)?;
    }

    loop {
        futures::select! {
            msg = command_reader.recv().fuse() => {
                match msg {
                    Ok(msg) => {
                        connection.send(msg).await.map_err(|_| AsyncTaskStopReason::NeedsRestart)?;
                    }
                    Err(_) => {
                        return Err(AsyncTaskStopReason::FatalError);
                    },
                }
            },
            read = connection.next().fuse() => {
                match read {
                    Some(Ok(msg)) => {
                        tracing::debug!(msg = ?msg, "Received message");
                            let msg = match msg {
                                ResponseMessage::Connection(msg) => BetfairData::ConnectionMessage(msg),
                                ResponseMessage::MarketChange(msg) => BetfairData::MarketChangeMessage(msg),
                                ResponseMessage::OrderChange(msg) => BetfairData::OrderChangeMessage(msg),
                                ResponseMessage::StatusMessage(msg) => BetfairData::StatusMessage(msg),
                            };
                        sender
                            .send(ExternalUpdates::Layer(msg))
                            .await
                            .map_err(|_| AsyncTaskStopReason::FatalError)?;
                    }
                    _ => {
                        return Err(AsyncTaskStopReason::NeedsRestart);
                    },
                }
            }
        }
    }
}

async fn handshake<'a>(
    session_token: SessionToken,
    application_key: ApplicationKey,
    mut connection: &mut Pin<&'a mut RawStreamApiConnection>,
    sender: tokio::sync::mpsc::Sender<ExternalUpdates<BetfairData>>,
) -> Result<StatusMessage, AsyncTaskStopReason> {
    async fn read_next<'a, S: Stream<Item = Result<ResponseMessage, StreamError>>>(
        read: &mut Pin<&'a mut S>,
    ) -> Result<ResponseMessage, AsyncTaskStopReason> {
        read.next()
            .await
            .ok_or(AsyncTaskStopReason::NeedsRestart)?
            .map_err(|_| AsyncTaskStopReason::NeedsRestart)
    }

    // get connection message
    let connection_message = read_next(&mut connection).await?;
    let ResponseMessage::Connection(connection_message) = connection_message else {
        tracing::error!(
            msg =? connection_message,
            "Expected connection message, got something else"
        );
        return Err(AsyncTaskStopReason::NeedsRestart);
    };
    sender
        .send(ExternalUpdates::Layer(BetfairData::ConnectionMessage(
            connection_message.clone(),
        )))
        .await
        .map_err(|_| AsyncTaskStopReason::FatalError)?;

    // send authentication message
    let authorization_message = authentication_message::AuthenticationMessage {
        id: Some(-1),
        session: session_token.0.expose_secret().clone(),
        app_key: application_key.0.expose_secret().clone(),
    };
    connection
        .send(RequestMessage::Authentication(authorization_message))
        .await
        .map_err(|_| AsyncTaskStopReason::FatalError)?;

    // get status message
    let status_message = read_next(&mut connection).await?;
    let ResponseMessage::StatusMessage(status_message) = status_message else {
        tracing::error!(
            msg =? status_message,
            "Expected status message, got something else"
        );
        return Err(AsyncTaskStopReason::NeedsRestart);
    };
    sender
        .send(ExternalUpdates::Layer(BetfairData::StatusMessage(
            status_message.clone(),
        )))
        .await
        .map_err(|_| AsyncTaskStopReason::FatalError)?;

    // parse status message
    if status_message
        .status_code
        .map(|x| x == StatusCode::Success)
        .unwrap_or(false)
    {
        tracing::info!("Successfully authenticated");
    } else {
        tracing::error!(
            msg =? status_message,
            "Failed to authenticate, got status message"
        );
        return Err(AsyncTaskStopReason::NeedsRestart);
    }
    Ok(status_message)
}

async fn heartbeat_loop(
    hb: HeartbeatStrategy,
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

            loop {
                futures::select! {
                    _ = interval.tick().fuse() => {
                        id = id.wrapping_add(1);
                        command_sender
                            .send(RequestMessage::Heartbeat(
                                betfair_stream_types::request::heartbeat_message::HeartbeatMessage {
                                    id: Some(id),
                                },
                            ))
                            .map_err(|_| AsyncTaskStopReason::FatalError)?;
                    }
                    complete => break,
                }
            }
        }
    };

    Err(AsyncTaskStopReason::FatalError)
}

#[derive(Debug, Clone)]
pub enum CacheEnabledMessages {
    MarketChangeMessage(Vec<MarketBookCache>),
    OrderChangeMessage(Vec<OrderBookCache>),
    ConnectionMessage(ConnectionMessage),
    StatusMessage(StatusMessage),
}

async fn cache_loop(
    mut receiver: tokio::sync::mpsc::Receiver<ExternalUpdates<BetfairData>>,
    external_sender: tokio::sync::mpsc::Sender<ExternalUpdates<CacheEnabledMessages>>,
) -> Result<Never, AsyncTaskStopReason> {
    let mut state = StreamStateTracker::new();
    while let Some(msg) = receiver.recv().await {
        let mut publish_time = None;
        if let ExternalUpdates::Layer(msg) = msg {
            let updates = match msg {
                BetfairData::MarketChangeMessage(msg) => {
                    publish_time = msg.publish_time;
                    state.calculate_updates(IncomingMessage::Market(msg))
                }
                BetfairData::OrderChangeMessage(msg) => {
                    publish_time = msg.publish_time;
                    state.calculate_updates(IncomingMessage::Order(msg))
                }
                BetfairData::ConnectionMessage(msg) => {
                    external_sender
                        .send(ExternalUpdates::Layer(
                            CacheEnabledMessages::ConnectionMessage(msg),
                        ))
                        .await
                        .map_err(|_| AsyncTaskStopReason::FatalError)?;
                    None
                }
                BetfairData::StatusMessage(msg) => {
                    external_sender
                        .send(ExternalUpdates::Layer(CacheEnabledMessages::StatusMessage(
                            msg,
                        )))
                        .await
                        .map_err(|_| AsyncTaskStopReason::FatalError)?;
                    None
                }
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
