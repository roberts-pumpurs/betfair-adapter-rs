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
    /// Token value delta (received in `MarketChangeMessage`) that should be passed to resume a
    /// subscription
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clk: Option<String>,
    /// Heartbeat Milliseconds - the heartbeat rate (looped back on initial image after validation:
    /// bounds are 500 to 5000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_ms: Option<i64>,
    /// Token value (received in initial `MarketChangeMessage`) that should be passed to resume a
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
/// Correctly configuring field filters can help by:
/// - Reducing the size (and time) of initial images
/// - Reducing the rate of change (as only changes matching your field filter are sent)
pub struct MarketDataFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    /// When bdatb and bdatl are sent with an empty array ("bdatb:[]"), this indicates
    /// that there's an update but this has been filtered out due to the "ladderLevels"
    /// marketDataFilter i.e. the update falls outside of the "ladderLevels" specified.
    pub ladder_levels: Option<LadderLevel>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub fields: Option<Vec<Fields>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct InvalidLadderLevel;

impl core::error::Error for InvalidLadderLevel {}
impl core::fmt::Display for InvalidLadderLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "`ladder_levels` must be between 1 and 10")
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Default, Serialize, Deserialize)]
// For depth-based ladders the number of levels to send (1 to 10). 1 is best price to back or lay
// etc.
pub struct LadderLevel(u8);

impl LadderLevel {
    pub fn new(level: u8) -> Result<Self, InvalidLadderLevel> {
        if !(1..=10).contains(&level) {
            return Err(InvalidLadderLevel);
        }

        Ok(Self(level))
    }
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
/// Filter Exchange market data:
/// - <https://support.developer.betfair.com/hc/en-us/articles/6540502258077-What-Betfair-Exchange-market-data-is-available-from-listMarketBook-Stream-API>-
/// - <https://docs.developer.betfair.com/display/1smk3cen4v3lu3yomq5qye0ni/Exchange+Stream+API#ExchangeStreamAPI-Marketdatafieldfiltering/MarketDataFilter>
pub enum Fields {
    /// - Fields: bdatb, bdatl
    /// - Type: level, price, size
    ///
    /// Best prices including Virtual Bets - depth is controlled by ladderLevels (1 to 10) - Please
    /// note: The virtual price stream is updated ~150 m/s after non-virtual prices. Virtual prices
    /// are calculated for all ladder levels.
    #[default]
    ExBestOffersDisp,
    /// - Fields: batb, batl
    /// - Type: level, price, size
    ///
    /// Best prices not including Virtual Bets - depth is controlled by ladderLevels (1 to 10).
    ExBestOffers,
    /// - Fields: atb, atl
    /// - Type: price, size
    ///
    /// Full available to BACK/LAY ladder.
    ExAllOffers,
    /// - Fields: trd
    /// - Type:  price, size
    ///
    /// Full traded ladder.This is the amount traded at any price on any
    /// selection in the market
    ExTraded,
    /// - Fields: tv
    /// - Type: size
    ///
    /// Market and runner level traded volume.
    ExTradedVol,
    /// - Fields: ltp
    /// - Type: price
    ///
    /// The "Last Price Matched" on a selection.
    ExLtp,
    /// - Fields: marketDefinition
    /// - Type: `MarketDefinition`
    ///
    /// Send market definitions. To receive updates to any of the following
    /// fields - `MarketDefinitionFields`
    ExMarketDef,
    /// - Fields: spb, spl
    /// - Type: price, size
    ///
    /// Starting price ladder.
    SpTraded,
    /// - Fields: spn, spf
    /// - Type: price
    ///
    /// Starting price projection prices. To receive any update to the Betfair SP Near and
    /// Far price.
    SpProjected,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize, TypedBuilder)]
#[serde(rename_all = "camelCase")]
/// Represents filters for market subscriptions.
///
/// This struct allows you to specify various criteria to filter the markets you want to subscribe
/// to.
///
/// Fields:
/// - `country_codes`: Optional list of country codes to filter markets by country.
/// - `betting_types`: Optional list of betting types to filter the markets.
/// - `turn_in_play_enabled`: Optional flag indicating if the market allows betting while the event
///   is in play.
/// - `market_types`: Optional list of market types to filter the markets.
/// - `venues`: Optional list of venues to filter the markets.
/// - `market_ids`: Optional list of specific market IDs to subscribe to.
/// - `event_type_ids`: Optional list of event type IDs to filter the markets.
/// - `event_ids`: Optional list of specific event IDs to subscribe to.
/// - `bsp_market`: Optional flag indicating if the market is a Best Starting Price market.
/// - `race_types`: Optional list of race types to filter the markets.
pub struct MarketFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    /// Optional list of country codes to filter markets by country.
    pub country_codes: Option<Vec<CountryCode>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    /// Optional list of betting types to filter the markets.
    pub betting_types: Option<Vec<StreamMarketFilterBettingType>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    /// Optional flag indicating if the market allows betting while the event is in play.
    pub turn_in_play_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    /// Optional list of market types to filter the markets.
    pub market_types: Option<Vec<MarketType>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    /// Optional list of venues to filter the markets.
    pub venues: Option<Vec<Venue>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    /// Optional list of specific market IDs to subscribe to.
    pub market_ids: Option<Vec<MarketId>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    /// Optional list of event type IDs to filter the markets.
    pub event_type_ids: Option<Vec<EventTypeId>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    /// Optional list of specific event IDs to subscribe to.
    pub event_ids: Option<Vec<EventId>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    /// Optional flag indicating if the market is a Best Starting Price market.
    pub bsp_market: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    /// Optional list of race types to filter the markets.
    pub race_types: Option<Vec<String>>,
}

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
/// Represents the betting type for market filtering.
pub enum StreamMarketFilterBettingType {
    /// Represents traditional odds betting.
    #[default]
    Odds,
    /// Represents line betting, where a line is set for the event.
    Line,
    /// Represents a range betting type.
    Range,
    /// Represents double line Asian handicap betting.
    AsianHandicapDoubleLine,
    /// Represents single line Asian handicap betting.
    AsianHandicapSingleLine,
}
