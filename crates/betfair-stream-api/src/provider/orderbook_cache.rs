use std::collections::HashMap;

use betfair_adapter::betfair_types::types::sports_aping::{MarketId, SelectionId};
use betfair_adapter::rust_decimal::Decimal;
use betfair_stream_types::response::order_change_message::OrderMarketChange;

use super::orderbook_runner_cache::OrderBookRunner;

pub struct OrderBookCache {
    market_id: MarketId,
    closed: Option<bool>,
    active: Option<bool>,
    runners: HashMap<(SelectionId, Option<Decimal>), OrderBookRunner>,
}

impl OrderBookCache {
    pub fn new(market_id: MarketId) -> Self {
        Self {
            market_id,
            closed: None,
            active: None,
            runners: HashMap::new(),
        }
    }

    pub fn update_cache(&mut self, change: OrderMarketChange) {
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
}
