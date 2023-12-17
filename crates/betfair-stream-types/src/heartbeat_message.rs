use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatMessage {
    /// The operation type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    /// Client generated unique id to link request with response (like json rpc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
}

impl HeartbeatMessage {
    #[allow(dead_code)]
    pub fn new() -> HeartbeatMessage {
        HeartbeatMessage { op: None, id: None }
    }
}
