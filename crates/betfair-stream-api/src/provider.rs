mod available_cache;
mod market_book_cache;
mod orderbook_cache;
mod orderbook_runner_cache;
mod runner_book_cache;

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
use betfair_stream_types::response::ResponseMessage;
use futures_concurrency::prelude::*;
use futures_util::{Future, Stream};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use tokio_stream::wrappers::ReceiverStream;

use crate::raw_stream::RawStream;
use crate::StreamError;

#[derive(Debug)]
pub struct StreamAPIProvider {
    command_sender: tokio::sync::mpsc::Sender<RequestMessage>,
    rng: SmallRng,
    market_tracker: MarketTracker,
    order_tracker: OrderTracker,
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

impl StreamAPIProvider {
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
        ),
        StreamError,
    > {
        let mut stream_wrapper = RawStream::new(application_key, session_token);
        let (command_sender, command_reader) = tokio::sync::mpsc::channel(5);
        let api = Arc::new(tokio::sync::RwLock::new(Self::new_with_commands(
            command_sender.clone(),
        )));

        let host = url.url().host_str().unwrap();
        let is_tls = url.url().scheme() == "https";
        let port = url.url().port().unwrap_or(if is_tls { 443 } else { 80 });
        let socket_addr = tokio::net::lookup_host((host, port)).await.unwrap().next();
        let domain = url.url().domain();

        match (is_tls, domain, socket_addr) {
            (true, Some(domain), Some(socket_addr)) => {
                let (write_to_wire, read) = stream_wrapper
                    .connect_tls(domain, socket_addr, ReceiverStream::new(command_reader))
                    .await
                    .unwrap();

                {
                    let mut api = api.write().await;
                    api.status = Status::Connected;
                }
                let async_task_2 = process(api.clone(), read, hb);
                let async_task = Box::pin(write_to_wire.race(async_task_2));
                Ok((api, async_task))
            }
            (false, _, Some(socket_addr)) => {
                let (write_to_wire, read) = stream_wrapper
                    .connect_non_tls(socket_addr, ReceiverStream::new(command_reader))
                    .await
                    .unwrap();
                {
                    let mut api = api.write().await;
                    api.status = Status::Connected;
                }
                let async_task_2 = process(api.clone(), read, hb);
                let async_task = Box::pin(write_to_wire.race(async_task_2));
                Ok((api, async_task))
            }
            _ => Err(StreamError::MisconfiguredStreamURL),
        }
    }

    fn new_with_commands(
        command_sender: tokio::sync::mpsc::Sender<RequestMessage>,
    ) -> StreamAPIProvider {
        Self {
            command_sender,
            rng: SmallRng::from_entropy(),
            status: Status::Disconnected,
            market_tracker: MarketTracker {
                market_subscriptions: HashMap::new(),
                market_state: HashMap::new(),
                initial_market_clk: None,
                latest_market_clk: None,
            },
            order_tracker: OrderTracker {
                order_subscriptions: HashMap::new(),
                order_state: HashMap::new(),
                initial_order_clk: None,
                latest_order_clk: None,
            },
        }
    }

    pub fn subscribe_to_market(
        &mut self,
        market_id: MarketId,
    ) -> futures::channel::mpsc::UnboundedReceiver<MarketChangeMessage> {
        let (tx, _rx) = futures::channel::mpsc::unbounded();
        self.market_tracker
            .market_subscriptions
            .insert(market_id, tx);

        let all_market_ids = self
            .market_tracker
            .market_subscriptions
            .keys()
            .cloned()
            .collect::<Vec<_>>();

        let req = RequestMessage::MarketSubscription(MarketSubscriptionMessage {
            id: Some(self.rng.gen()),
            clk: None,         // empty to reset the clock
            initial_clk: None, // empty to reset the clock
            segmentation_enabled: Some(true),
            heartbeat_ms: Some(500),
            market_filter: Some(Box::new(MarketFilter {
                country_codes: None,
                betting_types: None,
                turn_in_play_enabled: None,
                market_types: None,
                venues: None,
                market_ids: Some(all_market_ids),
                event_type_ids: None,
                event_ids: None,
                bsp_market: None,
                race_types: None,
            })),
            conflate_ms: None,
            market_data_filter: Some(Box::new(MarketDataFilter {
                ladder_levels: None,
                fields: None,
            })),
        });
        self.send_message(req);

        // rx
        todo!()
    }

    pub fn unsubscribe_from_market(&mut self, market_ids: MarketId) -> Option<MarketChange> {
        let _value = self.market_tracker.market_subscriptions.remove(&market_ids);
        let value = self.market_tracker.market_state.remove(&market_ids);

        if self.market_tracker.market_subscriptions.is_empty() {
            self.unsubscribe_from_all_markets();
        }

        value
    }

