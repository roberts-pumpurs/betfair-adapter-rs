use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderMarketChange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<i64>,
    /// Order Changes - a list of changes to orders on a selection
    pub orc: Option<Vec<super::OrderRunnerChange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_image: Option<bool>,
}
