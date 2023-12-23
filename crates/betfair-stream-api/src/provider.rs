use std::borrow::Cow;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use betfair_adapter::betfair_types::customer_strategy_ref::CustomerStrategyRef;
use betfair_adapter::betfair_types::types::sports_aping::MarketId;
use betfair_adapter::{ApplicationKey, BetfairUrl, SessionToken};
use betfair_stream_types::request::heartbeat_message::HeartbeatMessage;
use betfair_stream_types::request::market_subscription_message::MarketFilter;
use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::response::market_change_message::{MarketChange, MarketChangeMessage};
use betfair_stream_types::response::order_change_message::{OrderChangeMessage, OrderMarketChange};
use betfair_stream_types::response::ResponseMessage;
use futures_concurrency::prelude::*;
use futures_util::Future;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use tokio_stream::wrappers::ReceiverStream;

use crate::raw_stream::RawStream;
use crate::StreamError;

#[derive(Debug)]
pub struct StreamAPIProvider {
    pub command_sender:  tokio::sync::mpsc::Sender<RequestMessage>,
    pub rng: SmallRng,
    pub market_tracker: MarketTracker,
    pub order_tracker: OrderTracker,
}

#[derive(Debug)]
pub enum HeartbeatStrategy {
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
            Arc<std::sync::RwLock<Self>>,
            Pin<Box<dyn Future<Output = Result<(), StreamError>> + Send>>,
        ),
        StreamError,
    > {
        let mut stream_wrapper = RawStream::new(application_key, session_token);
        let (command_sender, command_reader) = tokio::sync::mpsc::channel(5);
        let api = Arc::new(std::sync::RwLock::new(Self::new_with_commands(
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
                let async_task_2 = process(api.clone(), read, hb);
                let async_task = Box::pin(write_to_wire.race(async_task_2));
                Ok((api, async_task))
            }
            (false, _, Some(socket_addr)) => {
                let (write_to_wire, read) = stream_wrapper
                    .connect_non_tls(socket_addr, ReceiverStream::new(command_reader))
                    .await
                    .unwrap();
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
            market_tracker: MarketTracker {
                market_subscriptions: HashMap::new(),
                market_state: HashMap::new(),
                market_filter: MarketFilter::default(),
                initial_market_clk: None,
                latest_market_clk: None,
            },
            order_tracker: OrderTracker {
                order_subscriptions: HashMap::new(),
                order_state: HashMap::new(),
                order_filter: MarketFilter::default(),
                initial_order_clk: None,
                latest_order_clk: None,
            },
        }
    }

    pub fn subscribe_to_markets(&mut self, _market_ids: &[MarketId]) {
        // TODO update internal state
        // TODO create a new subscription
        // TODO send the subscription to the stream via the command_sender
        unimplemented!()
    }

    pub fn unsubscribe_from_markets(&mut self, _market_ids: &[MarketId]) {
        unimplemented!()
    }

    pub fn unsubscribe_from_all_markets(&mut self) {
        unimplemented!()
    }

    fn process_response_message(&mut self, msg: ResponseMessage) {
        match msg {
            ResponseMessage::Connection(_) => todo!(),
            ResponseMessage::MarketChange(_) => todo!(),
            ResponseMessage::OrderChange(_) => todo!(),
            ResponseMessage::StatusMessage(_) => todo!(),
        }
    }

    fn send_message(&mut self, mut msg: RequestMessage) {
        msg.set_id(self.rng.gen());
        self.command_sender.try_send(msg).unwrap();
    }
}

#[derive(Debug)]
struct MarketTracker {
    market_subscriptions:
        HashMap<MarketId, futures::channel::mpsc::UnboundedSender<MarketChangeMessage>>,
    market_state: HashMap<MarketId, MarketChange>,
    market_filter: MarketFilter,
    initial_market_clk: Option<String>,
    latest_market_clk: Option<String>,
}

#[derive(Debug)]
struct OrderTracker {
    order_subscriptions:
        HashMap<CustomerStrategyRef, futures::channel::mpsc::UnboundedSender<OrderChangeMessage>>,
    order_state: HashMap<CustomerStrategyRef, OrderMarketChange>,
    order_filter: MarketFilter,
    initial_order_clk: Option<String>,
    latest_order_clk: Option<String>,
}

async fn process(
    api: Arc<std::sync::RwLock<StreamAPIProvider>>,
    read: impl futures_util::Stream<Item = Result<ResponseMessage, StreamError>>,
    hb: HeartbeatStrategy,
) -> Result<(), StreamError> {
    use futures_util::{FutureExt, StreamExt};
    futures::pin_mut!(read);

    let mut hb_interval = match hb {
        HeartbeatStrategy::Interval(period) => tokio::time::interval(period),
    };
    hb_interval.reset();

    loop {
        tracing::info!("PROCESSING LOOP");
        tokio::select! {
            _ = hb_interval.tick() => {
                tracing::info!("Sending heartbeat");
                let mut api = api.write().unwrap();
                api.send_message(RequestMessage::Heartbeat(HeartbeatMessage{
                    id: None,
                }));
                drop(api)
            }
            // _ = &mut res => {
            //     tracing::info!("Sending heartbeat");
            //     let mut api = api.write().unwrap();
            //     api.send_message(RequestMessage::Heartbeat(HeartbeatMessage{
            //         id: None,
            //     }));
            //     drop(api)
            // }
            // msg = read.next() => {
            //     tracing::info!("Received message");
            //     match msg {
            //         Some(Ok(msg)) => {
            //             tracing::debug!(msg = ?msg, "Received message");
            //             let mut api = api.write().unwrap();
            //             api.process_response_message(msg);
            //             drop(api)
            //         }
            //         Some(Err(err)) => {
            //             tracing::error!(err = ?err, "Error reading from stream");
            //             break
            //         }
            //         None => {
            //             tracing::error!("Stream closed");
            //             break
            //         }
            //     }
            // }
        }
    }

    // TODO change internal state to disconnected
    Ok(())
}
