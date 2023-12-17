use serde::{Deserialize, Serialize};

pub mod authentication_message;
pub mod connection_message;
pub mod heartbeat_message;
pub mod market_change_message;
pub mod market_subscription_message;
pub mod order_change_message;
pub mod order_subscription_message;
pub mod status_message;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op")]
#[serde(rename_all = "camelCase")]
pub enum RequestMessage {
    #[serde(rename = "authentication")]
    Authentication(authentication_message::AuthenticationMessage),
    #[serde(rename = "heartbeat")]
    Heartbeat(heartbeat_message::HeartbeatMessage),
    #[serde(rename = "marketSubscription")]
    MarketSubscription(market_subscription_message::MarketSubscriptionMessage),
    #[serde(rename = "orderSubscription")]
    OrderSubscription(order_subscription_message::OrderSubscriptionMessage),
}

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
#[derive(Clone, Copy, Default,Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Ct {
    #[default]
    SubImage,
    ResubDelta,
    Heartbeat,
}

/// Segment Type - if the change is split into multiple segments, this denotes the beginning and end
/// of a change, and segments in between. Will be null if data is not segmented
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SegmentType {
    #[default]
    SegStart,
    Seg,
    SegEnd,
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
                connection_id: Some("206-221122192222-702491".to_string()),
                id: None,
            })
        );
    }
}
