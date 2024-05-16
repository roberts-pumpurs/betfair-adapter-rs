use std::collections::HashMap;

use betfair_adapter::betfair_types::customer_strategy_ref::CustomerStrategyRef;
use betfair_adapter::betfair_types::types::sports_aping::{BetId, MarketId, SelectionId};
use betfair_adapter::rust_decimal::Decimal;
use betfair_stream_types::response::order_change_message::{Order, StrategyMatchChange};
use betfair_stream_types::response::UpdateSet2;
use serde::{Deserialize, Serialize};

use super::available_cache::Available;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct OrderBookRunner {
    pub market_id: MarketId,
    pub selection_id: SelectionId,
    pub matched_lays: Available<UpdateSet2>,
    pub matched_backs: Available<UpdateSet2>,
    pub unmatched_orders: HashMap<BetId, Order>,
    pub handicap: Option<Decimal>,
    pub strategy_matches: HashMap<CustomerStrategyRef, StrategyMatchChange>,
}

impl OrderBookRunner {
    pub(crate) fn new(market_id: MarketId, selection_id: SelectionId) -> Self {
        Self {
            market_id,
            selection_id,
            matched_lays: Available::new(&[]),
            matched_backs: Available::new(&[]),
            unmatched_orders: HashMap::new(),
            handicap: None,
            strategy_matches: HashMap::new(),
        }
    }

    pub(crate) fn update_unmatched(&mut self, unmatched_orders: impl IntoIterator<Item = Order>) {
        for order in unmatched_orders {
            self.unmatched_orders.insert(order.id.clone(), order);
        }
    }

    pub(crate) fn update_matched_lays(&mut self, ml: Vec<UpdateSet2>) {
        self.matched_lays.update(ml);
    }

    pub(crate) fn update_matched_backs(&mut self, mb: Vec<UpdateSet2>) {
        self.matched_backs.update(mb);
    }
}
