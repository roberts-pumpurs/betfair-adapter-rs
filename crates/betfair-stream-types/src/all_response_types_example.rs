use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllResponseTypesExample {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op_types: Option<OpTypes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_change_message: Option<Box<super::MarketChangeMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection: Option<Box<super::ConnectionMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_change_message: Option<Box<super::OrderChangeMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Box<super::StatusMessage>>,
}

impl AllResponseTypesExample {
    #[allow(dead_code)]
    pub fn new() -> AllResponseTypesExample {
        AllResponseTypesExample {
            op_types: None,
            market_change_message: None,
            connection: None,
            order_change_message: None,
            status: None,
        }
    }
}

///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OpTypes {
    Connection,
    Status,
    Mcm,
    Ocm,
}

impl Default for OpTypes {
    fn default() -> OpTypes {
        Self::Connection
    }
}
