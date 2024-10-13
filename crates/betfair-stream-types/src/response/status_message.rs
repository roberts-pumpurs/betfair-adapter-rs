use serde::{Deserialize, Serialize};

/// Represents the status message returned from the server.
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
    /// No application key provided
    #[default]
    NoAppKey,
    /// Invalid application key
    InvalidAppKey,
    /// No session available
    NoSession,
    /// Invalid session information
    InvalidSessionInformation,
    /// Not authorized to perform the action
    NotAuthorized,
    /// Invalid input provided
    InvalidInput,
    /// Invalid clock value
    InvalidClock,
    /// An unexpected error occurred
    UnexpectedError,
    /// Request timed out
    Timeout,
    /// Subscription limit exceeded
    SubscriptionLimitExceeded,
    /// Invalid request format
    InvalidRequest,
    /// Connection failed
    ConnectionFailed,
    /// Maximum connection limit exceeded
    MaxConnectionLimitExceeded,
    /// Too many requests made in a short period
    TooManyRequests,
}

/// The status of the last request
#[derive(
    Clone, Copy, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatusCode {
    /// Indicates that the request was successful
    #[default]
    Success,
    /// Indicates that the request failed
    Failure,
}
