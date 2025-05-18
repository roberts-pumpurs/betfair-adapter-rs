use betfair_adapter::betfair_types::customer_strategy_ref::CustomerStrategyRef;
use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::request::order_subscription_message::{
    OrderFilter, OrderSubscriptionMessage,
};
use tokio::sync::mpsc::Sender;

use crate::{BetfairStreamClient, MessageProcessor};

/// A wrapper around a `StreamListener` that allows subscribing to order updates with a somewhat
/// ergonomic API.
pub struct OrderSubscriber {
    command_sender: Sender<RequestMessage>,
    filter: OrderFilter,
}

impl OrderSubscriber {
    #[must_use]
    pub fn new<T: MessageProcessor>(
        stream_api_connection: &BetfairStreamClient<T>,
        filter: OrderFilter,
    ) -> Self {
        let command_sender = stream_api_connection.send_to_stream.clone();
        Self {
            command_sender,
            filter,
        }
    }

    /// Create a new market subscriber.
    ///
    /// # Errors
    /// If the message cannot be sent to the stream.
    pub async fn subscribe_to_strategy_updates(
        &mut self,
        strategy_ref: CustomerStrategyRef,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<RequestMessage>> {
        if let Some(ref mut strategy_refs) = self.filter.customer_strategy_refs {
            strategy_refs.push(strategy_ref);
        } else {
            self.filter.customer_strategy_refs = Some(vec![strategy_ref]);
        }

        self.resubscribe().await
    }

    /// Unsubscribe from a market.
    ///
    /// # Errors
    /// If the message cannot be sent to the stream.
    pub async fn unsubscribe_from_strategy_updates(
        &mut self,
        strategy_ref: &CustomerStrategyRef,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<RequestMessage>> {
        if let Some(x) = self.filter.customer_strategy_refs.as_mut() {
            x.retain(|iter_strategy_ref| iter_strategy_ref != strategy_ref);
        }

        if self
            .filter
            .customer_strategy_refs
            .as_ref()
            .is_none_or(alloc::vec::Vec::is_empty)
        {
            self.unsubscribe_from_all_markets().await?;
        }

        Ok(())
    }

    /// Unsubscribe from all markets.
    ///
    /// Internally it uses a weird trick of subscribing to a market that does not exist to simulate
    /// unsubscribing from all markets.
    /// [betfair docs](https://forum.developer.betfair.com/forum/sports-exchange-api/exchange-api/34555-stream-api-unsubscribe-from-all-markets)
    ///
    /// # Errors
    /// if the message cannot be sent to the stream.
    pub async fn unsubscribe_from_all_markets(
        &mut self,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<RequestMessage>> {
        let strategy_that_does_not_exist = CustomerStrategyRef::new([
            'd', 'o', 's', 'e', 'n', 't', ' ', 'e', 'x', 'i', 's', 't', ' ', ' ', ' ',
        ]);
        self.filter = OrderFilter::default();

        let req = RequestMessage::OrderSubscription(OrderSubscriptionMessage {
            id: None,
            segmentation_enabled: Some(true),
            clk: None,
            heartbeat_ms: None,
            initial_clk: None,
            order_filter: Some(Box::new(OrderFilter {
                include_overall_position: Some(false),
                account_ids: None,
                customer_strategy_refs: Some(vec![strategy_that_does_not_exist]),
                partition_matched_by_strategy_ref: None,
            })),
            conflate_ms: None,
        });
        self.command_sender.send(req).await?;

        Ok(())
    }

    /// Resubscribe to the stream.
    ///
    /// This is useful when the stream is disconnected and you want to resubscribe to the stream.
    ///
    /// # Errors
    /// if the stream fails to send the message
    pub async fn resubscribe(
        &self,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<RequestMessage>> {
        let req = RequestMessage::OrderSubscription(OrderSubscriptionMessage {
            id: None,
            clk: None,         // empty to reset the clock
            initial_clk: None, // empty to reset the clock
            segmentation_enabled: Some(true),
            heartbeat_ms: None,
            order_filter: Some(Box::new(self.filter.clone())),
            conflate_ms: None,
        });
        self.command_sender.send(req).await?;

        Ok(())
    }

    #[must_use]
    pub const fn filter(&self) -> &OrderFilter {
        &self.filter
    }

    /// Set the filter for the subscriber.
    ///
    /// # Errors
    /// if the stream fails to send the message
    pub async fn set_filter(
        &mut self,
        filter: OrderFilter,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<RequestMessage>> {
        self.filter = filter;
        self.resubscribe().await
    }
}
