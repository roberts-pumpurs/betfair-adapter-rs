use betfair_types::numeric::F64Ord;
use betfair_types::customer_order_ref::CustomerOrderRef;
use betfair_types::customer_strategy_ref::CustomerStrategyRef;
use betfair_types::handicap::Handicap;
use betfair_types::price::Price;
use betfair_types::size::Size;
use betfair_types::types::sports_aping::{BetId, MarketId, SelectionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{DataChange, DatasetChangeMessage, UpdateSet2};

/// Order Change Message - represents a message containing changes to orders.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderChangeMessage(pub DatasetChangeMessage<OrderMarketChange>);

impl DataChange<Self> for OrderMarketChange {
    fn key() -> &'static str {
        "oc"
    }
}

impl core::ops::Deref for OrderChangeMessage {
    type Target = DatasetChangeMessage<OrderMarketChange>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Order Market Change - represents changes to orders in a specific market.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderMarketChange {
    /// Account ID - the identifier for the account associated with the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<i64>,
    /// Order Changes - a list of changes to orders on a selection.
    #[serde(rename = "orc")]
    pub order_runner_change: Option<Vec<OrderRunnerChange>>,
    /// Closed - indicates if the market is closed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
    /// Market ID - the identifier for the market associated with the order changes.
    #[serde(rename = "id")]
    pub market_id: MarketId,
    /// Full Image - indicates if a full image of the order is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_image: Option<bool>,
}

/// Order Runner Change - represents changes to a specific runner's orders.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRunnerChange {
    /// Matched Backs - matched amounts by distinct matched price on the Back side for this runner
    /// (selection)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mb")]
    pub matched_backs: Option<Vec<UpdateSet2>>,
    /// Matched Lays - matched amounts by distinct matched price on the Lay side for this runner
    /// (selection)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ml")]
    pub matched_lays: Option<Vec<UpdateSet2>>,

    /// Strategy Matches - Matched Backs and Matched Lays grouped by strategy reference
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "smc")]
    pub strategy_matches:
        Option<::std::collections::HashMap<CustomerStrategyRef, StrategyMatchChange>>,
    /// Unmatched Orders - orders on this runner (selection) that are not fully matched
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "uo")]
    pub unmatched_orders: Option<Vec<Order>>,
    /// Selection Id - the id of the runner (selection)
    pub id: SelectionId,
    /// Handicap - the handicap of the runner (selection) (null if not applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hc")]
    pub handicap: Option<Handicap>,
    /// Indicates if the runner has a full image of the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_image: Option<bool>,
}

/// Order - represents a single order with its details.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Side - the side of the order. For Line markets a 'B' bet refers to a SELL line and an 'L'
    /// bet refers to a BUY line.
    pub side: Side,
    /// Size Voided - the amount of the order that has been voided
    #[serde(rename = "sv")]
    pub size_voided: Size,
    /// Persistence Type - whether the order will persist at in play or not (L = LAPSE, P =
    /// PERSIST, MOC = Market On Close)
    #[serde(rename = "pt")]
    pub persistence_type: PersistenceType,
    /// Order Type - the type of the order (L = LIMIT, MOC = `MARKET_ON_CLOSE`, LOC =
    /// `LIMIT_ON_CLOSE`)
    #[serde(rename = "ot")]
    pub order_type: OrderType,
    /// Lapse Status Reason Code - the reason that some or all of this order has been lapsed (null
    /// if no portion of the order is lapsed
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "lsrc")]
    pub lapse_status_reason_code: Option<String>,
    /// Price - the original placed price of the order. Line markets operate at even-money odds of
    /// 2.0. However, price for these markets refers to the line positions available as defined by
    /// the markets min-max range and interval steps
    #[serde(rename = "p")]
    pub price: Price,
    /// Size Cancelled - the amount of the order that has been cancelled
    #[serde(rename = "sc")]
    pub size_cancelled: Size,
    /// Regulator Code - the regulator of the order
    /// This is occasionally missing, so we default to an empty string.
    #[serde(rename = "rc", default)]
    pub regulator_code: String,
    /// Size - the original placed size of the order
    #[serde(rename = "s")]
    pub size: Size,
    /// The date the order was placed.
    #[serde(with = "ts_millis", rename = "pd")]
    pub place_date: chrono::DateTime<chrono::Utc>,
    /// Regulator Auth Code - the auth code returned by the regulator
    /// This is occasionally missing, so we default to an empty string.
    #[serde(rename = "rac", default)]
    pub regulator_auth_code: String,
    /// Matched Date - the date the order was matched (null if the order is not matched)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "ts_millis::option", default, rename = "md")]
    pub matched_date: Option<DateTime<Utc>>,
    /// Cancelled Date - the date the order was cancelled (null if the order is not cancelled)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "ts_millis::option", default, rename = "cd")]
    pub cancelled_date: Option<DateTime<Utc>>,
    /// Lapsed Date - the date the order was lapsed (null if the order is not lapsed)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "ts_millis::option", default, rename = "ld")]
    pub lapsed_date: Option<DateTime<Utc>>,
    /// Size Lapsed - the amount of the order that has been lapsed
    #[serde(rename = "sl")]
    pub size_lapsed: Size,
    /// Average Price Matched - the average price the order was matched at (null if the order is
    /// not matched). This value is not meaningful for activity on Line markets and is not
    /// guaranteed to be returned or maintained for these markets.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "avp")]
    pub average_price_matched: Option<Price>,
    /// Size Matched - the amount of the order that has been matched
    #[serde(rename = "sm")]
    pub size_matched: Size,
    /// Order Reference - the customer's order reference for this order (empty string if one was
    /// not set)
    #[serde(rename = "rfo", default)]
    pub order_reference: CustomerOrderRef,
    /// Bet Id - the id of the order
    pub id: BetId,
    /// BSP Liability - the BSP liability of the order (null if the order is not a BSP order)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(
        deserialize_with = "betfair_types::types::deserialize_f64_option",
        default
    )]
    pub bsp: Option<F64Ord>,
    /// Strategy Reference - the customer's strategy reference for this order (empty string if one
    /// was not set)
    #[serde(rename = "rfs", default)]
    pub strategy_reference: CustomerStrategyRef,
    /// Status - the status of the order (E = EXECUTABLE, EC = `EXECUTION_COMPLETE`)
    pub status: StreamOrderStatus,
    /// Size Remaining - the amount of the order that is remaining unmatched
    #[serde(rename = "sr")]
    pub size_remaining: Size,
}

