use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusMessage {
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

/// The type of error in case of a failure
#[derive(
    Clone, Copy, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    #[default]
    NoAppKey,
    InvalidAppKey,
    NoSession,
    InvalidSessionInformation,
    NotAuthorized,
    InvalidInput,
    InvalidClock,
    UnexpectedError,
    Timeout,
    SubscriptionLimitExceeded,
    InvalidRequest,
    ConnectionFailed,
    MaxConnectionLimitExceeded,
    TooManyRequests,
}

/// The status of the last request
#[derive(
    Clone, Copy, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatusCode {
    #[default]
    Success,
    Failure,
}
