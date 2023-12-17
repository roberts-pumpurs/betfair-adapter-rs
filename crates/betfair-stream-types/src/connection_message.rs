use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionMessage {
    /// The operation type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    /// Client generated unique id to link request with response (like json rpc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    /// The connection id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,
}

impl ConnectionMessage {
    #[allow(dead_code)]
    pub fn new() -> ConnectionMessage {
        ConnectionMessage { op: None, id: None, connection_id: None }
    }
}
