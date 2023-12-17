use serde::{Deserialize, Serialize};

use super::{Ct, SegmentType};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketChangeMessage {
    /// The operation type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    /// Client generated unique id to link request with response (like json rpc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    /// Change Type - set to indicate the type of change - if null this is a delta)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ct: Option<Ct>,
    /// Token value (non-null) should be stored and passed in a MarketSubscriptionMessage to resume
    /// subscription (in case of disconnect)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clk: Option<String>,
    /// Heartbeat Milliseconds - the heartbeat rate (may differ from requested: bounds are 500 to
    /// 30000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_ms: Option<i64>,
    /// Publish Time (in millis since epoch) that the changes were generated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pt: Option<i64>,
    /// Token value (non-null) should be stored and passed in a MarketSubscriptionMessage to resume
    /// subscription (in case of disconnect)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_clk: Option<String>,
    /// MarketChanges - the modifications to markets (will be null on a heartbeat
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mc: Option<Vec<super::MarketChange>>,
    /// Conflate Milliseconds - the conflation rate (may differ from that requested if subscription
    /// is delayed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflate_ms: Option<i64>,
    /// Segment Type - if the change is split into multiple segments, this denotes the beginning
    /// and end of a change, and segments in between. Will be null if data is not segmented
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_type: Option<SegmentType>,
    /// Stream status: set to null if the exchange stream data is up to date and 503 if the
    /// downstream services are experiencing latencies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
}

impl MarketChangeMessage {
    #[allow(dead_code)]
    pub fn new() -> MarketChangeMessage {
        MarketChangeMessage {
            op: None,
            id: None,
            ct: None,
            clk: None,
            heartbeat_ms: None,
            pt: None,
            initial_clk: None,
            mc: None,
            conflate_ms: None,
            segment_type: None,
            status: None,
        }
    }
}
