use std::borrow::Cow;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use betfair_adapter::betfair_types::customer_strategy_ref::CustomerStrategyRef;
use betfair_adapter::betfair_types::types::sports_aping::MarketId;
use betfair_adapter::{ApplicationKey, BetfairUrl, SessionToken};
use betfair_stream_types::request::heartbeat_message::HeartbeatMessage;
use betfair_stream_types::request::market_subscription_message::{
    MarketDataFilter, MarketFilter, MarketSubscriptionMessage,
};
use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::response::market_change_message::{MarketChange, MarketChangeMessage};
use betfair_stream_types::response::order_change_message::{OrderChangeMessage, OrderMarketChange};
use betfair_stream_types::response::status_message::StatusCode;
use betfair_stream_types::response::{
    market_change_message, order_change_message, ResponseMessage,
};
use futures_concurrency::prelude::*;
use futures_util::{Future, SinkExt, Stream};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use tokio_stream::wrappers::ReceiverStream;

use super::raw_stream_connection::RawStreamConnection;
use crate::provider::cache::tracker::{IncomingMessage, StreamStateTracker};
use crate::provider::primitives::{MarketBookCache, OrderBookCache};
use crate::StreamError;

/// Stream listener, processes results from socket, holds a stream which can hold order or market
/// book caches
#[derive(Debug)]
pub struct StreamListener {
    command_sender: tokio::sync::mpsc::Sender<RequestMessage>,
    output_queue: futures::channel::mpsc::UnboundedSender<ExternalUpdates>,
    rng: SmallRng,
    connections_available: Option<i32>,
    connection_id: Option<String>,
    tracker: StreamStateTracker,
    status: Status,
}

#[derive(Debug)]
pub enum Status {
    Connected,
    Disconnected,
}

#[derive(Debug)]
pub enum HeartbeatStrategy {
    None,
    Interval(Duration),
}

pub enum ExternalUpdates {
    Market(Vec<MarketBookCache>),
    Order(Vec<OrderBookCache>),
}

