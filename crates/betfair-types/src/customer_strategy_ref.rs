use serde::{Deserialize, Serialize};

/// An optional reference customers can use to specify which strategy has sent the order.
/// The reference will be returned on order change messages through the stream API. The string
/// is limited to 15 characters. If an empty string is provided it will be treated as null.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct CustomerStrategyRef([char; 15]);

impl CustomerStrategyRef {
    pub fn new(customer_strategy_ref: [char; 15]) -> Self {
        CustomerStrategyRef(customer_strategy_ref)
    }
}
