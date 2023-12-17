use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderFilter {
    /// Returns overall / net position (See: OrderRunnerChange.mb / OrderRunnerChange.ml).
    /// Default=true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_overall_position: Option<bool>,
    /// Internal use only & should not be set on your filter (your subscription is already locked
    /// to your account). If set subscription will fail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_ids: Option<Vec<i64>>,
    /// Restricts to specified customerStrategyRefs; this will filter orders and
    /// StrategyMatchChanges accordingly (Note: overall postition is not filtered)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_strategy_refs: Option<Vec<String>>,
    /// Returns strategy positions (See: OrderRunnerChange.smc=Map<customerStrategyRef,
    /// StrategyMatchChange>) - these are sent in delta format as per overall position.
    /// Default=false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition_matched_by_strategy_ref: Option<bool>,
}

impl OrderFilter {
    #[allow(dead_code)]
    pub fn new() -> OrderFilter {
        OrderFilter {
            include_overall_position: None,
            account_ids: None,
            customer_strategy_refs: None,
            partition_matched_by_strategy_ref: None,
        }
    }
}