impl StreamListener {
    #[tracing::instrument(skip(application_key, session_token, url), err)]
    pub async fn new<'a>(
        application_key: Cow<'a, ApplicationKey>,
        session_token: Cow<'a, SessionToken>,
        url: BetfairUrl<'a, betfair_adapter::Stream>,
        hb: HeartbeatStrategy,
    ) -> Result<
        (
            Arc<tokio::sync::RwLock<Self>>,
            Pin<Box<dyn Future<Output = Result<(), StreamError>> + Send>>,
            futures::channel::mpsc::UnboundedReceiver<ExternalUpdates>,
        ),
        StreamError,
    > {
        let mut stream_wrapper = RawStreamConnection::new(application_key, session_token);
        let (command_sender, command_reader) = tokio::sync::mpsc::channel(5);
        let (output_sender, output_reader) = futures::channel::mpsc::unbounded();
        let mut api = Self::new_with_commands(command_sender.clone(), output_sender);
        let unique_id = api.rng.gen::<i32>();
        api.tracker.update_unique_id(unique_id);
        let api = Arc::new(tokio::sync::RwLock::new(api));

        let host = url.url().host_str().unwrap();
        let is_tls = url.url().scheme() == "https";
        let port = url.url().port().unwrap_or(if is_tls { 443 } else { 80 });
        let socket_addr = tokio::net::lookup_host((host, port)).await.unwrap().next();
        let domain = url.url().domain();

        match (is_tls, domain, socket_addr) {
            (true, Some(domain), Some(socket_addr)) => {
                let (write_to_wire, read) = stream_wrapper
                    .connect_tls(
                        unique_id,
                        domain,
                        socket_addr,
                        ReceiverStream::new(command_reader),
                    )
                    .await
                    .unwrap();

                {
                    let mut api = api.write().await;
                    api.status = Status::Connected;
                }
                let async_task_2 = process(api.clone(), read, hb);
                let async_task = Box::pin(write_to_wire.race(async_task_2));
                Ok((api, async_task, output_reader))
            }
            (false, _, Some(socket_addr)) => {
                let (write_to_wire, read) = stream_wrapper
                    .connect_non_tls(unique_id, socket_addr, ReceiverStream::new(command_reader))
                    .await
                    .unwrap();
                {
                    let mut api = api.write().await;
                    api.status = Status::Connected;
                }
                let async_task_2 = process(api.clone(), read, hb);
                let async_task = Box::pin(write_to_wire.race(async_task_2));
                Ok((api, async_task, output_reader))
            }
            _ => Err(StreamError::MisconfiguredStreamURL),
        }
    }

    fn new_with_commands(
        command_sender: tokio::sync::mpsc::Sender<RequestMessage>,
        output_queue: futures::channel::mpsc::UnboundedSender<ExternalUpdates>,
    ) -> StreamListener {
        Self {
            command_sender,
            connection_id: None,
            connections_available: None,
            tracker: StreamStateTracker::new(),
            output_queue,
            rng: SmallRng::from_entropy(),
            status: Status::Disconnected,
        }
    }

    fn replace_queue(&mut self) -> futures::channel::mpsc::UnboundedReceiver<ExternalUpdates> {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        self.output_queue = tx;
        rx
    }

    // TODO separate out this logic into a separate module, maybe a wrapper around the stream listener
    // pub fn subscribe_to_market(
    //     &mut self,
    //     market_id: MarketId,
    // ) -> futures::channel::mpsc::UnboundedReceiver<MarketChangeMessage> {
    //     let (tx, _rx) = futures::channel::mpsc::unbounded();
    //     self.market_tracker
    //         .market_subscriptions
    //         .insert(market_id, tx);

    //     let all_market_ids = self
    //         .market_tracker
    //         .market_subscriptions
    //         .keys()
    //         .cloned()
    //         .collect::<Vec<_>>();

    //     let req = RequestMessage::MarketSubscription(MarketSubscriptionMessage {
    //         id: Some(self.rng.gen()),
    //         clk: None,         // empty to reset the clock
    //         initial_clk: None, // empty to reset the clock
    //         segmentation_enabled: Some(true),
    //         heartbeat_ms: Some(500),
    //         market_filter: Some(Box::new(MarketFilter {
    //             country_codes: None,
    //             betting_types: None,
    //             turn_in_play_enabled: None,
    //             market_types: None,
    //             venues: None,
    //             market_ids: Some(all_market_ids),
    //             event_type_ids: None,
    //             event_ids: None,
    //             bsp_market: None,
    //             race_types: None,
    //         })),
    //         conflate_ms: None,
    //         market_data_filter: Some(Box::new(MarketDataFilter {
    //             ladder_levels: None,
    //             fields: None,
    //         })),
    //     });
    //     self.send_message(req);

    //     // rx
    //     todo!()
    // }

    // pub fn unsubscribe_from_market(&mut self, market_ids: MarketId) -> Option<MarketChange> {
    //     let _value = self.market_tracker.market_subscriptions.remove(&market_ids);
    //     let value = self.market_tracker.market_state.remove(&market_ids);

    //     if self.market_tracker.market_subscriptions.is_empty() {
    //         self.unsubscribe_from_all_markets();
    //     }

    //     value
    // }

    // /// https://forum.developer.betfair.com/forum/sports-exchange-api/exchange-api/34555-stream-api-unsubscribe-from-all-markets
    // fn unsubscribe_from_all_markets(&mut self) {
    //     let market_that_does_not_exist = MarketId("1.23456789".to_string());
    //     let req = RequestMessage::MarketSubscription(MarketSubscriptionMessage {
    //         id: Some(self.rng.gen()),
    //         segmentation_enabled: Some(true),
    //         clk: self.market_tracker.latest_market_clk.clone(),
    //         heartbeat_ms: Some(500),
    //         initial_clk: self.market_tracker.initial_market_clk.clone(),
    //         market_filter: Some(Box::new(MarketFilter {
    //             country_codes: None,
    //             betting_types: None,
    //             turn_in_play_enabled: None,
    //             market_types: None,
    //             venues: None,
    //             market_ids: Some(vec![market_that_does_not_exist]),
    //             event_type_ids: None,
    //             event_ids: None,
    //             bsp_market: None,
    //             race_types: None,
    //         })),
    //         conflate_ms: None,
    //         market_data_filter: Some(Box::new(MarketDataFilter {
    //             ladder_levels: None,
    //             fields: None,
    //         })),
    //     });
    //     self.send_message(req);
    // }

    async fn process_response_message(&mut self, msg: ResponseMessage) {
        let mut publish_time = None;
        let updates = match msg {
            ResponseMessage::Connection(msg) => {
                tracing::debug!(msg = ?msg, "Received connection message");
                None
            }
            ResponseMessage::MarketChange(msg) => {
                publish_time = msg.publish_time.clone();
                self.tracker.calculate_updates(IncomingMessage::Market(msg))
            }
            ResponseMessage::OrderChange(msg) => {
                publish_time = msg.publish_time.clone();
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
            let _res = self.output_queue.feed(update).await;
            if let Some(publish_time) = publish_time {
                self.tracker.clear_stale_cache(publish_time);
            }
        }
    }

    fn handle_status_message(
        &mut self,
        msg: betfair_stream_types::response::status_message::StatusMessage,
    ) {
        if let Some(true) = msg.connection_closed {
            self.status = Status::Disconnected;
        }
        if let Some(_) = msg.error_code {
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
    }

    fn send_message(&mut self, mut msg: RequestMessage) {
        let id = self.rng.gen();
        msg.set_id(id);
        self.tracker.update_unique_id(id);
        self.command_sender.try_send(msg).expect("Unable to send message to the underlying stream!");
    }
}

async fn process(
    api: Arc<tokio::sync::RwLock<StreamListener>>,
    read: impl futures_util::Stream<Item = Result<ResponseMessage, StreamError>>,
    hb: HeartbeatStrategy,
) -> Result<(), StreamError> {
    let hb_loop = create_heartbeat_loop(hb, api.clone()).await;
    let write_loop = create_write_loop(read, api.clone()).await;
    hb_loop.race(write_loop).await;

    Err(StreamError::StreamProcessorMalfunction)
}

/// Send a heartbeat message to the Betfair stream every `period`
async fn create_heartbeat_loop(
    hb: HeartbeatStrategy,
    api_c: Arc<tokio::sync::RwLock<StreamListener>>,
) -> impl Future<Output = ()> {
    async move {
        match hb {
            HeartbeatStrategy::None => loop {
                std::future::pending::<()>().await;
            },
            HeartbeatStrategy::Interval(period) => {
                let mut interval = tokio::time::interval(period);
                interval.reset();
                loop {
                    interval.tick().await;
                    let mut api = api_c.write().await;
                    api.send_message(RequestMessage::Heartbeat(HeartbeatMessage { id: None }));
                    drop(api)
                }
            }
        };
    }
}

/// Read from the Betfair stream and process the messages, send them away!
async fn create_write_loop(
    read: impl Stream<Item = Result<ResponseMessage, StreamError>>,
    api_c: Arc<tokio::sync::RwLock<StreamListener>>,
) -> impl Future<Output = ()> {
    use futures_util::StreamExt;

    async move {
        tokio::pin!(read);
        while let Some(msg) = read.next().await {
            match msg {
                Ok(msg) => {
                    tracing::debug!(msg = ?msg, "Received message");
                    let mut api = api_c.write().await;
                    api.process_response_message(msg);
                    drop(api)
                }
                Err(err) => {
                    tracing::error!(err = ?err, "Error reading from stream");
                }
            }
        }
    }
}
