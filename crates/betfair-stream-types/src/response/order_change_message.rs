use betfair_types::{customer_order_ref::CustomerOrderRef, types::sports_aping::MarketId};
use betfair_types::customer_strategy_ref::CustomerStrategyRef;
use betfair_types::price::Price;
use betfair_types::size::Size;
use betfair_types::types::sports_aping::{BetId, SelectionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{DataChange, DatasetChangeMessage, UpdateSet2};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderChangeMessage(pub DatasetChangeMessage<OrderMarketChange>);

impl DataChange<OrderMarketChange> for OrderMarketChange {
    fn key() -> &'static str {
        "oc"
    }
}

impl std::ops::Deref for OrderChangeMessage {
    type Target = DatasetChangeMessage<OrderMarketChange>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderMarketChange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<i64>,
    /// Order Changes - a list of changes to orders on a selection
    #[serde(rename = "orc")]
    pub order_runner_change: Option<Vec<OrderRunnerChange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
    pub market_id: MarketId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_image: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    pub handicap: Option<rust_decimal::Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_image: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    pub persistance_type: Pt,
    /// Order Type - the type of the order (L = LIMIT, MOC = MARKET_ON_CLOSE, LOC = LIMIT_ON_CLOSE)
    #[serde(rename = "ot")]
    pub order_type: Ot,
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
    #[serde(rename = "rc")]
    pub regulator_code: String,
    /// Size - the original placed size of the order
    #[serde(rename = "s")]
    pub size: Size,
    /// Placed Date - the date the order was placed
    #[serde(rename = "pd")]
    pub place_date: i64,
    /// Regulator Auth Code - the auth code returned by the regulator
    #[serde(rename = "rac")]
    pub regulator_auth_code: String,
    /// Matched Date - the date the order was matched (null if the order is not matched)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "md")]
    pub matched_date: Option<DateTime<Utc>>,
    /// Cancelled Date - the date the order was cancelled (null if the order is not cancelled)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "cd")]
    pub cancelled_date: Option<DateTime<Utc>>,
    /// Lapsed Date - the date the order was lapsed (null if the order is not lapsed)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ld")]
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
    #[serde(rename = "rfo")]
    pub order_reference: CustomerOrderRef,
    /// Bet Id - the id of the order
    pub id: BetId,
    /// BSP Liability - the BSP liability of the order (null if the order is not a BSP order)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bsp: Option<rust_decimal::Decimal>,
    /// Strategy Reference - the customer's strategy reference for this order (empty string if one
    /// was not set)
    #[serde(rename = "rfs")]
    pub strategy_reference: CustomerStrategyRef,
    /// Status - the status of the order (E = EXECUTABLE, EC = EXECUTION_COMPLETE)
    pub status: StreamOrderStatus,
    /// Size Remaining - the amount of the order that is remaining unmatched
    #[serde(rename = "sr")]
    pub size_remaining: Size,
}

/// Side - the side of the order. For Line markets a 'B' bet refers to a SELL line and an 'L' bet
/// refers to a BUY line.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    #[default]
    B,
    L,
}

/// Persistence Type - whether the order will persist at in play or not (L = LAPSE, P = PERSIST, MOC
/// = Market On Close)
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Pt {
    #[default]
    L,
    P,
    Moc,
}

/// Order Type - the type of the order (L = LIMIT, MOC = MARKET_ON_CLOSE, LOC = LIMIT_ON_CLOSE)
#[derive(
    Clone, Copy, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Ot {
    #[default]
    L,
    Loc,
    Moc,
}

/// Status - the status of the order (E = EXECUTABLE, EC = EXECUTION_COMPLETE)
#[derive(
    Clone, Copy, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamOrderStatus {
    #[default]
    E,
    Ec,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
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
