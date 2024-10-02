use serde::{Deserialize, Serialize};

/// Represents an authentication message for the Betfair API.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationMessage {
    /// Client generated unique id to link request with response (like json rpc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    
    /// The session token for the authenticated user.
    pub session: String,
    
    /// The application key for accessing the Betfair API.
    pub app_key: String,
}
