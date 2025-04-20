//! Module for Betfair streaming API status messages.
//!
//! This module defines types for representing status messages received from the Betfair
//! streaming API, indicating the success or failure of operations such as authentication,
//! subscription, and other control messages.
use core::error::Error as StdError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Status message returned by the Betfair streaming API.
///
/// A status message indicates whether a request or operation was successful or failed.
///
/// # Variants
///
/// - `Success(StatusSuccess)`: Represents a successful operation with associated details.
/// - `Failure(StatusError)`: Represents a failed operation with error information.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "statusCode")]
#[serde(rename_all = "camelCase")]
pub enum StatusMessage {
    /// Success variant (authentication / heartbeat was successful)
    #[serde(rename = "SUCCESS")]
    Success(StatusSuccess),
    /// Failure variant (auth was not successful)
    #[serde(rename = "FAILURE")]
    Failure(StatusError),
}

/// Represents a successful status response in the Betfair streaming API.
///
/// Contains optional metadata such as request identifier, connection limits, and status flags.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusSuccess {
    /// Client generated unique id to link request with response (like json rpc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    /// The number of connections available for this account at this moment in time. Present on
    /// responses to Authentication messages only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connections_available: Option<i32>,
    /// The connection id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,
    /// Is the connection now closed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_closed: Option<bool>,
}

/// Represents a failed status response in the Betfair streaming API.
///
/// Contains an error code, optional error message, and connection metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusError {
    /// Client generated unique id to link request with response (like json rpc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    /// Additional message in case of a failure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// The type of error in case of a failure
    pub error_code: ErrorCode,
    /// The connection id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,
    /// Is the connection now closed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_closed: Option<bool>,
}

impl fmt::Display for StatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Betfair API error: {} (code: {:?})",
            self.error_message.as_deref().unwrap_or("Unknown error"),
            self.error_code
        )
    }
}

/// Implements the standard `Error` trait for `StatusError`.
impl StdError for StatusError {}

/// Error codes returned in a failed status response.
///
/// These codes describe the type of failure encountered.
#[derive(
    Clone, Copy, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// The request did not include an application key.
    #[default]
    NoAppKey,
    /// The provided application key is not recognized or is invalid.
    InvalidAppKey,
    /// The request did not include a session token.
    NoSession,
    /// The provided session token is invalid, expired, or unauthorized.
    InvalidSessionInformation,
    /// The session is not authorized to perform this operation.
    NotAuthorized,
    /// The input parameters for the request are invalid.
    InvalidInput,
    /// The client system clock is out of sync with the server.
    InvalidClock,
    /// An unexpected internal error occurred on the server.
    UnexpectedError,
    /// The request timed out waiting for a response.
    Timeout,
    /// The client has exceeded the maximum number of subscriptions allowed.
    SubscriptionLimitExceeded,
    /// The request is malformed or has invalid parameters.
    InvalidRequest,
    /// The connection to the streaming service failed.
    ConnectionFailed,
    /// The account has reached its maximum connection limit.
    MaxConnectionLimitExceeded,
    /// The client is making requests too rapidly and has been rate limited.
    TooManyRequests,
}

#[cfg(test)]
mod tests {
    use crate::response::ResponseMessage;

    use super::*;

    #[test]
    fn test_status_message_success_deserialization() {
        let json_str = r#"{
            "op": "status",
            "id": 1,
            "statusCode": "SUCCESS",
            "connectionClosed": false
        }"#;

        let status_message: StatusMessage = serde_json::from_str(json_str).unwrap();

        assert!(matches!(status_message, StatusMessage::Success(_)));
    }

    #[test]
    fn test_status_message_failure_deserialization() {
        let json_str = r#"{
            "op": "status",
            "id": 1,
            "statusCode": "FAILURE",
            "errorCode": "INVALID_SESSION_INFORMATION",
            "errorMessage": "Session expired or invalid"
        }"#;

        let status_message: StatusMessage = serde_json::from_str(json_str).unwrap();

        assert!(matches!(status_message, StatusMessage::Failure(_)));
    }

    #[test]
    fn test_status_message_failure_deserialization_2() {
        let json_str = r#"{
            "connectionClosed": true,
            "connectionId": "101-200425105305-1131705",
            "errorCode": "MAX_CONNECTION_LIMIT_EXCEEDED",
            "errorMessage": "You have exceeded your max connection limit which is: 10 connection(s).You currently have: 11 active connection(s).",
            "id": -1,
            "statusCode": "FAILURE"
        }"#;

        let status_message: StatusMessage = serde_json::from_str(json_str).unwrap();

        assert!(matches!(status_message, StatusMessage::Failure(_)));
    }

    #[test]
    fn test_status_message_success_serialization() {
        let success = StatusSuccess {
            id: Some(1),
            connection_closed: Some(false),
            connections_available: Some(1),
            connection_id: None,
        };
        let status_message = ResponseMessage::Status(StatusMessage::Success(success));

        let json_value = serde_json::to_value(&status_message).unwrap();

        assert_eq!(json_value["op"], "status");
        assert_eq!(json_value["id"], 1);
        assert_eq!(json_value["statusCode"], "SUCCESS");
        assert_eq!(json_value["connectionClosed"], false);
        assert!(!json_value.as_object().unwrap().contains_key("connectionId"));
    }

    #[test]
    fn test_status_message_failure_serialization() {
        let error = StatusError {
            id: Some(1),
            error_code: ErrorCode::InvalidSessionInformation,
            error_message: Some("Session expired or invalid".to_owned()),
            connection_id: None,
            connection_closed: None,
        };
        let status_message = ResponseMessage::Status(StatusMessage::Failure(error));

        let json_value = serde_json::to_value(&status_message).unwrap();

        assert_eq!(json_value["op"], "status");
        assert_eq!(json_value["id"], 1);
        assert_eq!(json_value["statusCode"], "FAILURE");
        assert_eq!(json_value["errorCode"], "INVALID_SESSION_INFORMATION");
        assert_eq!(json_value["errorMessage"], "Session expired or invalid");
    }
}
