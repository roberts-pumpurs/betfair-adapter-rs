use betfair_adapter::betfair_types::types::sports_aping::MarketId;
use betfair_stream_types::request::market_subscription_message::{
    Fields, LadderLevel, MarketDataFilter, MarketFilter, MarketSubscriptionMessage,
};
use betfair_stream_types::request::RequestMessage;

use crate::StreamApiConnection;

/// A wrapper around a `StreamListener` that allows subscribing to markets with a somewhat ergonomic
/// API.
pub struct MarketSubscriber {
    command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
    filter: MarketFilter,
    /// The list of market data fields to subscribe to.
    market_data_fields: Vec<Fields>,
    /// For depth-based ladders the number of levels to send (1 to 10). 1 is best price to back or
    /// lay etc.
    ladder_level: Option<LadderLevel>,
}

impl MarketSubscriber {
    pub fn new<T>(
        stream_api_connection: &StreamApiConnection<T>,
        filter: MarketFilter,
        market_data_fields: Vec<Fields>,
        ladder_level: Option<LadderLevel>,
    ) -> Self {
        let command_sender = stream_api_connection.command_sender().clone();
        Self {
            command_sender,
            filter,
            market_data_fields,
            ladder_level,
        }
    }

    /// Create a new market subscriber.
    pub async fn subscribe_to_market(
        &mut self,
        market_id: MarketId,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        if let Some(market_ids) = &mut self.filter.market_ids {
            market_ids.push(market_id);
        } else {
            self.filter.market_ids = Some(vec![market_id]);
        }
        self.resubscribe().await
    }

    /// Unsubscribe from a market.
    pub async fn unsubscribe_from_market(
        &mut self,
        market_id: MarketId,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        if let Some(x) = self.filter.market_ids.as_mut() {
            x.retain(|x| x != &market_id)
        }

        if self
            .filter
            .market_ids
            .as_ref()
            .map(|x| x.is_empty())
            .unwrap_or(true)
        {
            self.unsubscribe_from_all_markets().await?;
        }

        Ok(())
    }

    /// Unsubscribe from all markets.
    ///
    /// Internally it uses a weird trick of subscribing to a market that does not exist to simulate
    /// unsubscribing from all markets.
    /// https://forum.developer.betfair.com/forum/sports-exchange-api/exchange-api/34555-stream-api-unsubscribe-from-all-markets
    pub async fn unsubscribe_from_all_markets(
        &mut self,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        let market_that_does_not_exist = MarketId("1.23456789".to_string());
        self.filter = MarketFilter::default();

        let req = RequestMessage::MarketSubscription(MarketSubscriptionMessage {
            id: None,
            segmentation_enabled: Some(true),
            clk: None,
            heartbeat_ms: Some(1000),
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

        self.command_sender.send(req)?;

        Ok(())
    }

    pub async fn resubscribe(
        &mut self,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        let req = RequestMessage::MarketSubscription(MarketSubscriptionMessage {
            id: None,
            clk: None,         // empty to reset the clock
            initial_clk: None, // empty to reset the clock
            segmentation_enabled: Some(true),
            heartbeat_ms: Some(1000),
            market_filter: Some(Box::new(self.filter.clone())),
            conflate_ms: None,
            market_data_filter: Some(Box::new(MarketDataFilter {
                ladder_levels: self.ladder_level.clone(),
                fields: Some(self.market_data_fields.clone()),
            })),
        });
        self.command_sender.send(req)?;

        Ok(())
    }

    pub fn filter(&self) -> &MarketFilter {
        &self.filter
    }

    pub async fn set_filter(
        &mut self,
        filter: MarketFilter,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        self.filter = filter;
        self.resubscribe().await
    }

    pub fn ladder_level(&self) -> Option<&LadderLevel> {
        self.ladder_level.as_ref()
    }

    pub async fn set_ladder_level(
        &mut self,
        ladder_level: Option<LadderLevel>,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        self.ladder_level = ladder_level;
        self.resubscribe().await
    }

    pub fn market_data_fields(&self) -> &Vec<Fields> {
        &self.market_data_fields
    }

    pub async fn set_market_data_fields(
        &mut self,
        market_data_fields: Vec<Fields>,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        self.market_data_fields = market_data_fields;
        self.resubscribe().await
    }
}
