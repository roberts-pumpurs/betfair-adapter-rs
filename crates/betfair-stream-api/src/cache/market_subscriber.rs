use std::sync::Arc;

use betfair_adapter::betfair_types::types::sports_aping::MarketId;
use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::request::market_subscription_message::{
    Fields, LadderLevel, MarketDataFilter, MarketFilter, MarketSubscriptionMessage,
};

use crate::{BetfairStreamClient, MessageProcessor};

/// A wrapper around a `StreamListener` that allows subscribing to markets with a somewhat ergonomic
/// API.
pub struct MarketSubscriber {
    command_sender: tokio::sync::mpsc::Sender<RequestMessage>,
    filter: MarketFilter,
    /// The list of market data fields to subscribe to.
    market_data_fields: Vec<Fields>,
    /// For depth-based ladders the number of levels to send (1 to 10). 1 is best price to back or
    /// lay etc.
    ladder_level: Option<LadderLevel>,
}

impl MarketSubscriber {
    /// Creates a new `MarketSubscriber`.
    ///
    /// # Parameters
    /// - `stream_api_connection`: A reference to the `StreamApi` connection.
    /// - `filter`: The `MarketFilter` to apply.
    /// - `market_data_fields`: A vector of `Fields` to subscribe to.
    /// - `ladder_level`: An optional `LadderLevel` for depth-based ladders.
    ///
    /// # Returns
    /// A new instance of `MarketSubscriber`.
    #[must_use]
    pub fn new<T: MessageProcessor>(
        stream_api_connection: &BetfairStreamClient<T>,
        filter: MarketFilter,
        market_data_fields: Vec<Fields>,
        ladder_level: Option<LadderLevel>,
    ) -> Self {
        let command_sender = stream_api_connection.send_to_stream.clone();
        Self {
            command_sender,
            filter,
            market_data_fields,
            ladder_level,
        }
    }

    /// Subscribe to a market using its `MarketId`.
    ///
    /// # Parameters
    /// - `market_id`: The `MarketId` of the market to subscribe to.
    ///
    /// # Errors
    /// If the message cannot be sent to the stream.
    pub async fn subscribe_to_market(
        &mut self,
        market_id: MarketId,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<RequestMessage>> {
        if let Some(ref mut market_ids) = self.filter.market_ids {
            market_ids.push(market_id);
        } else {
            self.filter.market_ids = Some(vec![market_id]);
        }
        self.resubscribe().await
    }

    /// Unsubscribe from a market using its `MarketId`.
    ///
    /// # Parameters
    /// - `market_id`: A reference to the `MarketId` of the market to unsubscribe from.
    ///
    /// # Errors
    /// If the message cannot be sent to the stream.
    pub async fn unsubscribe_from_market(
        &mut self,
        market_id: &MarketId,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<RequestMessage>> {
        if let Some(x) = self.filter.market_ids.as_mut() {
            x.retain(|iter_market_id| iter_market_id != market_id);
        }

        if self
            .filter
            .market_ids
            .as_ref()
            .is_none_or(alloc::vec::Vec::is_empty)
        {
            self.unsubscribe_from_all_markets().await?;
        }

        Ok(())
    }

    /// Unsubscribe from all markets.
    ///
    /// Internally it uses a trick of subscribing to a non-existent market to simulate
    /// unsubscribing from all markets.
    ///
    /// # Errors
    /// If sending the request to the underlying stream fails.
    pub async fn unsubscribe_from_all_markets(
        &mut self,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<RequestMessage>> {
        let market_that_does_not_exist = MarketId(Arc::new("1.23456789".to_owned()));
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

        self.command_sender.send(req).await?;

        Ok(())
    }

    /// Resubscribe to the markets.
    ///
    /// # Errors
    /// If sending the request to the underlying stream fails.
    pub async fn resubscribe(
        &self,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<RequestMessage>> {
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
        self.command_sender.send(req).await?;

        Ok(())
    }

    /// Get the current filter for the market subscription.
    ///
    /// # Returns
    /// A reference to the current `MarketFilter`.
    #[must_use]
    pub const fn filter(&self) -> &MarketFilter {
        &self.filter
    }

    /// Set the filter for the market subscription.
    ///
    /// # Parameters
    /// - `filter`: The new `MarketFilter` to apply.
    ///
    /// # Errors
    /// If the request to change the subscription fails.
    pub async fn set_filter(
        &mut self,
        filter: MarketFilter,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<RequestMessage>> {
        self.filter = filter;
        self.resubscribe().await
    }

    /// Get the current ladder level for depth-based ladders.
    ///
    /// # Returns
    /// An optional reference to the current `LadderLevel`.
    #[must_use]
    pub const fn ladder_level(&self) -> Option<&LadderLevel> {
        self.ladder_level.as_ref()
    }

    /// Set the ladder level for depth-based ladders.
    ///
    /// # Parameters
    /// - `ladder_level`: An optional `LadderLevel` to set.
    ///
    /// # Errors
    /// If the request to change the subscription fails.
    pub async fn set_ladder_level(
        &mut self,
        ladder_level: Option<LadderLevel>,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<RequestMessage>> {
        self.ladder_level = ladder_level;
        self.resubscribe().await
    }

    /// Get the current market data fields to subscribe to.
    ///
    /// # Returns
    /// A reference to the vector of `Fields`.
    #[must_use]
    pub const fn market_data_fields(&self) -> &Vec<Fields> {
        &self.market_data_fields
    }

    /// Set the market data fields to subscribe to.
    ///
    /// # Parameters
    /// - `market_data_fields`: A vector of `Fields` to set.
    ///
    /// # Errors
    /// If the request to change the subscription fails.
    pub async fn set_market_data_fields(
        &mut self,
        market_data_fields: Vec<Fields>,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<RequestMessage>> {
        self.market_data_fields = market_data_fields;
        self.resubscribe().await
    }
}
