use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use betfair_adapter::betfair_types::customer_strategy_ref::CustomerStrategyRef;
use betfair_stream_types::request::order_subscription_message::{
    OrderFilter, OrderSubscriptionMessage,
};
use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::response::market_change_message::MarketChangeMessage;

use crate::StreamListener;

/// A warpper around a `StreamListener` that allows subscribing to order updates with a somewhat
/// ergonomic API.
pub struct OrderSubscriber {
    stream_listener: std::sync::Arc<tokio::sync::RwLock<StreamListener>>,
    filter: OrderFilter,
    order_mpsc:
        HashMap<CustomerStrategyRef, futures::channel::mpsc::UnboundedSender<MarketChangeMessage>>,
}

impl OrderSubscriber {
    pub fn new(
        stream_listener: std::sync::Arc<tokio::sync::RwLock<StreamListener>>,
        filter: OrderFilter,
    ) -> Self {
        Self {
            stream_listener,
            filter,
            order_mpsc: HashMap::new(),
        }
    }

    /// Create a new market subscriber.
    pub async fn subscribe_to_strategy_updates(
        &mut self,
        strategy_ref: CustomerStrategyRef,
    ) -> futures::channel::mpsc::UnboundedReceiver<MarketChangeMessage> {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        self.order_mpsc.insert(strategy_ref, tx);

        self.resubscribe().await;

        rx
    }

    /// Unsubscribe from a market.
    pub async fn unsubscribe_from_strategy_updates(&mut self, strategy_ref: CustomerStrategyRef) {
        let _value = self.order_mpsc.remove(&strategy_ref);

        if self.order_mpsc.is_empty() {
            self.unsubscribe_from_all_markets().await;
        }
    }

    /// Unsubscribe from all markets.
    ///
    /// Internally it uses a weird trick of subscribing to a market that does not exist to simulate
    /// unsubscribing from all markets.
    /// https://forum.developer.betfair.com/forum/sports-exchange-api/exchange-api/34555-stream-api-unsubscribe-from-all-markets
    pub async fn unsubscribe_from_all_markets(&mut self) {
        let strategy_that_does_not_exist = CustomerStrategyRef::new([
            'd', 'o', 's', 'e', 'n', 't', ' ', 'e', 'x', 'i', 's', 't', ' ', ' ', ' ',
        ]);
        self.filter = OrderFilter::default();

        let req = RequestMessage::OrderSubscription(OrderSubscriptionMessage {
            id: None,
            segmentation_enabled: Some(true),
            clk: None,
            heartbeat_ms: Some(500),
            initial_clk: None,
            order_filter: Some(Box::new(OrderFilter {
                include_overall_position: Some(false),
                account_ids: None,
                customer_strategy_refs: Some(vec![strategy_that_does_not_exist]),
                partition_matched_by_strategy_ref: None,
            })),
            conflate_ms: None,
        });
        let mut w = self.stream_listener.write().await;
        w.send_message(req).unwrap();
        drop(w);
    }

    async fn resubscribe(&mut self) {
        let all_strategy_refs = self.order_mpsc.keys().cloned().collect::<Vec<_>>();
        self.filter.customer_strategy_refs = Some(all_strategy_refs);

        let req = RequestMessage::OrderSubscription(OrderSubscriptionMessage {
            id: None,
            clk: None,         // empty to reset the clock
            initial_clk: None, // empty to reset the clock
            segmentation_enabled: Some(true),
            heartbeat_ms: Some(500),
            order_filter: Some(Box::new(self.filter.clone())),
            conflate_ms: None,
        });
        let mut w = self.stream_listener.write().await;
        w.send_message(req).unwrap();
        drop(w);
    }

    pub fn filter(&self) -> &OrderFilter {
        &self.filter
    }

    pub async fn set_filter(&mut self, filter: OrderFilter) {
        self.filter = filter;
        self.resubscribe().await;
    }
}

impl Deref for OrderSubscriber {
    type Target = std::sync::Arc<tokio::sync::RwLock<StreamListener>>;

    fn deref(&self) -> &Self::Target {
        &self.stream_listener
    }
}

impl DerefMut for OrderSubscriber {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream_listener
    }
}
