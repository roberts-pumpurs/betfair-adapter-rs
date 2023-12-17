use serde::{Deserialize, Serialize};

use super::{ErrorCode, StatusCode};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusMessage {
    /// The operation type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    /// Client generated unique id to link request with response (like json rpc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    /// The number of connections available for this account at this moment in time. Present on
    /// responses to Authentication messages only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connections_available: Option<i32>,
    /// Additional message in case of a failure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// The type of error in case of a failure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<ErrorCode>,
    /// The connection id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,
    /// Is the connection now closed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_closed: Option<bool>,
    /// The status of the last request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<StatusCode>,
}

impl StatusMessage {
    #[allow(dead_code)]
    pub fn new() -> StatusMessage {
        StatusMessage {
            op: None,
            id: None,
            connections_available: None,
            error_message: None,
            error_code: None,
            connection_id: None,
            connection_closed: None,
            status_code: None,
        }
    }
}
