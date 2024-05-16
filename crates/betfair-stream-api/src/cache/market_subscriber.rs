use betfair_adapter::betfair_types::types::sports_aping::MarketId;
use betfair_stream_types::request::market_subscription_message::{
    Fields, LadderLevel, MarketDataFilter, MarketFilter, MarketSubscriptionMessage,
};
use betfair_stream_types::request::RequestMessage;

use crate::StreamApi;

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
    #[must_use]
    pub fn new<T>(
        stream_api_connection: &StreamApi<T>,
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
    /// # Errors
    /// If the message cannot be sent to the stream.
    pub fn subscribe_to_market(
        &mut self,
        market_id: MarketId,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        if let Some(ref mut market_ids) = self.filter.market_ids {
            market_ids.push(market_id);
        } else {
            self.filter.market_ids = Some(vec![market_id]);
        }
        self.resubscribe()
    }

    /// Unsubscribe from a market.
    /// # Errors
    /// If the message cannot be sent to the stream.
    pub fn unsubscribe_from_market(
        &mut self,
        market_id: &MarketId,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        if let Some(x) = self.filter.market_ids.as_mut() {
            x.retain(|iter_market_id| iter_market_id != market_id);
        }

        if self
            .filter
            .market_ids
            .as_ref()
            .map_or(true, alloc::vec::Vec::is_empty)
        {
            self.unsubscribe_from_all_markets()?;
        }

        Ok(())
    }

    /// Unsubscribe from all markets.
    ///
    /// Internally it uses a weird trick of subscribing to a market that does not exist to simulate
    /// unsubscribing from all markets.
    /// [refernce](https://forum.developer.betfair.com/forum/sports-exchange-api/exchange-api/34555-stream-api-unsubscribe-from-all-markets)
    ///
    /// # Errors
    /// If sending the request to the underlying stream fails.
    pub fn unsubscribe_from_all_markets(
        &mut self,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        let market_that_does_not_exist = MarketId("1.23456789".to_owned());
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

    /// Resubscribe to the markets.
    ///
    /// # Errors
    /// If sending the request to the underlying stream fails.
    pub fn resubscribe(
        &self,
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

    #[must_use]
    pub const fn filter(&self) -> &MarketFilter {
        &self.filter
    }

    /// Set the filter for the market subscription.
    ///
    /// # Errors
    /// If the request to change the subscription fails.
    pub fn set_filter(
        &mut self,
        filter: MarketFilter,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        self.filter = filter;
        self.resubscribe()
    }

    #[must_use]
    pub const fn ladder_level(&self) -> Option<&LadderLevel> {
        self.ladder_level.as_ref()
    }

    /// Set the ladder level for depth-based ladders.
    ///
    /// # Errors
    /// If the request to change the subscription fails.
    pub fn set_ladder_level(
        &mut self,
        ladder_level: Option<LadderLevel>,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        self.ladder_level = ladder_level;
        self.resubscribe()
    }

    #[must_use]
    pub const fn market_data_fields(&self) -> &Vec<Fields> {
        &self.market_data_fields
    }

    /// Set the market data fields to subscribe to.
    ///
    /// # Errors
    /// If the request to change the subscription fails.
    pub fn set_market_data_fields(
        &mut self,
        market_data_fields: Vec<Fields>,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        self.market_data_fields = market_data_fields;
        self.resubscribe()
    }
}
