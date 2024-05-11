use std::convert::Infallible as Never;
use std::fmt::Debug;

use betfair_adapter::{ApplicationKey, SessionToken, UnauthenticatedBetfairRpcProvider};
use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::response::ResponseMessage;
use chrono::DateTime;
use futures::{pin_mut, FutureExt, SinkExt, TryFutureExt};
use futures_concurrency::prelude::*;
use tokio_stream::StreamExt;

use crate::cache::tracker::{IncomingMessage, StreamStateTracker};
use crate::connection::handshake::Handshake;
use crate::tls_sream::RawStreamApiConnection;
use crate::{CacheEnabledMessages, ExternalUpdates, HeartbeatStrategy, MetadataUpdates};

#[derive(Debug, Clone)]
pub enum AsyncTaskStopReason {
    FatalError(FatalError),
    NeedsRestart(NeedsRestart),
}

#[derive(Debug, Clone)]
pub struct FatalError;

#[derive(Debug, Clone)]
pub struct NeedsRestart;

#[derive(Debug)]
pub struct StreamConnectioProcessor {
    pub sender: tokio::sync::mpsc::Sender<ExternalUpdates<ResponseMessage>>,
    pub command_reader: tokio::sync::broadcast::Receiver<RequestMessage>,
    pub command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
    pub provider: betfair_adapter::UnauthenticatedBetfairRpcProvider,
    pub runtime_handle: tokio::runtime::Handle,
    pub hb: HeartbeatStrategy,
    pub last_time_token_refreshed: Option<(std::time::Instant, SessionToken)>,
}

impl StreamConnectioProcessor {
    pub async fn connect_and_process_loop(&mut self) -> Result<Never, FatalError> {
        loop {
            match self.connect_and_process_internal().await {
                Ok(_) => {
                    tracing::info!("Stream connection processor stopped");
                    return Err(FatalError);
                }
                Err(AsyncTaskStopReason::FatalError(err)) => {
                    tracing::error!(err =? err, "Fatal error occurred, stopping the stream");
                    return Err(FatalError);
                }
                Err(AsyncTaskStopReason::NeedsRestart(_)) => {
                    tracing::warn!("Restarting the stream");
                    continue;
                }
            }
        }
    }

    async fn connect_and_process_internal(&mut self) -> Result<Never, AsyncTaskStopReason> {
        // get the session token
        let session_token = get_session_token(&mut self.last_time_token_refreshed, &self.provider)
            .await
            .map_err(|x| {
                tracing::error!(err =? x, "Failed to get the session token");
                AsyncTaskStopReason::NeedsRestart(x)
            })?;

        // connect to the stream
        let connection = crate::tls_sream::connect(self.provider.base().stream.clone())
            .await
            .map_err(|x| {
                tracing::error!(err =? x, "Failed establish the connection to the stream");
                AsyncTaskStopReason::NeedsRestart(NeedsRestart)
            });
        let connection = match connection {
            Ok(connection) => connection,
            Err(err) => {
                tracing::error!(err =? err, "Failed to connect to the stream, retrying...");
                self.sender
                    .send(ExternalUpdates::Metadata(MetadataUpdates::FailedToConnect))
                    .await
                    .map_err(|err| {
                        tracing::error!(err =? err, "Failed to send metadata update to the output");
                        AsyncTaskStopReason::FatalError(FatalError)
                    })?;
                return Err(AsyncTaskStopReason::NeedsRestart(NeedsRestart));
            }
        };
        self.sender
            .send(ExternalUpdates::Metadata(MetadataUpdates::TcpConnected))
            .await
            .map_err(|err| {
                tracing::error!(err =? err, "Failed to send metadata update to the output queue");
                AsyncTaskStopReason::FatalError(FatalError)
            })?;

        // handshake & process the stream
        let (auth_done_sender, auth_done_recv) = futures::channel::oneshot::channel();
        let mut process = self.runtime_handle.spawn(handle_stream_connection(
            connection,
            self.sender.clone(),
            self.command_reader.resubscribe(),
            session_token.clone(),
            self.provider.base().secret_provider.application_key.clone(),
            auth_done_sender,
        ));

        // wait for the handshake to complete
        if auth_done_recv.await.is_err() {
            process.abort();
            return Err(AsyncTaskStopReason::NeedsRestart(NeedsRestart));
        }

        // post-handshake start the heartbeat
        let mut heartbeat = self.runtime_handle.spawn(
            heartbeat_loop(self.hb.clone(), self.command_sender.clone()).map_err(|err| {
                tracing::error!("Heartbeat loop failed");
                AsyncTaskStopReason::FatalError(err)
            }),
        );

        // wait for the process or the heartbeat to finish
        let result = (&mut heartbeat).race(&mut process).await;
        heartbeat.abort();
        process.abort();

        // handle the result
        if let Ok(Err(AsyncTaskStopReason::FatalError(err))) = result {
            return Err(AsyncTaskStopReason::FatalError(err));
        }
        Err(AsyncTaskStopReason::NeedsRestart(NeedsRestart))
    }
}

