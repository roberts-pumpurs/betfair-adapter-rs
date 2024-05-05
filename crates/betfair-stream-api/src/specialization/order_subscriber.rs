use betfair_adapter::betfair_types::customer_strategy_ref::CustomerStrategyRef;
use betfair_stream_types::request::order_subscription_message::{
    OrderFilter, OrderSubscriptionMessage,
};
use betfair_stream_types::request::RequestMessage;

use crate::StreamApiConnection;

/// A warpper around a `StreamListener` that allows subscribing to order updates with a somewhat
/// ergonomic API.
pub struct OrderSubscriber {
    command_sender: tokio::sync::broadcast::Sender<RequestMessage>,
    filter: OrderFilter,
}

impl OrderSubscriber {
    pub fn new<T>(stream_api_connection: &StreamApiConnection<T>, filter: OrderFilter) -> Self {
        let command_sender = stream_api_connection.command_sender().clone();
        Self {
            command_sender,
            filter,
        }
    }

    /// Create a new market subscriber.
    pub async fn subscribe_to_strategy_updates(
        &mut self,
        strategy_ref: CustomerStrategyRef,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        self.filter
            .customer_strategy_refs
            .as_mut()
            .map(|x| x.push(strategy_ref));

        self.resubscribe().await
    }

    /// Unsubscribe from a market.
    pub async fn unsubscribe_from_strategy_updates(
        &mut self,
        strategy_ref: CustomerStrategyRef,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        self.filter
            .customer_strategy_refs
            .as_mut()
            .map(|x| x.retain(|x| x != &strategy_ref));

        if self
            .filter
            .customer_strategy_refs
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
        self.command_sender.send(req)?;

        Ok(())
    }

    pub async fn resubscribe(
        &mut self,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        let req = RequestMessage::OrderSubscription(OrderSubscriptionMessage {
            id: None,
            clk: None,         // empty to reset the clock
            initial_clk: None, // empty to reset the clock
            segmentation_enabled: Some(true),
            heartbeat_ms: Some(500),
            order_filter: Some(Box::new(self.filter.clone())),
            conflate_ms: None,
        });
        self.command_sender.send(req)?;

        Ok(())
    }

    pub fn filter(&self) -> &OrderFilter {
        &self.filter
    }

    pub async fn set_filter(
        &mut self,
        filter: OrderFilter,
    ) -> Result<(), tokio::sync::broadcast::error::SendError<RequestMessage>> {
        self.filter = filter;
        self.resubscribe().await
    }
}
