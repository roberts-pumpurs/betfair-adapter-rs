use std::sync::Arc;
use std::time::Duration;

use betfair_adapter::{ApplicationKey, BetfairUrl, SessionToken};
use betfair_stream_types::request::heartbeat_message::HeartbeatMessage;
use betfair_stream_types::request::{authentication_message, RequestMessage};
use betfair_stream_types::response::status_message::StatusCode;
use betfair_stream_types::response::ResponseMessage;
use futures_concurrency::prelude::*;
use futures_util::{Future, FutureExt, SinkExt, Stream, StreamExt};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use tokio::sync::Notify;
use tokio_stream::wrappers::BroadcastStream;

use super::raw_stream_connection::{self};
use crate::provider::cache::tracker::{IncomingMessage, StreamStateTracker};
use crate::provider::primitives::{MarketBookCache, OrderBookCache};
use crate::StreamError;

/// Stream listener, processes results from socket, holds a stream which can hold order or market
/// book caches
#[derive(Debug)]
pub struct StreamListener {
    /// Send data to the underlying stream
    command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
    /// Notify whoever is interested in the updates
    output_queue: tokio::sync::broadcast::Sender<ExternalUpdates>,
    rng: SmallRng,
    connections_available: Option<i32>,
    connection_id: Option<String>,
    tracker: StreamStateTracker,
    application_key: ApplicationKey,
    session_token: SessionToken,
    status: Status,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Connected,
    /// Fatal error where the underlying TCP connection needs to be rest
    Disconnected,
    Authenticated,
}

#[derive(Debug, Clone)]
pub enum HeartbeatStrategy {
    None,
    Interval(Duration),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExternalUpdates {
    Market(Vec<MarketBookCache>),
    Order(Vec<OrderBookCache>),
}

impl StreamListener {
    pub async fn new(
        application_key: ApplicationKey,
        session_token: SessionToken,
        url: BetfairUrl<'static, betfair_adapter::Stream>,
        hb: HeartbeatStrategy,
    ) -> Result<
        (
            Arc<tokio::sync::RwLock<Self>>,
            impl Future<Output = ()>,
            tokio::sync::broadcast::Receiver<ExternalUpdates>,
        ),
        StreamError,
    > {
        let (command_sender, command_reader) = tokio::sync::broadcast::channel(100);
        let (output_sender, output_reader) = tokio::sync::broadcast::channel(100);
        let mut api = Self::new_with_commands(
            command_sender.clone(),
            output_sender,
            application_key,
            session_token,
        );
        api.tracker.update_unique_id(api.rng.gen());
        let api = Arc::new(tokio::sync::RwLock::new(api));

        let async_task = connect(url, command_reader, api.clone(), hb).await;
        Ok((api, async_task, output_reader))
    }

    fn new_with_commands(
        command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
        output_queue: tokio::sync::broadcast::Sender<ExternalUpdates>,
        application_key: ApplicationKey,
        session_token: SessionToken,
    ) -> StreamListener {
        Self {
            command_sender,
            connection_id: None,
            connections_available: None,
            tracker: StreamStateTracker::new(),
            output_queue,
            rng: SmallRng::from_entropy(),
            status: Status::Connected,
            application_key,
            session_token,
        }
    }

