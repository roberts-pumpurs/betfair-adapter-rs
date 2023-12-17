use serde::{Deserialize, Serialize};

use super::{Ct, ErrorCode, SegmentType, StatusCode};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op")]
#[serde(rename_all = "camelCase")]
pub enum ResponseMessage {
    #[serde(rename = "connection")]
    Connection {
        /// Client generated unique id to link request with response (like json rpc)
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<i32>,
        /// The connection id
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "connectionId")]
        connection_id: Option<String>,
    },
    #[serde(rename = "mcm")]
    MarketChange {
        /// Client generated unique id to link request with response (like json rpc)
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<i32>,
        /// Change Type - set to indicate the type of change - if null this is a delta)
        #[serde(skip_serializing_if = "Option::is_none")]
        ct: Option<Ct>,
        /// Token value (non-null) should be stored and passed in a MarketSubscriptionMessage to
        /// resume subscription (in case of disconnect)
        #[serde(skip_serializing_if = "Option::is_none")]
        clk: Option<String>,
        /// Heartbeat Milliseconds - the heartbeat rate (may differ from requested: bounds are 500
        /// to 30000)
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "heartbeatMs")]
        heartbeat_ms: Option<i64>,
        /// Publish Time (in millis since epoch) that the changes were generated
        #[serde(skip_serializing_if = "Option::is_none")]
        pt: Option<i64>,
        /// Token value (non-null) should be stored and passed in a MarketSubscriptionMessage to
        /// resume subscription (in case of disconnect)
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "initialClk")]
        initial_clk: Option<String>,
        /// MarketChanges - the modifications to markets (will be null on a heartbeat
        #[serde(skip_serializing_if = "Option::is_none")]
        mc: Option<Vec<super::MarketChange>>,
        /// Conflate Milliseconds - the conflation rate (may differ from that requested if
        /// subscription is delayed)
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "conflateMs")]
        conflate_ms: Option<i64>,
        /// Segment Type - if the change is split into multiple segments, this denotes the
        /// beginning and end of a change, and segments in between. Will be null if data is not
        /// segmented
        #[serde(skip_serializing_if = "Option::is_none")]
        segment_type: Option<SegmentType>,
        /// Stream status: set to null if the exchange stream data is up to date and 503 if the
        /// downstream services are experiencing latencies
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<i32>,
    },
    #[serde(rename = "ocm")]
    OrderChange {
        /// Client generated unique id to link request with response (like json rpc)
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<i32>,
        /// Change Type - set to indicate the type of change - if null this is a delta)
        #[serde(skip_serializing_if = "Option::is_none")]
        ct: Option<Ct>,
        /// Token value (non-null) should be stored and passed in a MarketSubscriptionMessage to
        /// resume subscription (in case of disconnect)
        #[serde(skip_serializing_if = "Option::is_none")]
        clk: Option<String>,
        /// Heartbeat Milliseconds - the heartbeat rate (may differ from requested: bounds are 500
        /// to 30000)
        #[serde(skip_serializing_if = "Option::is_none")]
        heartbeat_ms: Option<i64>,
        /// Publish Time (in millis since epoch) that the changes were generated
        #[serde(skip_serializing_if = "Option::is_none")]
        pt: Option<i64>,
        /// OrderMarketChanges - the modifications to account's orders (will be null on a heartbeat
        #[serde(skip_serializing_if = "Option::is_none")]
        oc: Option<Vec<super::OrderMarketChange>>,
        /// Token value (non-null) should be stored and passed in a MarketSubscriptionMessage to
        /// resume subscription (in case of disconnect)
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "initialClk")]
        initial_clk: Option<String>,
        /// Conflate Milliseconds - the conflation rate (may differ from that requested if
        /// subscription is delayed)
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "conflateMs")]
        conflate_ms: Option<i64>,
        /// Segment Type - if the change is split into multiple segments, this denotes the
        /// beginning and end of a change, and segments in between. Will be null if data is not
        /// segmented
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "segmentType")]
        segment_type: Option<SegmentType>,
        /// Stream status: set to null if the exchange stream data is up to date and 503 if the
        /// downstream services are experiencing latencies
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<i32>,
    },
    #[serde(rename = "status")]
    StatusMessage {
        /// Client generated unique id to link request with response (like json rpc)
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<i32>,
        /// The number of connections available for this account at this moment in time. Present on
        /// responses to Authentication messages only.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "connectionsAvailable")]
        connections_available: Option<i32>,
        /// Additional message in case of a failure
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "errorMessage")]
        error_message: Option<String>,
        /// The type of error in case of a failure
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "errorCode")]
        error_code: Option<ErrorCode>,
        /// The connection id
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "connectionId")]
        connection_id: Option<String>,
        /// Is the connection now closed
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "connectionClosed")]
        connection_closed: Option<bool>,
        /// The status of the last request
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "statusCode")]
        status_code: Option<StatusCode>,
    },
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
            ResponseMessage::Connection {
                connection_id: Some("206-221122192222-702491".to_string()),
                id: None,
            }
        );
    }
}
