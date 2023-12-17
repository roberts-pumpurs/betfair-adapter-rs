use serde::{Deserialize, Serialize};

use crate::market_subscription_message::StreamMarketFilterBettingType;
use crate::{Ct, SegmentType};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketChangeMessage {
    /// Client generated unique id to link request with response (like json rpc)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    /// Change Type - set to indicate the type of change - if null this is a delta)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ct: Option<Ct>,
    /// Token value (non-null) should be stored and passed in a MarketSubscriptionMessage to resume
    /// subscription (in case of disconnect)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clk: Option<String>,
    /// Heartbeat Milliseconds - the heartbeat rate (may differ from requested: bounds are 500 to
    /// 30000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_ms: Option<i64>,
    /// Publish Time (in millis since epoch) that the changes were generated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pt: Option<i64>,
    /// Token value (non-null) should be stored and passed in a MarketSubscriptionMessage to resume
    /// subscription (in case of disconnect)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_clk: Option<String>,
    /// MarketChanges - the modifications to markets (will be null on a heartbeat
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mc: Option<Vec<MarketChange>>,
    /// Conflate Milliseconds - the conflation rate (may differ from that requested if subscription
    /// is delayed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflate_ms: Option<i64>,
    /// Segment Type - if the change is split into multiple segments, this denotes the beginning
    /// and end of a change, and segments in between. Will be null if data is not segmented
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment_type: Option<SegmentType>,
    /// Stream status: set to null if the exchange stream data is up to date and 503 if the
    /// downstream services are experiencing latencies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketChange {
    /// Runner Changes - a list of changes to runners (or null if un-changed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rc: Option<Vec<RunnerChange>>,
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
    pub market_definition: Option<Box<MarketDefinition>>,
    /// Market Id - the id of the market
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketDefinition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub race_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settled_time: Option<String>,
    pub timezone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub each_way_divisor: Option<rust_decimal::Decimal>,

    /// The market regulators.
    pub regulators: Vec<String>,

    pub market_type: String,

    pub market_base_rate: rust_decimal::Decimal,

    pub number_of_winners: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,

    /// For Handicap and Line markets, the maximum value for the outcome, in market units for this
    /// market (eg 100 runs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_max_unit: Option<rust_decimal::Decimal>,

    pub in_play: bool,

    /// The number of seconds an order is held until it is submitted into the market. Orders are
    /// usually delayed when the market is in-play
    pub bet_delay: i32,

    pub bsp_market: bool,

    pub betting_type: StreamMarketFilterBettingType,

    pub number_of_active_runners: i32,

    /// For Handicap and Line markets, the minimum value for the outcome, in market units for this
    /// market (eg 0 runs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_min_unit: Option<rust_decimal::Decimal>,

    pub event_id: String,

    pub cross_matching: bool,

    pub runners_voidable: bool,

    pub turn_in_play_enabled: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_ladder_definition: Option<Box<PriceLadderDefinition>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_line_definition: Option<Box<KeyLineDefinition>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspend_time: Option<String>,

    pub discount_allowed: bool,

    pub persistence_enabled: bool,

    pub runners: Vec<RunnerDefinition>,

    pub version: i64,

    /// The Event Type the market is contained within.
    pub event_type_id: String,

    pub complete: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_date: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_time: Option<String>,

    pub bsp_reconciled: bool,

    /// For Handicap and Line markets, the lines available on this market will be between the range
    /// of lineMinUnit and lineMaxUnit, in increments of the lineInterval value. e.g. If unit is
    /// runs, lineMinUnit=10, lineMaxUnit=20 and lineInterval=0.5, then valid lines include 10,
    /// 10.5, 11, 11.5 up to 20 runs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_interval: Option<rust_decimal::Decimal>,

    pub status: StreamMarketDefinitionStatus,
}

///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamMarketDefinitionStatus {
    Inactive,
    Open,
    Suspended,
    Closed,
}

impl Default for StreamMarketDefinitionStatus {
    fn default() -> StreamMarketDefinitionStatus {
        Self::Inactive
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyLineDefinition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kl: Option<Vec<KeyLineSelection>>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyLineSelection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hc: Option<rust_decimal::Decimal>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceLadderDefinition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _type: Option<Type>,
}

#[derive(
    Clone, Copy, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Type {
    #[default]
    Classic,
    Finest,
    LineRange,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnerChange {
    /// The total amount matched. This value is truncated at 2dp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tv: Option<rust_decimal::Decimal>,

    /// Best Available To Back - LevelPriceVol triple delta of price changes, keyed by level (0 vol
    /// is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batb: Option<Vec<Vec<rust_decimal::Decimal>>>,

    /// Starting Price Back - PriceVol tuple delta of price changes (0 vol is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spb: Option<Vec<Vec<rust_decimal::Decimal>>>,

    /// Best Display Available To Lay (includes virtual prices)- LevelPriceVol triple delta of
    /// price changes, keyed by level (0 vol is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bdatl: Option<Vec<Vec<rust_decimal::Decimal>>>,

    /// Traded - PriceVol tuple delta of price changes (0 vol is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trd: Option<Vec<Vec<rust_decimal::Decimal>>>,

    /// Starting Price Far - The far starting price (or null if un-changed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spf: Option<rust_decimal::Decimal>,

    /// Last Traded Price - The last traded price (or null if un-changed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ltp: Option<rust_decimal::Decimal>,

    /// Available To Back - PriceVol tuple delta of price changes (0 vol is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atb: Option<Vec<Vec<rust_decimal::Decimal>>>,

    /// Starting Price Lay - PriceVol tuple delta of price changes (0 vol is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spl: Option<Vec<Vec<rust_decimal::Decimal>>>,

    /// Starting Price Near - The far starting price (or null if un-changed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spn: Option<rust_decimal::Decimal>,

    /// Available To Lay - PriceVol tuple delta of price changes (0 vol is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atl: Option<Vec<Vec<rust_decimal::Decimal>>>,

    /// Best Available To Lay - LevelPriceVol triple delta of price changes, keyed by level (0 vol
    /// is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batl: Option<Vec<Vec<rust_decimal::Decimal>>>,

    /// Selection Id - the id of the runner (selection)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>, // NOTE: Manually changed from i64 to u64

    /// Handicap - the handicap of the runner (selection) (null if not applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hc: Option<rust_decimal::Decimal>,

    /// Best Display Available To Back (includes virtual prices)- LevelPriceVol triple delta of
    /// price changes, keyed by level (0 vol is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bdatb: Option<Vec<Vec<rust_decimal::Decimal>>>,
}

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

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamRunnerDefinitionStatus {
    #[default]
    Active,
    Winner,
    Loser,
    Removed,
    RemovedVacant,
    Hidden,
    Placed,
}
