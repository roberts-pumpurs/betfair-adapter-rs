use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationMessageAllOf {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_key: Option<String>,
}

impl AuthenticationMessageAllOf {
    #[allow(dead_code)]
    pub fn new() -> AuthenticationMessageAllOf {
        AuthenticationMessageAllOf { session: None, app_key: None }
    }
}
