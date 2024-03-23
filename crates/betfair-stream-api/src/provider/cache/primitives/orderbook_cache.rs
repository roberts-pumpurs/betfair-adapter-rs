use std::collections::HashMap;

use betfair_adapter::betfair_types::handicap::Handicap;
use betfair_adapter::betfair_types::types::sports_aping::{MarketId, SelectionId};
use betfair_stream_types::response::order_change_message::OrderMarketChange;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::orderbook_runner_cache::OrderBookRunner;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct OrderBookCache {
    pub market_id: MarketId,
    publish_time: DateTime<Utc>,
    closed: Option<bool>,
    /// cache of orders placed on a runner
    pub runners: HashMap<(SelectionId, Option<Handicap>), OrderBookRunner>,
}

impl OrderBookCache {
    pub fn new(market_id: MarketId, publish_time: DateTime<Utc>) -> Self {
        Self {
            market_id,
            publish_time,
            closed: None,
            runners: HashMap::new(),
        }
    }

    pub fn is_closed(&self) -> bool {
        self.closed.unwrap_or(false)
    }

    pub fn update_cache(&mut self, change: OrderMarketChange, publish_time: DateTime<Utc>) {
        self.publish_time = publish_time;
        if let Some(closed) = change.closed {
            self.closed = Some(closed);
        }

        let Some(order_runner_change) = change.order_runner_change else {
            return;
        };

        for runner_change in order_runner_change {
            let runner = self
                .runners
                .entry((runner_change.id.clone(), runner_change.handicap))
                .or_insert_with(|| OrderBookRunner::new(self.market_id.clone(), runner_change.id));

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

    pub fn publish_time(&self) -> DateTime<Utc> {
        self.publish_time
    }
}
