use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionMessageAllOf {
    /// The connection id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,
}

impl ConnectionMessageAllOf {
    #[allow(dead_code)]
    pub fn new() -> ConnectionMessageAllOf {
        ConnectionMessageAllOf { connection_id: None }
    }
}
