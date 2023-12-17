use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(
    TypedBuilder, Clone, Debug, PartialEq, PartialOrd, Eq, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct MarketDataFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub ladder_levels: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub fields: Option<Vec<Fields>>,
}

impl MarketDataFilter {
    #[allow(dead_code)]
    pub fn new() -> MarketDataFilter {
        MarketDataFilter { ladder_levels: None, fields: None }
    }
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Fields {
    ExBestOffersDisp,
    ExBestOffers,
    ExAllOffers,
    ExTraded,
    ExTradedVol,
    ExLtp,
    ExMarketDef,
    SpTraded,
    SpProjected,
}

impl Default for Fields {
    fn default() -> Fields {
        Self::ExBestOffersDisp
    }
}
