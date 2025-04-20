use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusMessage(pub Result<StatusSuccess, StatusError>);

impl AsRef<Result<StatusSuccess, StatusError>> for StatusMessage {
    fn as_ref(&self) -> &Result<StatusSuccess, StatusError> {
        &self.0
    }
}

impl core::ops::Deref for StatusMessage {
    type Target = Result<StatusSuccess, StatusError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for StatusMessage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'de> Deserialize<'de> for StatusMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        // Check if it's a success response
        if let Some(status) = value.get("statusCode").and_then(|v| v.as_str()) {
            match status {
                "SUCCESS" => {
                    let response =
                        StatusSuccess::deserialize(value).map_err(serde::de::Error::custom)?;
                    return Ok(Self(Ok(response)));
                }
                "FAIL" => {
                    let response =
                        StatusError::deserialize(value).map_err(serde::de::Error::custom)?;
                    return Ok(Self(Err(response)));
                }
                _ => {}
            }
        }

        Err(serde::de::Error::custom("invalid response"))
    }
}

impl Serialize for StatusMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            Ok(success) => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("status_code", "SUCCESS")?;

                if let Some(id) = &success.id {
                    map.serialize_entry("id", id)?;
                }
                if let Some(connections_available) = &success.connections_available {
                    map.serialize_entry("connectionsAvailable", connections_available)?;
                }
                if let Some(connection_id) = &success.connection_id {
                    map.serialize_entry("connectionId", connection_id)?;
                }
                if let Some(connection_closed) = &success.connection_closed {
                    map.serialize_entry("connectionClosed", connection_closed)?;
                }

                map.end()
            }
            Err(error) => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("status_code", "FAILURE")?;

                if let Some(id) = &error.id {
                    map.serialize_entry("id", id)?;
                }
                if let Some(error_message) = &error.error_message {
                    map.serialize_entry("errorMessage", error_message)?;
                }
                map.serialize_entry("errorCode", &error.error_code)?;
                if let Some(connection_id) = &error.connection_id {
                    map.serialize_entry("connectionId", connection_id)?;
                }
                if let Some(connection_closed) = &error.connection_closed {
                    map.serialize_entry("connectionClosed", connection_closed)?;
                }

                map.end()
            }
        }
    }
}

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

impl StdError for StatusError {}

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
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_status_message_success_deserialization() {
        let json_str = r#"{
            "op": "status",
            "id": 1,
            "statusCode": "SUCCESS",
            "connectionClosed": false
        }"#;

        let status_message: StatusMessage = serde_json::from_str(json_str).unwrap();

        assert!(status_message.is_ok());
        let success = status_message.as_ref().as_ref().unwrap();
        assert_eq!(success.id, Some(1));
        assert_eq!(success.connection_closed, Some(false));
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

        assert!(status_message.is_err());
        let error = status_message.as_ref().as_ref().unwrap_err();
        assert_eq!(error.id, Some(1));
        assert_eq!(error.error_code, ErrorCode::InvalidSessionInformation);
        assert_eq!(
            error.error_message,
            Some("Session expired or invalid".to_string())
        );
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

        assert!(status_message.is_err());
        let error = status_message.as_ref().as_ref().unwrap_err();
        assert_eq!(error.id, Some(1));
        assert_eq!(error.error_code, ErrorCode::InvalidSessionInformation);
        assert_eq!(
            error.error_message,
            Some("Session expired or invalid".to_string())
        );
    }

    #[test]
    fn test_status_message_success_serialization() {
        let success = StatusSuccess {
            id: Some(1),
            connection_closed: Some(false),
            connections_available: Some(1),
            connection_id: None,
        };
        let status_message = StatusMessage(Ok(success));

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
            error_message: Some("Session expired or invalid".to_string()),
            connection_id: None,
            connection_closed: None,
        };
        let status_message = StatusMessage(Err(error));

        let json_value = serde_json::to_value(&status_message).unwrap();

        assert_eq!(json_value["op"], "status");
        assert_eq!(json_value["id"], 1);
        assert_eq!(json_value["statusCode"], "FAILURE");
        assert_eq!(json_value["errorCode"], "INVALID_SESSION_INFORMATION");
        assert_eq!(json_value["errorMessage"], "Session expired or invalid");
    }
}
