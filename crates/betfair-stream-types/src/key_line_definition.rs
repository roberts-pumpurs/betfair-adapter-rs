use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyLineDefinition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kl: Option<Vec<super::KeyLineSelection>>,
}

impl KeyLineDefinition {
    #[allow(dead_code)]
    pub fn new() -> KeyLineDefinition {
        KeyLineDefinition { kl: None }
    }
}
