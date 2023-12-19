use betfair_types::types::sports_aping::{
    CountryCode, EventId, EventTypeId, MarketId, MarketType, Venue,
};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSubscriptionMessage {
    /// Client generated unique id to link request with response (like json rpc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    /// Segmentation Enabled - allow the server to send large sets of data in segments, instead of
    /// a single block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segmentation_enabled: Option<bool>,
    /// Token value delta (received in MarketChangeMessage) that should be passed to resume a
    /// subscription
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clk: Option<String>,
    /// Heartbeat Milliseconds - the heartbeat rate (looped back on initial image after validation:
    /// bounds are 500 to 5000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_ms: Option<i64>,
    /// Token value (received in initial MarketChangeMessage) that should be passed to resume a
    /// subscription
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_clk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_filter: Option<Box<MarketFilter>>,
    /// Conflate Milliseconds - the conflation rate (looped back on initial image after validation:
    /// bounds are 0 to 120000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflate_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_data_filter: Option<Box<MarketDataFilter>>,
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct MarketFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub country_codes: Option<Vec<CountryCode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub betting_types: Option<Vec<StreamMarketFilterBettingType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub turn_in_play_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub market_types: Option<Vec<MarketType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub venues: Option<Vec<Venue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub market_ids: Option<Vec<MarketId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub event_type_ids: Option<Vec<EventTypeId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub event_ids: Option<Vec<EventId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub bsp_market: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub race_types: Option<Vec<String>>,
}

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamMarketFilterBettingType {
    #[default]
    Odds,
    Line,
    Range,
    AsianHandicapDoubleLine,
    AsianHandicapSingleLine,
}
