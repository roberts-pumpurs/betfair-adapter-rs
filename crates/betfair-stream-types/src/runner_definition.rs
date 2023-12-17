use serde::{Deserialize, Serialize};



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnerDefinition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removal_date: Option<String>,
    /// Selection Id - the id of the runner (selection)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>, // NOTE manually altered
    /// Handicap - the handicap of the runner (selection) (null if not applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hc: Option<rust_decimal::Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjustment_factor: Option<rust_decimal::Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bsp: Option<rust_decimal::Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<StreamRunnerDefinitionStatus>,
}

impl RunnerDefinition {
    #[allow(dead_code)]
    pub fn new() -> RunnerDefinition {
        RunnerDefinition {
            sort_priority: None,
            removal_date: None,
            id: None,
            hc: None,
            adjustment_factor: None,
            bsp: None,
            status: None,
        }
    }
}

///
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamRunnerDefinitionStatus {
    Active,
    Winner,
    Loser,
    Removed,
    RemovedVacant,
    Hidden,
    Placed,
}

impl Default for StreamRunnerDefinitionStatus {
    fn default() -> StreamRunnerDefinitionStatus {
        Self::Active
    }
}