    async fn process_response_message(&mut self, msg: ResponseMessage) {
        let mut publish_time = None;
        let updates = match msg {
            ResponseMessage::Connection(_msg) => {
                // TODO store the _msg.connection_id and _msg.id, use it as Display properties of
                // the client
                self.status = Status::Connected;

                let authorization_message = authentication_message::AuthenticationMessage {
                    id: None,
                    session: self.session_token.0.expose_secret().clone(),
                    app_key: self.application_key.0.expose_secret().clone(),
                };
                let _ = self.send_message(RequestMessage::Authentication(authorization_message));
                None
            }
            ResponseMessage::MarketChange(msg) => {
                publish_time = msg.publish_time;
                self.tracker.calculate_updates(IncomingMessage::Market(msg))
            }
            ResponseMessage::OrderChange(msg) => {
                publish_time = msg.publish_time;
                self.tracker.calculate_updates(IncomingMessage::Order(msg))
            }
            ResponseMessage::StatusMessage(msg) => {
                self.handle_status_message(msg);
                None
            }
        };
        if let Some(updates) = updates {
            let update = match updates {
                crate::provider::cache::tracker::Updates::Market(msg) => {
                    ExternalUpdates::Market(msg.into_iter().cloned().collect())
                }
                crate::provider::cache::tracker::Updates::Order(msg) => {
                    ExternalUpdates::Order(msg.into_iter().cloned().collect())
                }
            };
            let _res = self.output_queue.send(update);
            if let Some(publish_time) = publish_time {
                self.tracker.clear_stale_cache(publish_time);
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn handle_status_message(
        &mut self,
        msg: betfair_stream_types::response::status_message::StatusMessage,
    ) {
        if let Some(true) = msg.connection_closed {
            self.status = Status::Disconnected;
        }
        if msg.error_code.is_some() {
            self.status = Status::Disconnected;
        }
        if let Some(StatusCode::Failure) = msg.status_code {
            self.status = Status::Disconnected;
        }

        if let Some(connections_available) = msg.connections_available {
            self.connections_available = Some(connections_available);
        }

        if let Some(connection_id) = msg.connection_id {
            self.connection_id = Some(connection_id);
        }

        if self.status != Status::Disconnected {
            self.status = Status::Authenticated;
        }
    }

    /// Send a message to the Betfair stream.
    /// It will generate a unique id for the message
    pub fn send_message(
        &mut self,
        mut msg: RequestMessage,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        let id = self.rng.gen();
        msg.set_id(id);
        self.tracker.update_unique_id(id);
        self.command_sender.send(msg).map(|_x| ())
    }
}

async fn connect(
    url: BetfairUrl<'static, betfair_adapter::Stream>,
    command_reader: tokio::sync::broadcast::Receiver<RequestMessage>,
    api: Arc<tokio::sync::RwLock<StreamListener>>,
    hb: HeartbeatStrategy,
) -> impl Future<Output = ()> {
    let reconnect_notifier = Arc::new(tokio::sync::Notify::new());

    let url = url.clone();

    async move {
        let async_task = loop {
            if let Ok(conn) = connect_and_process(
                url.clone(),
                command_reader.resubscribe(),
                api.clone(),
                hb.clone(),
            )
            .await
            {
                break conn;
            }
            // Sleep for 5 seconds if we cannot connect to the stream
            tokio::time::sleep(Duration::from_secs(5)).await;
            tracing::warn!("Reconnecting to the betfair stream during initial setup");
        };
        let mut async_task = Box::pin(async_task);

        loop {
            let mut connection = None;
            let _ = async {
                // Reconnect if the reconnect_notifier is notified
                reconnect_notifier.notified().await;
                tracing::info!("Reconnecting to the stream!");
                let new_connection = connect_and_process(
                    url.clone(),
                    command_reader.resubscribe(),
                    api.clone(),
                    hb.clone(),
                )
                .await;
                if let Ok(new_connection) = new_connection {
                    connection = Some(new_connection);
                } else {
                    tracing::error!("Error reconnecting to the stream!");
                }
                Ok(())
            }
            .race(&mut async_task)
            .race(create_reconnect_loop(reconnect_notifier.clone(), api.clone()).map(|_| Ok(())))
            .await;

            // Pin the new connection future if it's been set
            if let Some(connection) = connection {
                async_task = Box::pin(connection);
            }
        }
    }
}

async fn connect_and_process(
    url: BetfairUrl<'_, betfair_adapter::Stream>,
    command_reader: tokio::sync::broadcast::Receiver<RequestMessage>,
    api: Arc<tokio::sync::RwLock<StreamListener>>,
    hb: HeartbeatStrategy,
) -> Result<impl Future<Output = Result<(), StreamError>>, StreamError> {
    let (write_to_socket, read) = raw_stream_connection::connect(
        url.clone(),
        BroadcastStream::new(command_reader.resubscribe()),
    )
    .await?;
    let async_task = write_to_socket
        .race(create_heartbeat_loop(hb, api.clone()).map(|_| Ok(())))
        .race(create_write_loop(read, api.clone()).map(|_| Ok(())))
        .fuse();

    Ok(async_task)
}

/// Send a heartbeat message to the Betfair stream every `period`
#[tracing::instrument(skip(api_c))]
fn create_heartbeat_loop(
    hb: HeartbeatStrategy,
    api_c: Arc<tokio::sync::RwLock<StreamListener>>,
) -> impl Future<Output = ()> {
    async move {
        match hb {
            HeartbeatStrategy::None => loop {
                std::future::pending::<()>().await;
            },
            HeartbeatStrategy::Interval(period) => loop {
                let mut interval = tokio::time::interval(period);
                interval.reset();
                loop {
                    interval.tick().await;
                    let mut api = api_c.write().await;

                    // Only send a heartbeat if we are connected
                    if api.status != Status::Authenticated {
                        continue;
                    }

                    let _ =
                        api.send_message(RequestMessage::Heartbeat(HeartbeatMessage { id: None }));
                    drop(api)
                }
            },
        };
    }
}

/// Read from the Betfair stream and process the messages, send them away!
#[tracing::instrument(skip(api_c, read))]
fn create_write_loop(
    read: impl Stream<Item = Result<ResponseMessage, StreamError>>,
    api_c: Arc<tokio::sync::RwLock<StreamListener>>,
) -> impl Future<Output = ()> {
    async move {
        tokio::pin!(read);
        while let Some(msg) = read.next().await {
            tracing::info!("GOT MSG!{:#?}", msg);
            match msg {
                Ok(msg) => {
                    tracing::debug!(msg = ?msg, "Received message");
                    let mut api = api_c.write().await;
                    api.process_response_message(msg).await;
                    drop(api)
                }
                Err(err) => {
                    let mut api = api_c.write().await;
                    api.status = Status::Disconnected;
                    drop(api);

                    tracing::error!(err = ?err, "Error reading from stream! Disconnecting!");
                }
            }
        }
    }
}

/// TODO list for reconnecting to the stream:
/// - [ ] **Exponential Backoff**: Implement an exponential backoff strategy for retries.
/// - [ ] **Jitter**: Add jitter to the retry intervals.
/// - [ ] **Retry Limits**: Set a maximum number of retries.
/// - [ ] **Circuit Breaker Pattern**: Use a circuit breaker for failing services.
/// - [ ] **Status Code Handling**: Handle different HTTP status codes appropriately.
/// - [ ] **Logging and Monitoring**: Log retries and monitor the API service.
/// - [ ] **Configurability**: Make retry parameters configurable.
/// - [ ] **Network Checks**: Check network availability before retrying.
/// - [ ] **Timeouts**: Implement request timeouts.
/// - [ ] **Async and Non-Blocking**: Make reconnection logic asynchronous and non-blocking.
/// - [ ] **User Notification**: Inform users about reconnection attempts.
#[tracing::instrument(skip(api_c, reconnect_notifier))]
fn create_reconnect_loop(
    reconnect_notifier: Arc<Notify>,
    api_c: Arc<tokio::sync::RwLock<StreamListener>>,
) -> impl Future<Output = ()> {
    async move {
        // TODO move these intervals to persistent state (api_c) maybe?
        // Give 20 seconds for the initial stream to connect before we start monitoring for
        // reconnections
        tokio::time::sleep(Duration::from_secs(20)).await;
        loop {
            // sleep for 10 sec
            tokio::time::sleep(Duration::from_secs(10)).await;

            let api = api_c.read().await;
            if api.status == Status::Disconnected {
                tracing::info!("Restart the whole stream");
                reconnect_notifier.notify_one();
            }
            drop(api);
        }
    }
}
