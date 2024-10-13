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
    pub account_id: Option<i64>, // The account ID associated with the order.

    /// Order Changes - a list of changes to orders on a selection.
    #[serde(rename = "orc")]
    pub order_runner_change: Option<Vec<OrderRunnerChange>>, /* List of changes to orders for a
                                                              * specific selection. */

    /// Closed - indicates if the market is closed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>, // True if the market is closed, otherwise false.

    /// Market ID - the identifier for the market associated with the order changes.
    pub market_id: MarketId, // The unique identifier for the market.

    /// Full Image - indicates if a full image of the order is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_image: Option<bool>, // True if a full image of the order is available.
}

/// Order Runner Change - represents changes to a specific runner's orders.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRunnerChange {
    /// Matched Backs - matched amounts by distinct matched price on the Back side for this runner
    /// (selection)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mb")]
    pub matched_backs: Option<Vec<UpdateSet2>>, // List of matched back amounts for this runner.

    /// Matched Lays - matched amounts by distinct matched price on the Lay side for this runner
    /// (selection)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ml")]
    pub matched_lays: Option<Vec<UpdateSet2>>, // List of matched lay amounts for this runner.

    /// Strategy Matches - Matched Backs and Matched Lays grouped by strategy reference
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "smc")]
    pub strategy_matches:
        Option<::std::collections::HashMap<CustomerStrategyRef, StrategyMatchChange>>, /* Strategy matches for this runner. */

    /// Unmatched Orders - orders on this runner (selection) that are not fully matched
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "uo")]
    pub unmatched_orders: Option<Vec<Order>>, // List of unmatched orders for this runner.

    /// Selection Id - the id of the runner (selection)
    pub id: SelectionId, // The unique identifier for the selection (runner).

    /// Handicap - the handicap of the runner (selection) (null if not applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hc")]
    pub handicap: Option<Handicap>, // The handicap value for the runner, if applicable.

    /// Indicates if the runner has a full image of the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_image: Option<bool>, // True if the runner has a full image of the order.
}