async fn get_session_token(
    last_time_token_refreshed: &mut Option<(std::time::Instant, SessionToken)>,
    provider: &UnauthenticatedBetfairRpcProvider,
) -> Result<SessionToken, NeedsRestart> {
    let session_token = {
        let get_token = |provider: UnauthenticatedBetfairRpcProvider| async move {
            let res = provider
                .clone()
                .authenticate()
                .map_err(|err| {
                    tracing::error!(?err, "Failed to authenticate");
                    NeedsRestart
                })
                .await?;
            Ok(res.session_token().clone())
        };
        match last_time_token_refreshed {
            Some((time, token)) => {
                let max_allowed_time = std::time::Duration::from_secs(60 * 5); // 5 minutes
                let elapsed = time.elapsed();
                if elapsed > max_allowed_time {
                    *token = get_token(provider.clone()).await?;
                    *time = std::time::Instant::now();
                }
                token.clone()
            }
            tracker @ None => {
                let token = get_token(provider.clone()).await?;
                tracker.replace((std::time::Instant::now(), token.clone()));
                token
            }
        }
    };
    Ok(session_token)
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
                        .map_err(|_| AsyncTaskStopReason::FatalError(FatalError))?;
                }
                Err(err) => {
                    sender
                        .send(ExternalUpdates::Metadata(
                            MetadataUpdates::FailedToAuthenticate,
                        ))
                        .await
                        .map_err(|_| AsyncTaskStopReason::FatalError(FatalError))?;
                    return Err(AsyncTaskStopReason::NeedsRestart(err));
                }
            }
        }
        authenticated.send(()).map_err(|_| {
            tracing::error!("Failed to commuincate with the auth one-shot channel");
            AsyncTaskStopReason::NeedsRestart(NeedsRestart)
        })?;
    }
    tracing::info!("authenticated");

    // todo replace with custom Future impl
    loop {
        futures::select! {
            msg = command_reader.recv().fuse() => {
                match msg {
                    Ok(msg) => {
                        tracing::info!(msg = ?msg, "Sending message to stream");
                        connection.send(msg).await.map_err(|_| AsyncTaskStopReason::NeedsRestart(NeedsRestart))?;
                    }
                    Err(_) => {
                        return Err(AsyncTaskStopReason::FatalError(FatalError));
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
                            .map_err(|_| AsyncTaskStopReason::FatalError(FatalError))?;
                    }
                    _ => {
                        return Err(AsyncTaskStopReason::NeedsRestart(NeedsRestart));
                    },
                }
            }
        }
    }
}

pub async fn cache_loop(
    mut receiver: tokio::sync::mpsc::Receiver<ExternalUpdates<ResponseMessage>>,
    external_sender: tokio::sync::mpsc::Sender<ExternalUpdates<CacheEnabledMessages>>,
) -> Result<Never, FatalError> {
    let mut state = StreamStateTracker::new();
    while let Some(msg) = receiver.recv().await {
        match msg {
            ExternalUpdates::Layer(ResponseMessage::MarketChange(msg)) => {
                process_cachable_items(
                    &mut state,
                    msg.publish_time,
                    IncomingMessage::Market(msg),
                    &external_sender,
                )
                .await?;
            }
            ExternalUpdates::Layer(ResponseMessage::OrderChange(msg)) => {
                process_cachable_items(
                    &mut state,
                    msg.publish_time,
                    IncomingMessage::Order(msg),
                    &external_sender,
                )
                .await?;
            }
            ExternalUpdates::Layer(ResponseMessage::Connection(msg)) => {
                external_sender
                    .send(ExternalUpdates::Layer(
                        CacheEnabledMessages::ConnectionMessage(msg),
                    ))
                    .await
                    .map_err(|_| FatalError)?;
            }
            ExternalUpdates::Layer(ResponseMessage::Status(msg)) => {
                external_sender
                    .send(ExternalUpdates::Layer(CacheEnabledMessages::StatusMessage(
                        msg,
                    )))
                    .await
                    .map_err(|_| FatalError)?;
            }
            ExternalUpdates::Metadata(metadata) => {
                external_sender
                    .send(ExternalUpdates::Metadata(metadata))
                    .await
                    .map_err(|_| FatalError)?;
            }
        }
    }

    Err(FatalError)
}

async fn process_cachable_items<'a>(
    state: &mut StreamStateTracker,
    publish_time: Option<DateTime<chrono::Utc>>,
    updates: IncomingMessage,
    external_sender: &tokio::sync::mpsc::Sender<ExternalUpdates<CacheEnabledMessages>>,
) -> Result<(), FatalError> {
    let updates = state.calculate_updates(updates);

    let Some(updates) = updates else {
        return Ok(())
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
        .map_err(|_| FatalError)?;

    let Some(publish_time) = publish_time else {
        return Ok(())
    };

    state.clear_stale_cache(publish_time);

    Ok(())
}

async fn heartbeat_loop(
    hb: HeartbeatStrategy,
    command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
) -> Result<Never, FatalError> {
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
                            .map_err(|_| FatalError)?;
                    }
                    complete => break,
                }
            }
        }
    };

    Err(FatalError)
}
