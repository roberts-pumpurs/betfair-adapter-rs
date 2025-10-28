use betfair_types::NumericPrimitive;
use betfair_types::price::Price;
use betfair_types::size::Size;
use chrono::{DateTime, TimeZone as _, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

/// Module for handling connection messages.
pub mod connection_message;

/// Module for handling market change messages.
pub mod market_change_message;

/// Module for handling order change messages.
pub mod order_change_message;

/// Module for handling status messages.
pub mod status_message;

/// Represents different types of response messages from the server.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op")]
#[serde(rename_all = "camelCase")]
pub enum ResponseMessage {
    /// Connection message indicating a successful connection.
    #[serde(rename = "connection")]
    Connection(connection_message::ConnectionMessage),
    /// Market change message indicating updates to market data.
    #[serde(rename = "mcm")]
    MarketChange(market_change_message::MarketChangeMessage),
    /// Order change message indicating updates to order data.
    #[serde(rename = "ocm")]
    OrderChange(order_change_message::OrderChangeMessage),
    /// Status message indicating the current status of the connection.
    #[serde(rename = "status")]
    Status(status_message::StatusMessage),
}

/// Change Type - set to indicate the type of change - if null this is a delta.
#[derive(
    Clone, Copy, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChangeType {
    /// Represents a full snapshot of the data.
    #[default]
    SubImage,
    /// Represents a resubscription delta.
    ResubDelta,
    /// Represents a heartbeat message.
    Heartbeat,
}

/// Segment Type - if the change is split into multiple segments, this denotes the beginning and end
/// of a change, and segments in between. Will be null if data is not segmented.
#[derive(
    Clone, Copy, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SegmentType {
    /// Indicates the start of a segment.
    #[default]
    SegStart,
    /// Represents a middle segment.
    Seg,
    /// Indicates the end of a segment.
    SegEnd,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetChangeMessage<T: DeserializeOwned + DataChange<T>> {
    /// Client generated unique id to link request with response (like json rpc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    /// Change Type - set to indicate the type of change - if null this is a delta)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ct")]
    pub change_type: Option<ChangeType>,
    /// Token value (non-null) should be stored and passed in a `MarketSubscriptionMessage` to
    /// resume subscription (in case of disconnect)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "clk")]
    pub clock: Option<Clock>,
    /// Heartbeat Milliseconds - the heartbeat rate (may differ from requested: bounds are 500 to
    /// 30000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_ms: Option<i64>,
    /// Publish Time (in millis since epoch) that the changes were generated
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "pt")]
    pub publish_time: Option<DateTime<Utc>>,
    /// Token value (non-null) should be stored and passed in a `MarketSubscriptionMessage` to
    /// resume subscription (in case of disconnect)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "initialClk")]
    pub initial_clock: Option<InitialClock>,
    /// the modifications to T (will be null on a heartbeat)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<T>>,
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

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Clock(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct InitialClock(pub String);

impl<'de, T> Deserialize<'de> for DatasetChangeMessage<T>
where
    T: DeserializeOwned + DataChange<T>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
        T: DeserializeOwned + DataChange<T>,
    {
        let v = Value::deserialize(deserializer)?;
        let id = v
            .get("id")
            .and_then(serde_json::Value::as_i64)
            .map(|id| id as i32);
        let data = v.get(T::key()).and_then(|data| {
            serde_json::from_value(data.clone()).expect("data item should be deserialized")
        });

        let res = Self {
            id,
            change_type: v.get("ct").and_then(|ct| {
                serde_json::from_value(ct.clone()).expect("ct should be deserialized")
            }),
            clock: v
                .get("clk")
                .and_then(|clk| clk.as_str())
                .map(|clk| Clock(clk.to_owned())),
            heartbeat_ms: v.get("heartbeatMs").and_then(serde_json::Value::as_i64),
            publish_time: v
                .get("pt")
                .and_then(serde_json::Value::as_i64)
                .and_then(|pt| Utc.timestamp_millis_opt(pt).latest()),
            initial_clock: v
                .get("initialClk")
                .and_then(|ic| ic.as_str())
                .map(|ic| InitialClock(ic.to_owned())),
            data,
            conflate_ms: v.get("conflateMs").and_then(serde_json::Value::as_i64),
            segment_type: v.get("segmentType").and_then(|st| {
                serde_json::from_value(st.clone()).expect("segmentType should be deserialized")
            }),
            status: v
                .get("status")
                .and_then(serde_json::Value::as_i64)
                .map(|s| s as i32),
        };

        Ok(res)
    }
}

use serde_json::Value;

#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Eq, Hash, Ord)]
pub struct UpdateSet2(pub Price, pub Size);

#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Eq, Hash, Ord)]
pub struct UpdateSet3(pub Position, pub Price, pub Size);

/// Represents the level of the order book.
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(not(feature = "fast-floats"), derive(Eq, Hash, Ord))]
pub struct Position(pub NumericPrimitive);

#[cfg(feature = "fast-floats")]
impl Eq for Position {}

#[cfg(feature = "fast-floats")]
impl Ord for Position {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

#[cfg(feature = "fast-floats")]
impl core::hash::Hash for Position {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

pub trait DataChange<T> {
    fn key() -> &'static str;
}

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
                connection_id: Some("206-221122192222-702491".to_owned()),
                id: None,
            })
        );
    }
}