/// Order - represents a single order with its details.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Side - the side of the order. For Line markets a 'B' bet refers to a SELL line and an 'L'
    /// bet refers to a BUY line.
    pub side: Side, // The side of the order (Back or Lay).

    /// Size Voided - the amount of the order that has been voided
    #[serde(rename = "sv")]
    pub size_voided: Size, // The amount of the order that has been voided.

    /// Persistence Type - whether the order will persist at in play or not (L = LAPSE, P =
    /// PERSIST, MOC = Market On Close)
    #[serde(rename = "pt")]
    pub persistence_type: PersistenceType, // The persistence type of the order.

    /// Order Type - the type of the order (L = LIMIT, MOC = `MARKET_ON_CLOSE`, LOC =
    /// `LIMIT_ON_CLOSE`)
    #[serde(rename = "ot")]
    pub order_type: OrderType, // The type of the order.

    /// Lapse Status Reason Code - the reason that some or all of this order has been lapsed (null
    /// if no portion of the order is lapsed
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "lsrc")]
    pub lapse_status_reason_code: Option<String>, // Reason for the order lapse, if applicable.

    /// Price - the original placed price of the order. Line markets operate at even-money odds of
    /// 2.0. However, price for these markets refers to the line positions available as defined by
    /// the markets min-max range and interval steps
    #[serde(rename = "p")]
    pub price: Price, // The price at which the order was placed.

    /// Size Cancelled - the amount of the order that has been cancelled
    #[serde(rename = "sc")]
    pub size_cancelled: Size, // The amount of the order that has been cancelled.

    /// Regulator Code - the regulator of the order
    #[serde(rename = "rc")]
    pub regulator_code: String, // The regulator code for the order.

    /// Size - the original placed size of the order
    #[serde(rename = "s")]
    pub size: Size, // The original size of the order.

    /// The date the order was placed.
    #[serde(rename = "pd")]
    pub place_date: chrono::DateTime<chrono::Utc>, // The date and time the order was placed.

    /// Regulator Auth Code - the auth code returned by the regulator
    #[serde(rename = "rac")]
    pub regulator_auth_code: String, // The authorization code from the regulator.

    /// Matched Date - the date the order was matched (null if the order is not matched)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "md")]
    pub matched_date: Option<DateTime<Utc>>, // The date the order was matched, if applicable.

    /// Cancelled Date - the date the order was cancelled (null if the order is not cancelled)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "cd")]
    pub cancelled_date: Option<DateTime<Utc>>, // The date the order was cancelled, if applicable.

    /// Lapsed Date - the date the order was lapsed (null if the order is not lapsed)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ld")]
    pub lapsed_date: Option<DateTime<Utc>>, // The date the order was lapsed, if applicable.

    /// Size Lapsed - the amount of the order that has been lapsed
    #[serde(rename = "sl")]
    pub size_lapsed: Size, // The amount of the order that has been lapsed.

    /// Average Price Matched - the average price the order was matched at (null if the order is
    /// not matched). This value is not meaningful for activity on Line markets and is not
    /// guaranteed to be returned or maintained for these markets.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "avp")]
    pub average_price_matched: Option<Price>, // The average price at which the order was matched.

    /// Size Matched - the amount of the order that has been matched
    #[serde(rename = "sm")]
    pub size_matched: Size, // The amount of the order that has been matched.

    /// Order Reference - the customer's order reference for this order (empty string if one was
    /// not set)
    #[serde(rename = "rfo")]
    pub order_reference: CustomerOrderRef, // The customer's reference for this order.

    /// Bet Id - the id of the order
    pub id: BetId, // The unique identifier for the bet.

    /// BSP Liability - the BSP liability of the order (null if the order is not a BSP order)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bsp: Option<rust_decimal::Decimal>, // The BSP liability for the order, if applicable.

    /// Strategy Reference - the customer's strategy reference for this order (empty string if one
    /// was not set)
    #[serde(rename = "rfs")]
    pub strategy_reference: CustomerStrategyRef, /* The customer's strategy reference for this
                                                  * order. */

    /// Status - the status of the order (E = EXECUTABLE, EC = `EXECUTION_COMPLETE`)
    pub status: StreamOrderStatus, // The current status of the order.

    /// Size Remaining - the amount of the order that is remaining unmatched
    #[serde(rename = "sr")]
    pub size_remaining: Size, // The amount of the order that remains unmatched.
}

/// Side - the side of the order (Back or Lay).
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    /// The Back side of the order.
    #[default]
    #[serde(rename = "B")]
    Back,
    /// The Lay side of the order.
    #[serde(rename = "L")]
    Lay,
}

/// Persistence Type - indicates whether the order will persist at in play or not.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PersistenceType {
    /// The order will lapse.
    #[default]
    #[serde(rename = "L")]
    Lapse,
    /// The order will persist.
    #[serde(rename = "P")]
    Persist,
    /// The order will be matched at market on close.
    #[serde(rename = "MOC")]
    MarketOnClose,
}

/// Order Type - the type of the order (Limit, Market On Close, etc.).
#[derive(
    Clone, Copy, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    /// The order will be matched at limit.
    #[default]
    #[serde(rename = "L")]
    Limit,
    /// The order will be matched at limit on close.
    #[serde(rename = "LOC")]
    LimitOnClose,
    /// The order will be matched at market on close.
    #[serde(rename = "MOC")]
    MarketOnClose,
}

/// Stream Order Status - the status of the order (Executable, Execution Complete).
#[derive(
    Clone, Copy, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamOrderStatus {
    /// The order is executable.
    #[default]
    #[serde(rename = "E")]
    Executable,
    /// The order has been fully matched.
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
    pub mb: Option<Vec<UpdateSet2>>, // List of matched back amounts for this strategy.

    /// Matched Lays - matched amounts by distinct matched price on the Lay side for this strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ml: Option<Vec<UpdateSet2>>, // List of matched lay amounts for this strategy.
}
