use serde::{Deserialize, Serialize};

/// Module for handling authentication messages.
pub mod authentication_message;

/// Module for handling heartbeat messages.
pub mod heartbeat_message;

/// Module for handling market subscription messages.
pub mod market_subscription_message;

/// Module for handling order subscription messages.
pub mod order_subscription_message;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op")]
#[serde(rename_all = "camelCase")]
/// Represents a request message.
pub enum RequestMessage {
    /// Represents an authentication message.
    #[serde(rename = "authentication")]
    Authentication(authentication_message::AuthenticationMessage),
    
    /// Represents a heartbeat message.
    #[serde(rename = "heartbeat")]
    Heartbeat(heartbeat_message::HeartbeatMessage),
    
    /// Represents a market subscription message.
    #[serde(rename = "marketSubscription")]
    MarketSubscription(market_subscription_message::MarketSubscriptionMessage),
    
    /// Represents an order subscription message.
    #[serde(rename = "orderSubscription")]
    OrderSubscription(order_subscription_message::OrderSubscriptionMessage),
}

impl RequestMessage {
    /// Sets the ID of the request message.
    pub fn set_id(&mut self, id: i32) {
        match self {
            Self::Authentication(msg) => msg.id = Some(id),
            Self::Heartbeat(msg) => msg.id = Some(id),
            Self::MarketSubscription(msg) => msg.id = Some(id),
            Self::OrderSubscription(msg) => msg.id = Some(id),
        }
    }
}
