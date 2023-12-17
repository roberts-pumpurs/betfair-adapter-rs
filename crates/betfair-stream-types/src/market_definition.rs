use serde::{Deserialize, Serialize};



use super::StreamMarketFilterBettingType;

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
    pub price_ladder_definition: Option<Box<super::PriceLadderDefinition>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_line_definition: Option<Box<super::KeyLineDefinition>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspend_time: Option<String>,

    pub discount_allowed: bool,

    pub persistence_enabled: bool,

    pub runners: Vec<super::RunnerDefinition>,

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
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
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
