use betfair_types::NumericOrdPrimitive;
use betfair_types::price::Price;
use betfair_types::size::Size;
use betfair_types::types::sports_aping::{MarketId, SelectionId};
use serde::{Deserialize, Serialize};

use super::{DataChange, DatasetChangeMessage, UpdateSet2, UpdateSet3};
use crate::request::market_subscription_message::StreamMarketFilterBettingType;

/// Represents a market change message.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MarketChangeMessage(pub DatasetChangeMessage<MarketChange>);

/// Trait for data change operations.
impl DataChange<Self> for MarketChange {
    fn key() -> &'static str {
        "mc"
    }
}

/// Implements Deref for `MarketChangeMessage`.
impl core::ops::Deref for MarketChangeMessage {
    type Target = DatasetChangeMessage<MarketChange>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Represents a market change.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketChange {
    /// Runner Changes - a list of changes to runners (or null if un-changed)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "rc")]
    pub runner_change: Option<Vec<RunnerChange>>,
    /// Image - replace existing prices / data with the data supplied: it is not a delta (or null
    /// if delta)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "img")]
    pub full_image: Option<bool>,
    /// The total amount matched across the market. This value is truncated at 2dp (or null if
    /// un-changed)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "tv")]
    pub total_value: Option<Size>,
    /// Conflated - have more than a single change been combined (or null if not conflated)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "con")]
    pub conflated: Option<bool>,
    /// Market definition details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_definition: Option<Box<MarketDefinition>>,
    /// Market Id - the id of the market
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id")]
    pub market_id: Option<MarketId>,
}

/// Represents the market definition.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketDefinition {
    /// The venue of the market.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venue: Option<String>,
    /// The type of race.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub race_type: Option<String>,
    /// The time the market was settled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settled_time: Option<String>,
    /// The timezone of the market.
    pub timezone: String,
    /// The divisor for each way betting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub each_way_divisor: Option<NumericOrdPrimitive>,

    /// The market regulators.
    pub regulators: Vec<String>,

    /// The type of market.
    pub market_type: String,

    /// The base rate for the market.
    pub market_base_rate: NumericOrdPrimitive,

    /// The number of winners in the market.
    pub number_of_winners: i32,

    /// The country code for the market.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,

    /// For Handicap and Line markets, the maximum value for the outcome, in market units for this
    /// market (eg 100 runs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_max_unit: Option<NumericOrdPrimitive>,

    /// Indicates if the market is in play.
    pub in_play: bool,

    /// The number of seconds an order is held until it is submitted into the market. Orders are
    /// usually delayed when the market is in-play
    pub bet_delay: i32,

    /// Indicates if the market is a BSP market.
    pub bsp_market: bool,

    /// The betting type for the market.
    pub betting_type: StreamMarketFilterBettingType,

    /// The number of active runners in the market.
    pub number_of_active_runners: i32,

    /// For Handicap and Line markets, the minimum value for the outcome, in market units for this
    /// market (eg 0 runs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_min_unit: Option<NumericOrdPrimitive>,

    /// The event ID associated with the market.
    pub event_id: String,

    /// Indicates if cross matching is enabled.
    pub cross_matching: bool,

    /// Indicates if runners can be voided.
    pub runners_voidable: bool,

    /// Indicates if turning in play is enabled.
    pub turn_in_play_enabled: bool,

    /// The price ladder definition for the market.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_ladder_definition: Option<Box<PriceLadderDefinition>>,

    /// The key line definition for the market.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_line_definition: Option<Box<KeyLineDefinition>>,

    /// The time the market was suspended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspend_time: Option<String>,

    /// Indicates if discounts are allowed.
    pub discount_allowed: bool,

    /// Indicates if persistence is enabled.
    pub persistence_enabled: bool,

    /// The list of runners in the market.
    pub runners: Vec<RunnerDefinition>,

    /// The version of the market definition.
    pub version: i64,

    /// The Event Type the market is contained within.
    pub event_type_id: String,

    /// Indicates if the market is complete.
    pub complete: bool,

    /// The open date of the market.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_date: Option<String>,

    /// The time of the market.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_time: Option<String>,

    /// Indicates if the BSP has been reconciled.
    pub bsp_reconciled: bool,

    /// For Handicap and Line markets, the lines available on this market will be between the range
    /// of lineMinUnit and lineMaxUnit, in increments of the lineInterval value. e.g. If unit is
    /// runs, lineMinUnit=10, lineMaxUnit=20 and lineInterval=0.5, then valid lines include 10,
    /// 10.5, 11, 11.5 up to 20 runs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_interval: Option<NumericOrdPrimitive>,

    /// The status of the market.
    pub status: StreamMarketDefinitionStatus,
}

