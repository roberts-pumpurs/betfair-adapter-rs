use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketChange {
    /// Runner Changes - a list of changes to runners (or null if un-changed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rc: Option<Vec<super::RunnerChange>>,
    /// Image - replace existing prices / data with the data supplied: it is not a delta (or null
    /// if delta)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub img: Option<bool>,
    /// The total amount matched across the market. This value is truncated at 2dp (or null if
    /// un-changed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tv: Option<rust_decimal::Decimal>,
    /// Conflated - have more than a single change been combined (or null if not conflated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub con: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_definition: Option<Box<super::MarketDefinition>>,
    /// Market Id - the id of the market
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}
