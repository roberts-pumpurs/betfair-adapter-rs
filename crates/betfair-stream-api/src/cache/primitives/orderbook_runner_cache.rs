//! Cache of a runner in an order book (used for order book caching)

use std::collections::HashMap;

use betfair_adapter::betfair_types::customer_strategy_ref::CustomerStrategyRef;
use betfair_adapter::betfair_types::numeric::F64Ord;
use betfair_adapter::betfair_types::types::sports_aping::{BetId, MarketId, SelectionId};
use betfair_stream_types::response::UpdateSet2;
use betfair_stream_types::response::order_change_message::{Order, StrategyMatchChange};
use serde::{Deserialize, Serialize};

use super::available_cache::Available;

/// Cache of a runner in an order book (used for order book caching)
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct OrderBookRunner {
    pub market_id: MarketId,
    pub selection_id: SelectionId,
    pub matched_lays: Available<UpdateSet2>,
    pub matched_backs: Available<UpdateSet2>,
    pub unmatched_orders: HashMap<BetId, Order>,
    pub handicap: Option<F64Ord>,
    pub strategy_matches: HashMap<CustomerStrategyRef, StrategyMatch>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct StrategyMatch {
    pub matched_lays: Available<UpdateSet2>,
    pub matched_backs: Available<UpdateSet2>,
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

    pub(crate) fn update_unmatched<'o>(
        &mut self,
        unmatched_orders: impl IntoIterator<Item = &'o Order>,
    ) {
        for order in unmatched_orders {
            self.unmatched_orders
                .insert(order.id.clone(), order.clone());
        }
    }

    pub(crate) fn update_matched_lays(&mut self, ml: &Vec<UpdateSet2>) {
        self.matched_lays.update(ml);
    }

    pub(crate) fn update_matched_backs(&mut self, mb: &Vec<UpdateSet2>) {
        self.matched_backs.update(mb);
    }

    pub(crate) fn update_strategy_matches(
        &mut self,
        sm: &HashMap<CustomerStrategyRef, StrategyMatchChange>,
    ) {
        for (key, value) in sm {
            let entry = self
                .strategy_matches
                .entry(key.clone())
                .or_insert_with(|| StrategyMatch {
                    matched_lays: Available::new(&[]),
                    matched_backs: Available::new(&[]),
                });

            if let Some(ref ml) = value.ml {
                entry.matched_lays.update(ml);
            }

            if let Some(ref mb) = value.mb {
                entry.matched_backs.update(mb);
            }
        }
    }
}
