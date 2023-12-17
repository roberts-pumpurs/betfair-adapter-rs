use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationMessage {
    /// The operation type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    /// Client generated unique id to link request with response (like json rpc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_key: Option<String>,
}

impl AuthenticationMessage {
    #[allow(dead_code)]
    pub fn new() -> AuthenticationMessage {
        AuthenticationMessage { op: None, id: None, session: None, app_key: None }
    }
}
