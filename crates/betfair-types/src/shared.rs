use serde::{Deserialize, Deserializer};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct SuccessResponse {
    pub token: redact::Secret<String>,
    pub product: redact::Secret<String>,
}

#[derive(Debug)]
pub struct Response(pub Result<SuccessResponse, ErrorValues>);

impl AsRef<Result<SuccessResponse, ErrorValues>> for Response {
    fn as_ref(&self) -> &Result<SuccessResponse, ErrorValues> {
        &self.0
    }
}

impl core::ops::Deref for Response {
    type Target = Result<SuccessResponse, ErrorValues>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Response {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'de> Deserialize<'de> for Response {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        // Check if it's a success response
        if let Some(status) = value.get("status").and_then(|v| v.as_str()) {
            match status {
                "SUCCESS" => {
                    let success_response = SuccessResponse {
                        token: redact::Secret::new(
                            value
                                .get("token")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| serde::de::Error::custom("invalid response"))?
                                .to_owned(),
                        ),
                        product: redact::Secret::new(
                            value
                                .get("product")
                                .and_then(|v| v.as_str())
                                .ok_or_else(|| serde::de::Error::custom("invalid response"))?
                                .to_owned(),
                        ),
                    };
                    return Ok(Self(Ok(success_response)));
                }
                "FAIL" => {
                    if let Some(error) = value.get("error") {
                        let login_error =
                            ErrorValues::deserialize(error).map_err(serde::de::Error::custom)?;
                        return Ok(Self(Err(login_error)));
                    }
                }
                _ => {}
            }
        }

        Err(serde::de::Error::custom("invalid response"))
    }
}
#[derive(Debug, Deserialize)]
pub enum StatusValues {
    #[serde(rename = "SUCCESS")]
    Success,
    #[serde(rename = "FAIL")]
    Failure,
}

#[derive(Debug, Deserialize)]
pub enum ErrorValues {
    #[serde(rename = "INPUT_VALIDATION_ERROR")]
    InputValidation,
    #[serde(rename = "INVALID_SESSION_INFORMATION")]
    InvalidSessionInformation,
    #[serde(rename = "NO_SESSION")]
    NoSession,
}