/// Side - the side of the order (Back or Lay).
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    #[default]
    #[serde(rename = "B")]
    Back,
    #[serde(rename = "L")]
    Lay,
}

/// Persistence Type - indicates whether the order will persist at in play or not.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PersistenceType {
    #[default]
    #[serde(rename = "L")]
    Lapse,
    #[serde(rename = "P")]
    Persist,
    #[serde(rename = "MOC")]
    MarketOnClose,
}

/// Order Type - the type of the order (Limit, Market On Close, etc.).
#[derive(
    Clone, Copy, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    #[default]
    #[serde(rename = "L")]
    Limit,
    #[serde(rename = "LOC")]
    LimitOnClose,
    #[serde(rename = "MOC")]
    MarketOnClose,
}

/// Stream Order Status - the status of the order (Executable, Execution Complete).
#[derive(
    Clone, Copy, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamOrderStatus {
    #[default]
    #[serde(rename = "E")]
    Executable,
    #[serde(rename = "EC")]
    ExecutionComplete,
}

/// Strategy Match Change - represents changes in matched amounts by strategy.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyMatchChange {
    /// Matched Backs - matched amounts by distinct matched price on the Back side for this
    /// strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mb: Option<Vec<UpdateSet2>>,
    /// Matched Lays - matched amounts by distinct matched price on the Lay side for this strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ml: Option<Vec<UpdateSet2>>,
}

mod ts_millis {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    // ----- DateTime<Utc> as millis -----
    pub(crate) fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(date.timestamp_millis())
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = i64::deserialize(deserializer)?;
        Utc.timestamp_millis_opt(millis)
            .single()
            .ok_or_else(|| serde::de::Error::custom(format!("invalid timestamp: {}", millis)))
    }

    // ----- Option<DateTime<Utc>> as millis or null -----
    pub(crate) mod option {
        use super::*;
        use serde::{Deserialize, Deserializer, Serializer};

        pub(crate) fn serialize<S>(
            date: &Option<DateTime<Utc>>,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match date {
                Some(dt) => serializer.serialize_some(&dt.timestamp_millis()),
                None => serializer.serialize_none(),
            }
        }

        pub(crate) fn deserialize<'de, D>(
            deserializer: D,
        ) -> Result<Option<DateTime<Utc>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            // Accept either null or an i64
            let maybe_ms = Option::<i64>::deserialize(deserializer)?;
            match maybe_ms {
                Some(ms) => Utc
                    .timestamp_millis_opt(ms)
                    .single()
                    .map(Some)
                    .ok_or_else(|| serde::de::Error::custom(format!("invalid timestamp: {}", ms))),
                None => Ok(None),
            }
        }
    }
}
