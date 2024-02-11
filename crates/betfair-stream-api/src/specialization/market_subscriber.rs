use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use betfair_adapter::betfair_types::types::sports_aping::MarketId;
use betfair_stream_types::request::market_subscription_message::{
    Fields, LadderLevel, MarketDataFilter, MarketFilter, MarketSubscriptionMessage,
};
use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::response::market_change_message::{MarketChange, MarketChangeMessage};

use crate::StreamListener;

/// A warpper around a `StreamListener` that allows subscribing to markets with a somewhat ergonomic API.
pub struct MarketSubscriber {
    stream_listener: std::sync::Arc<tokio::sync::RwLock<StreamListener>>,
    filter: MarketFilter,
    /// The list of market data fields to subscribe to.
    market_data_fields: Vec<Fields>,
    /// For depth-based ladders the number of levels to send (1 to 10). 1 is best price to back or
    /// lay etc.
    ladder_level: Option<LadderLevel>,
    market_mpsc: HashMap<MarketId, futures::channel::mpsc::UnboundedSender<MarketChangeMessage>>,
}

impl MarketSubscriber {
    pub fn new(
        stream_listener: std::sync::Arc<tokio::sync::RwLock<StreamListener>>,
        filter: MarketFilter,
        market_data_fields: Vec<Fields>,
        ladder_level: Option<LadderLevel>,
    ) -> Self {
        Self {
            stream_listener,
            filter,
            market_data_fields,
            ladder_level,
            market_mpsc: HashMap::new(),
        }
    }

    /// Create a new market subscriber.
    pub async fn subscribe_to_market(
        &mut self,
        market_id: MarketId,
    ) -> futures::channel::mpsc::UnboundedReceiver<MarketChangeMessage> {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        self.market_mpsc.insert(market_id, tx);

        self.resubscribe().await;

        rx
    }


    /// Unsubscribe from a market.
    pub async fn unsubscribe_from_market(&mut self, market_ids: MarketId)  {
        let _value = self.market_mpsc.remove(&market_ids);

        if self.market_mpsc.is_empty() {
            self.unsubscribe_from_all_markets().await;
        }
    }

    /// Unsubscribe from all markets.
    ///
    /// Internally it uses a weird trick of subscribing to a market that does not exist to simulate
    /// unsubscribing from all markets.
    /// https://forum.developer.betfair.com/forum/sports-exchange-api/exchange-api/34555-stream-api-unsubscribe-from-all-markets
    pub async fn unsubscribe_from_all_markets(&mut self) {
        let market_that_does_not_exist = MarketId("1.23456789".to_string());
        self.filter = MarketFilter::default();

        let req = RequestMessage::MarketSubscription(MarketSubscriptionMessage {
            id: None,
            segmentation_enabled: Some(true),
            clk: None,
            heartbeat_ms: Some(500),
            initial_clk: None,
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
        let mut w = self.stream_listener.write().await;
        w.send_message(req).unwrap();
        drop(w);
    }

    async fn resubscribe(&mut self) {
        let all_market_ids = self.market_mpsc.keys().cloned().collect::<Vec<_>>();
        self.filter.market_ids = Some(all_market_ids);

        let req = RequestMessage::MarketSubscription(MarketSubscriptionMessage {
            id: None,
            clk: None,         // empty to reset the clock
            initial_clk: None, // empty to reset the clock
            segmentation_enabled: Some(true),
            heartbeat_ms: Some(500),
            market_filter: Some(Box::new(self.filter.clone())),
            conflate_ms: None,
            market_data_filter: Some(Box::new(MarketDataFilter {
                ladder_levels: self.ladder_level.clone(),
                fields: Some(self.market_data_fields.clone()),
            })),
        });
        let mut w = self.stream_listener.write().await;
        w.send_message(req).unwrap();
        drop(w);
    }

    pub fn filter(&self) -> &MarketFilter {
        &self.filter
    }

    pub async fn set_filter(&mut self, filter: MarketFilter) {
        self.filter = filter;
        self.resubscribe().await;
    }

    pub fn ladder_level(&self) -> Option<&LadderLevel> {
        self.ladder_level.as_ref()
    }

    pub async fn set_ladder_level(&mut self, ladder_level: Option<LadderLevel>) {
        self.ladder_level = ladder_level;
        self.resubscribe().await;
    }

    pub fn market_data_fields(&self) -> &Vec<Fields> {
        &self.market_data_fields
    }

    pub async fn set_market_data_fields(&mut self, market_data_fields: Vec<Fields>) {
        self.market_data_fields = market_data_fields;
        self.resubscribe().await;
    }
}

impl Deref for MarketSubscriber {
    type Target = std::sync::Arc<tokio::sync::RwLock<StreamListener>>;

    fn deref(&self) -> &Self::Target {
        &self.stream_listener
    }
}

impl DerefMut for MarketSubscriber {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream_listener
    }
}
