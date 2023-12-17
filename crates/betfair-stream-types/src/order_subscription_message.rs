use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderSubscriptionMessage {
    /// The operation type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    /// Client generated unique id to link request with response (like json rpc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    /// Segmentation Enabled - allow the server to send large sets of data in segments, instead of
    /// a single block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segmentation_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_filter: Option<Box<super::OrderFilter>>,
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

impl OrderSubscriptionMessage {
    #[allow(dead_code)]
    pub fn new() -> OrderSubscriptionMessage {
        OrderSubscriptionMessage {
            op: None,
            id: None,
            segmentation_enabled: None,
            order_filter: None,
            clk: None,
            heartbeat_ms: None,
            initial_clk: None,
            conflate_ms: None,
        }
    }
}
