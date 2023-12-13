use redact::Secret;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct KeepAliveResponse {
    pub token: Secret<String>,
    pub product: Secret<String>,
    pub status: StatusValues,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorValues>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum StatusValues {
    #[serde(rename = "SUCCESS")]
    Success,
    #[serde(rename = "FAIL")]
    Failure,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum ErrorValues {
    #[serde(rename = "INPUT_VALIDATION_ERROR")]
    InputValidation,
    #[serde(rename = "INTERNAL_ERROR")]
    InternalServer,
    #[serde(rename = "NO_SESSION")]
    NoSession,
}
