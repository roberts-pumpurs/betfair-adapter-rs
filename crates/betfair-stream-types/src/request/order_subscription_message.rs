use betfair_types::customer_strategy_ref::CustomerStrategyRef;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderSubscriptionMessage {
    /// Client generated unique id to link request with response (like json rpc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    /// Segmentation Enabled - allow the server to send large sets of data in segments, instead of
    /// a single block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segmentation_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_filter: Option<Box<OrderFilter>>,
    /// Token value delta (received in MarketChangeMessage) that should be passed to resume a
    /// subscription
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clk: Option<String>,
    /// Heartbeat Milliseconds - the heartbeat rate (looped back on initial image after validation:
    /// bounds are 500 to 5000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_ms: Option<i64>,
    /// Token value (received in initial MarketChangeMessage) that should be passed to resume a
    /// subscription
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_clk: Option<String>,
    /// Conflate Milliseconds - the conflation rate (looped back on initial image after validation:
    /// bounds are 0 to 120000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflate_ms: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize, TypedBuilder)]
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
    pub customer_strategy_refs: Option<Vec<CustomerStrategyRef>>,
    /// Returns strategy positions (See: OrderRunnerChange.smc=Map<customerStrategyRef,
    /// StrategyMatchChange>) - these are sent in delta format as per overall position.
    /// Default=false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition_matched_by_strategy_ref: Option<bool>,
}
