use std::convert::Infallible as Never;

use betfair_adapter::{ApplicationKey, BetfairUrl, SessionToken};
use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::response::ResponseMessage;
use futures::{pin_mut, FutureExt, SinkExt};
use futures_concurrency::prelude::*;
use tokio_stream::StreamExt;

use crate::cache::tracker::{IncomingMessage, StreamStateTracker};
use crate::connection::handshake::{Handshake};
use crate::tls_sream::RawStreamApiConnection;
use crate::{CacheEnabledMessages, ExternalUpdates, HeartbeatStrategy, MetadataUpdates};

#[derive(Debug, Clone)]
pub enum AsyncTaskStopReason {
    FatalError,
    NeedsRestart,
}

pub async fn broadcast_internal_updates(
    mut updates_receiver: tokio::sync::broadcast::Receiver<MetadataUpdates>,
    output_queue_sender: tokio::sync::mpsc::Sender<ExternalUpdates<ResponseMessage>>,
) -> Result<Never, AsyncTaskStopReason> {
    while let Ok(msg) = updates_receiver.recv().await {
        output_queue_sender
            .send(ExternalUpdates::Metadata(msg))
            .await
            .map_err(|_| AsyncTaskStopReason::FatalError)?;
    }

    Err(AsyncTaskStopReason::FatalError)
}

pub async fn connect_and_process(
    url: BetfairUrl<betfair_adapter::Stream>,
    sender: tokio::sync::mpsc::Sender<ExternalUpdates<ResponseMessage>>,
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
        // TODO: check if we need to fetch a new token
        let connection = crate::tls_sream::connect(url.clone()).await;

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

pub async fn handle_stream_connection(
    connection: RawStreamApiConnection,
    sender: tokio::sync::mpsc::Sender<ExternalUpdates<ResponseMessage>>,
    mut command_reader: tokio::sync::broadcast::Receiver<RequestMessage>,
    session_token: SessionToken,
    application_key: ApplicationKey,
    authenticated: futures::channel::oneshot::Sender<()>,
) -> Result<Never, AsyncTaskStopReason> {
    pin_mut!(connection);

    {
        // do handshake
        let mut handshake = Handshake::new(&session_token, &application_key, connection.as_mut());
        while let Some(results) = handshake.next().await {
            match results {
                Ok(msg) => {
                    sender
                        .send(msg)
                        .await
                        .map_err(|_| AsyncTaskStopReason::FatalError)?;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        authenticated.send(()).map_err(|_| {
            tracing::error!("Failed to commuincate with the auth one-shot channel");
            AsyncTaskStopReason::NeedsRestart
        })?;
    }

    tracing::info!("authenticated");

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

pub async fn cache_loop(
    mut receiver: tokio::sync::mpsc::Receiver<ExternalUpdates<ResponseMessage>>,
    external_sender: tokio::sync::mpsc::Sender<ExternalUpdates<CacheEnabledMessages>>,
) -> Result<Never, AsyncTaskStopReason> {
    let mut state = StreamStateTracker::new();
    while let Some(msg) = receiver.recv().await {
        let mut publish_time = None;
        if let ExternalUpdates::Layer(msg) = msg {
            let updates = match msg {
                ResponseMessage::MarketChange(msg) => {
                    publish_time = msg.publish_time;
                    state.calculate_updates(IncomingMessage::Market(msg))
                }
                ResponseMessage::OrderChange(msg) => {
                    publish_time = msg.publish_time;
                    state.calculate_updates(IncomingMessage::Order(msg))
                }
                ResponseMessage::Connection(msg) => {
                    external_sender
                        .send(ExternalUpdates::Layer(
                            CacheEnabledMessages::ConnectionMessage(msg),
                        ))
                        .await
                        .map_err(|_| AsyncTaskStopReason::FatalError)?;
                    None
                }
                ResponseMessage::Status(msg) => {
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
        } else if let Some(msg) = map_update(msg) {
            external_sender
                .send(msg)
                .await
                .map_err(|_| AsyncTaskStopReason::FatalError)?;
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
