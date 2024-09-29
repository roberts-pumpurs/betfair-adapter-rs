use std::collections::HashMap;

use betfair_adapter::betfair_types::handicap::Handicap;
use betfair_adapter::betfair_types::types::sports_aping::{MarketId, SelectionId};
use betfair_stream_types::response::order_change_message::OrderMarketChange;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::orderbook_runner_cache::OrderBookRunner;

/// Represents a cache for order book data.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct OrderBookCache {
    market_id: MarketId,
    publish_time: DateTime<Utc>,
    closed: bool,
    /// cache of orders placed on a runner
    runners: HashMap<(SelectionId, Option<Handicap>), OrderBookRunner>,
}

/// Implements methods for managing the order book cache.
impl OrderBookCache {
    /// Creates a new `OrderBookCache` with the given market ID and publish time.
    #[must_use]
    pub fn new(market_id: MarketId, publish_time: DateTime<Utc>) -> Self {
        Self {
            market_id,
            publish_time,
            closed: false,
            runners: HashMap::new(),
        }
    }

    /// Checks if the order book is closed.
    #[must_use]
    pub const fn is_closed(&self) -> bool {
        self.closed
    }

    /// Updates the cache with changes from the order market.
    pub fn update_cache(&mut self, change: OrderMarketChange, publish_time: DateTime<Utc>) {
        self.publish_time = publish_time;
        self.closed = change.closed.unwrap_or(self.closed);

        if let Some(order_runner_change) = change.order_runner_change {
            for runner_change in order_runner_change {
                let runner = self
                    .runners
                    .entry((runner_change.id.clone(), runner_change.handicap))
                    .or_insert_with(|| {
                        OrderBookRunner::new(self.market_id.clone(), runner_change.id.clone())
                    });

                if let Some(ml) = runner_change.matched_lays {
                    runner.update_matched_lays(ml);
                }
                if let Some(mb) = runner_change.matched_backs {
                    runner.update_matched_backs(mb);
                }
                if let Some(uo) = runner_change.unmatched_orders {
                    runner.update_unmatched(uo);
                }
            }
        }
    }

    /// Returns the publish time of the order book.
    #[must_use]
    pub const fn publish_time(&self) -> DateTime<Utc> {
        self.publish_time
    }

    /// Returns a reference to the runners in the order book cache.
    #[must_use]
    pub const fn runners(&self) -> &HashMap<(SelectionId, Option<Handicap>), OrderBookRunner> {
        &self.runners
    }

    /// Consumes the `OrderBookCache` and returns the runners.
    #[must_use]
    pub fn into_runners(self) -> HashMap<(SelectionId, Option<Handicap>), OrderBookRunner> {
        self.runners
    }

    /// Returns a reference to the market ID of the order book cache.
    #[must_use]
    pub const fn market_id(&self) -> &MarketId {
        &self.market_id
    }
}