/// Represents the status of a market definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamMarketDefinitionStatus {
    Inactive,
    Open,
    Suspended,
    Closed,
}

impl Default for StreamMarketDefinitionStatus {
    fn default() -> Self {
        Self::Inactive
    }
}

/// Represents the key line definition.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyLineDefinition {
    /// The key line selections.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "kl")]
    pub key_line: Option<Vec<KeyLineSelection>>,
}

/// Represents a selection in the key line.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyLineSelection {
    /// The ID of the key line selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// The handicap value for the selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hc")]
    pub handicap: Option<NumericOrdPrimitive>,
}

/// Represents the price ladder definition.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceLadderDefinition {
    /// The type of price ladder.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub r#type: Option<Type>,
}

/// Represents the type of price ladder.
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

/// Represents a change in runner information.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunnerChange {
    /// The total amount matched. This value is truncated at 2dp.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "tv")]
    pub total_value: Option<Size>,

    /// Best Available To Back - `LevelPriceVol` triple delta of price changes, keyed by level (0
    /// vol is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "batb")]
    pub best_available_to_back: Option<Vec<UpdateSet3>>,

    /// Starting Price Back - `PriceVol` tuple delta of price changes (0 vol is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "spb")]
    pub starting_price_back: Option<Vec<UpdateSet2>>,

    /// Best Display Available To Lay (includes virtual prices)- `LevelPriceVol` triple delta of
    /// price changes, keyed by level (0 vol is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bdatl")]
    pub best_display_available_to_lay: Option<Vec<UpdateSet3>>,

    /// Traded - `PriceVol` tuple delta of price changes (0 vol is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "trd")]
    pub traded: Option<Vec<UpdateSet2>>,

    /// Starting Price Far - The far starting price (or null if un-changed)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "spf")]
    pub starting_price_far: Option<Price>,

    /// Last Traded Price - The last traded price (or null if un-changed)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ltp")]
    pub last_traded_price: Option<Price>,

    /// Available To Back - `PriceVol` tuple delta of price changes (0 vol is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "atb")]
    pub available_to_back: Option<Vec<UpdateSet2>>,

    /// Starting Price Lay - `PriceVol` tuple delta of price changes (0 vol is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "spl")]
    pub starting_price_lay: Option<Vec<UpdateSet2>>,

    /// Starting Price Near - The far starting price (or null if un-changed)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "spn")]
    pub starting_price_near: Option<Price>,

    /// Available To Lay - `PriceVol` tuple delta of price changes (0 vol is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "atl")]
    pub available_to_lay: Option<Vec<UpdateSet2>>,

    /// Best Available To Lay - `LevelPriceVol` triple delta of price changes, keyed by level (0
    /// vol is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "batl")]
    pub best_available_to_lay: Option<Vec<UpdateSet3>>,

    /// Selection Id - the id of the runner (selection)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id")]
    pub id: Option<SelectionId>,

    /// Handicap - the handicap of the runner (selection) (null if not applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hc")]
    pub handicap: Option<NumericOrdPrimitive>,

    /// Best Display Available To Back (includes virtual prices)- `LevelPriceVol` triple delta of
    /// price changes, keyed by level (0 vol is remove)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bdatb")]
    pub best_display_available_to_back: Option<Vec<UpdateSet3>>,
}

/// Represents the definition of a runner.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RunnerDefinition {
    /// The sort priority of the runner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_priority: Option<i32>,
    /// The removal date of the runner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removal_date: Option<String>,
    /// Selection Id - the id of the runner (selection)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<SelectionId>,
    /// Handicap - the handicap of the runner (selection) (null if not applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hc")]
    pub handicap: Option<NumericOrdPrimitive>,
    /// The adjustment factor for the runner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjustment_factor: Option<NumericOrdPrimitive>,
    /// The BSP value for the runner.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bsp")]
    pub bsp: Option<NumericOrdPrimitive>,
    /// The status of the runner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<StreamRunnerDefinitionStatus>,
}

/// Implements comparison for `RunnerDefinition`.
impl PartialOrd for RunnerDefinition {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Implements ordering for `RunnerDefinition`.
impl Ord for RunnerDefinition {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.sort_priority.cmp(&other.sort_priority)
    }
}

/// Represents the status of a runner definition.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamRunnerDefinitionStatus {
    /// The runner is currently active in the market.
    #[default]
    Active,
    /// The runner has won the event.
    Winner,
    /// The runner has lost the event.
    Loser,
    /// The runner has been removed from the market.
    Removed,
    /// The runner was removed but is considered vacant.
    RemovedVacant,
    /// The runner is hidden from the market view.
    Hidden,
    /// The runner has been placed in the market.
    Placed,
}
