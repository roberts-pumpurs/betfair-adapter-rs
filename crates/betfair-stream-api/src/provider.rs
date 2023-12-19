use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use betfair_adapter::betfair_types::customer_strategy_ref::CustomerStrategyRef;
use betfair_adapter::betfair_types::types::sports_aping::MarketId;
use betfair_adapter::{ApplicationKey, BetfairUrl, SessionToken};
use betfair_stream_types::request::market_subscription_message::MarketFilter;
use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::response::market_change_message::{MarketChange, MarketChangeMessage};
use betfair_stream_types::response::order_change_message::{OrderChangeMessage, OrderMarketChange};
use betfair_stream_types::response::ResponseMessage;
use futures_util::Future;

use crate::raw_stream::RawStream;
use crate::StreamError;

pub struct StreamAPIProvider {
    command_sender: futures::channel::mpsc::UnboundedSender<RequestMessage>,
    market_tracker: MarketTracker,
    order_tracker: OrderTracker,
}

impl StreamAPIProvider {
    pub async fn new<'a>(
        application_key: Cow<'a, ApplicationKey>,
        session_token: Cow<'a, SessionToken>,
        url: BetfairUrl<'a, betfair_adapter::Stream, url::Url>,
    ) -> Result<
        (
            Arc<std::sync::RwLock<Self>>,
            Box<dyn Future<Output = Result<(), StreamError>>>,
            Box<dyn Future<Output = ()>>,
        ),
        StreamError,
    > {
        let mut stream_wrapper = RawStream::new(application_key, session_token);
        let (command_sender, command_reader) = futures::channel::mpsc::unbounded();
        let api = Arc::new(std::sync::RwLock::new(Self::new_with_commands(
            command_sender,
        )));

        let host = url.url().host_str().unwrap();
        let is_tls = url.url().scheme() == "https";
        let port = url.url().port().unwrap_or(if is_tls { 443 } else { 80 });
        let socket_addr = tokio::net::lookup_host((host, port)).await.unwrap().next();
        let domain = url.url().domain();

        match (is_tls, domain, socket_addr) {
            (true, Some(domain), Some(socket_addr)) => {
                let (async_task, read) = stream_wrapper
                    .connect_tls(domain, socket_addr, command_reader)
                    .await
                    .unwrap();
                let async_task_2 = process(api.clone(), read);
                Ok((api, Box::new(async_task), Box::new(async_task_2)))
            }
            (false, _, Some(socket_addr)) => {
                let (async_task, read) = stream_wrapper
                    .connect_non_tls(socket_addr, command_reader)
                    .await
                    .unwrap();
                let async_task_2 = process(api.clone(), read);
                Ok((api, Box::new(async_task), Box::new(async_task_2)))
            }
            _ => Err(StreamError::MisconfiguredStreamURL),
        }
    }

    fn new_with_commands(
        command_sender: futures::channel::mpsc::UnboundedSender<RequestMessage>,
    ) -> StreamAPIProvider {
        Self {
            command_sender,
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
}

struct MarketTracker {
    market_subscriptions:
        HashMap<MarketId, futures::channel::mpsc::UnboundedSender<MarketChangeMessage>>,
    market_state: HashMap<MarketId, MarketChange>,
    market_filter: MarketFilter,
    initial_market_clk: Option<String>,
    latest_market_clk: Option<String>,
}

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
) {
    use futures_util::{FutureExt, StreamExt};
    futures::pin_mut!(read);
    let mut read = read.fuse();

    // TODO periodically send heartbeats

    while let Some(msg) = read.next().await {
        match msg {
            Ok(msg) => {
                tracing::debug!(msg = ?msg, "Received message");
                let mut w = api.write().unwrap();
                w.process_response_message(msg);
                drop(w)
            }
            Err(err) => {
                tracing::error!(err = ?err, "Error reading from stream");
                break
            }
        }
    }

    // TODO change internal state to disconnected
}