    /// https://forum.developer.betfair.com/forum/sports-exchange-api/exchange-api/34555-stream-api-unsubscribe-from-all-markets
    fn unsubscribe_from_all_markets(&mut self) {
        let market_that_does_not_exist = MarketId("1.23456789".to_string());
        let req = RequestMessage::MarketSubscription(MarketSubscriptionMessage {
            id: Some(self.rng.gen()),
            segmentation_enabled: Some(true),
            clk: self.market_tracker.latest_market_clk.clone(),
            heartbeat_ms: Some(500),
            initial_clk: self.market_tracker.initial_market_clk.clone(),
            market_filter: Some(Box::new(MarketFilter {
                country_codes: None,
                betting_types: None,
                turn_in_play_enabled: None,
                market_types: None,
                venues: None,
                market_ids: Some(vec![market_that_does_not_exist]),
                event_type_ids: None,
                event_ids: None,
                bsp_market: None,
                race_types: None,
            })),
            conflate_ms: None,
            market_data_filter: Some(Box::new(MarketDataFilter {
                ladder_levels: None,
                fields: None,
            })),
        });
        self.send_message(req);
    }

    fn process_response_message(&mut self, msg: ResponseMessage) {
        match msg {
            ResponseMessage::Connection(msg) => {
                tracing::debug!(msg = ?msg, "Received connection message");
            }
            ResponseMessage::MarketChange(msg) => {
                self.handle_market_change_message(msg);
            }
            ResponseMessage::OrderChange(msg) => {
                self.handle_order_change_message(msg);
            }
            ResponseMessage::StatusMessage(msg) => {
                self.handle_status_message(msg);
            }
        }
    }

    fn handle_order_change_message(&mut self, _msg: OrderChangeMessage) {}
    fn handle_market_change_message(&mut self, msg: MarketChangeMessage) {
        match msg.clock {
            Some(ref clk) => {
                self.market_tracker.latest_market_clk = Some(clk.clone());
            }
            None => {}
        }
        match msg.initial_clk {
            Some(ref initial_clk) => {
                self.market_tracker.initial_market_clk = Some(initial_clk.clone());
            }
            None => {}
        }
        match msg.0.data {
            Some(mcs) => {
                for mc in mcs {
                    let market_id = mc.id.clone();
                    let Some(_market_id) = market_id else {
                        continue;
                    };

                    todo!("https://github.com/betcode-org/betfair/blob/7084866a50e6e62ad1f71039507b098fa5249822/betfairlightweight/streaming/stream.py line 190")
                    // let full_image = mc.img.unwrap_or(false);

                    // let market_change = self
                    //     .market_tracker
                    //     .market_state
                    //     .get_mut(&market_id);

                    // match (market_change, full_image) {
                    //     (Some(val), false) => {

                    //     }
                    //     (_, true) | (None, _)  => {
                    //         // replace the market change
                    //         self.market_tracker.market_state.insert(market_id, mc.clone());
                    //     }
                    // }

                    // let market_change = self
                    //     .market_tracker
                    //     .market_state
                    //     .entry(market_id)
                    //     .or_insert_with(|| mc.clone());

                    // if mc.img.unwrap_or(false) {
                    //     // replace the market change
                    //     *market_change = mc.clone();
                    // } else {
                    //     // partial update
                    //     // if mc.con
                    //     // market_change.tv = mc.tv;
                    // }

                    // if let Some(tx) = self.market_tracker.market_subscriptions.get(&market_id) {
                    //     tx.unbounded_send(mc).unwrap();
                    // }
                }
            }
            None => {}
        }
    }

    fn handle_status_message(
        &mut self,
        msg: betfair_stream_types::response::status_message::StatusMessage,
    ) {
        match msg.connection_closed {
            Some(true) => {
                self.status = Status::Disconnected;
            }
            _ => {}
        }
        match msg.error_code {
            Some(_) => {
                self.status = Status::Disconnected;
            }
            _ => {}
        }
        match msg.status_code {
            Some(StatusCode::Failure) => {
                self.status = Status::Disconnected;
            }
            _ => {}
        }
    }

    fn send_message(&mut self, mut msg: RequestMessage) {
        msg.set_id(self.rng.gen());
        self.command_sender.try_send(msg).unwrap();
    }
}

#[derive(Debug)]
struct MarketTracker {
    market_subscriptions: HashMap<MarketId, futures::channel::mpsc::UnboundedSender<MarketChange>>,
    market_state: HashMap<MarketId, MarketChange>,
    initial_market_clk: Option<String>,
    latest_market_clk: Option<String>,
}

#[derive(Debug)]
struct OrderTracker {
    order_subscriptions:
        HashMap<CustomerStrategyRef, futures::channel::mpsc::UnboundedSender<OrderChangeMessage>>,
    order_state: HashMap<CustomerStrategyRef, OrderMarketChange>,
    initial_order_clk: Option<String>,
    latest_order_clk: Option<String>,
}

async fn process(
    api: Arc<tokio::sync::RwLock<StreamAPIProvider>>,
    read: impl futures_util::Stream<Item = Result<ResponseMessage, StreamError>>,
    hb: HeartbeatStrategy,
) -> Result<(), StreamError> {
    let hb_loop = create_heartbeat_loop(hb, api.clone()).await;
    let write_loop = create_write_loop(read, api.clone()).await;
    hb_loop.race(write_loop).await;

    Err(StreamError::StreamProcessorMalfunction)
}

async fn create_heartbeat_loop(
    hb: HeartbeatStrategy,
    api_c: Arc<tokio::sync::RwLock<StreamAPIProvider>>,
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

async fn create_write_loop(
    read: impl Stream<Item = Result<ResponseMessage, StreamError>>,
    api_c: Arc<tokio::sync::RwLock<StreamAPIProvider>>,
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
