use serde::{Deserialize, Serialize};

/// Heartbeat message structure.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatMessage {
    /// Client generated unique id to link request with response (like json rpc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
}
