use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyLineSelection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hc: Option<rust_decimal::Decimal>,
}

impl KeyLineSelection {
    #[allow(dead_code)]
    pub fn new() -> KeyLineSelection {
        KeyLineSelection { id: None, hc: None }
    }
}
