use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyMatchChange {
    /// Matched Backs - matched amounts by distinct matched price on the Back side for this
    /// strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mb: Option<Vec<Vec<rust_decimal::Decimal>>>,
    /// Matched Lays - matched amounts by distinct matched price on the Lay side for this strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ml: Option<Vec<Vec<rust_decimal::Decimal>>>,
}

impl StrategyMatchChange {
    #[allow(dead_code)]
    pub fn new() -> StrategyMatchChange {
        StrategyMatchChange { mb: None, ml: None }
    }
}
