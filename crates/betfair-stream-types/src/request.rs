use serde::{Deserialize, Serialize};

pub mod authentication_message;
pub mod heartbeat_message;
pub mod market_subscription_message;
pub mod order_subscription_message;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl RequestMessage {
    pub fn set_id(&mut self, id: i32) {
        match self {
            RequestMessage::Authentication(msg) => msg.id = Some(id),
            RequestMessage::Heartbeat(msg) => msg.id = Some(id),
            RequestMessage::MarketSubscription(msg) => msg.id = Some(id),
            RequestMessage::OrderSubscription(msg) => msg.id = Some(id),
        }
    }
}
