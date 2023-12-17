use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllRequestTypesExample {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op_types: Option<OpTypes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat: Option<Box<super::HeartbeatMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_subscription_message: Option<Box<super::OrderSubscriptionMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_subscription: Option<Box<super::MarketSubscriptionMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<Box<super::AuthenticationMessage>>,
}

impl AllRequestTypesExample {
    #[allow(dead_code)]
    pub fn new() -> AllRequestTypesExample {
        AllRequestTypesExample {
            op_types: None,
            heartbeat: None,
            order_subscription_message: None,
            market_subscription: None,
            authentication: None,
        }
    }
}

///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OpTypes {
    Heartbeat,
    Authentication,
    MarketSubscription,
    OrderSubscription,
}

impl Default for OpTypes {
    fn default() -> OpTypes {
        Self::Heartbeat
    }
}
