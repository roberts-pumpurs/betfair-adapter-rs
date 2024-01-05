use betfair_types::{price::Price, size::Size};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub mod connection_message;
pub mod market_change_message;
pub mod order_change_message;
pub mod status_message;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
#[serde(rename_all = "camelCase")]
pub enum ResponseMessage {
    #[serde(rename = "connection")]
    Connection(connection_message::ConnectionMessage),
    #[serde(rename = "mcm")]
    MarketChange(market_change_message::MarketChangeMessage),
    #[serde(rename = "ocm")]
    OrderChange(order_change_message::OrderChangeMessage),
    #[serde(rename = "status")]
    StatusMessage(status_message::StatusMessage),
}

/// Change Type - set to indicate the type of change - if null this is a delta)
#[derive(
    Clone, Copy, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Ct {
    #[default]
    SubImage,
    ResubDelta,
    Heartbeat,
}

/// Segment Type - if the change is split into multiple segments, this denotes the beginning and end
/// of a change, and segments in between. Will be null if data is not segmented
#[derive(
    Clone, Copy, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SegmentType {
    #[default]
    SegStart,
    Seg,
    SegEnd,
}
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetChangeMessage<T> {
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
    /// the modifications to T (will be null on a heartbeat)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mc: Option<Vec<T>>,
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

#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Eq, Hash, Ord)]
pub struct UpdateSet2(pub Price, pub Size);

#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Eq, Hash, Ord)]
pub struct UpdateSet3(pub Position, pub Price, pub Size);

/// Represents the level of the order book.
#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Eq, Hash, Ord)]
pub struct Position(pub Decimal);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_deserialize_connection() {
        let msg = "{\"op\":\"connection\",\"connectionId\":\"206-221122192222-702491\"}";
        let msg = serde_json::from_str::<ResponseMessage>(msg).unwrap();

        assert_eq!(
            msg,
            ResponseMessage::Connection(connection_message::ConnectionMessage {
                connection_id: Some("206-221122192222-702491".to_string()),
                id: None,
            })
        );
    }
}
