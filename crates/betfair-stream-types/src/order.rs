use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Side - the side of the order. For Line markets a 'B' bet refers to a SELL line and an 'L'
    /// bet refers to a BUY line.
    pub side: Side,
    /// Size Voided - the amount of the order that has been voided
    pub sv: rust_decimal::Decimal,
    /// Persistence Type - whether the order will persist at in play or not (L = LAPSE, P =
    /// PERSIST, MOC = Market On Close)
    pub pt: Pt,
    /// Order Type - the type of the order (L = LIMIT, MOC = MARKET_ON_CLOSE, LOC = LIMIT_ON_CLOSE)
    pub ot: Ot,
    /// Lapse Status Reason Code - the reason that some or all of this order has been lapsed (null
    /// if no portion of the order is lapsed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lsrc: Option<String>,
    /// Price - the original placed price of the order. Line markets operate at even-money odds of
    /// 2.0. However, price for these markets refers to the line positions available as defined by
    /// the markets min-max range and interval steps
    pub p: rust_decimal::Decimal,
    /// Size Cancelled - the amount of the order that has been cancelled
    pub sc: rust_decimal::Decimal,
    /// Regulator Code - the regulator of the order
    pub rc: String,
    /// Size - the original placed size of the order
    pub s: rust_decimal::Decimal,
    /// Placed Date - the date the order was placed
    pub pd: i64,
    /// Regulator Auth Code - the auth code returned by the regulator
    pub rac: String,
    /// Matched Date - the date the order was matched (null if the order is not matched)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md: Option<i64>,
    /// Cancelled Date - the date the order was cancelled (null if the order is not cancelled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cd: Option<i64>,
    /// Lapsed Date - the date the order was lapsed (null if the order is not lapsed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ld: Option<i64>,
    /// Size Lapsed - the amount of the order that has been lapsed
    pub sl: rust_decimal::Decimal,
    /// Average Price Matched - the average price the order was matched at (null if the order is
    /// not matched). This value is not meaningful for activity on Line markets and is not
    /// guaranteed to be returned or maintained for these markets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avp: Option<rust_decimal::Decimal>,
    /// Size Matched - the amount of the order that has been matched
    pub sm: rust_decimal::Decimal,
    /// Order Reference - the customer's order reference for this order (empty string if one was
    /// not set)
    pub rfo: String,
    /// Bet Id - the id of the order
    pub id: String,
    /// BSP Liability - the BSP liability of the order (null if the order is not a BSP order)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bsp: Option<rust_decimal::Decimal>,
    /// Strategy Reference - the customer's strategy reference for this order (empty string if one
    /// was not set)
    pub rfs: String,
    /// Status - the status of the order (E = EXECUTABLE, EC = EXECUTION_COMPLETE)
    pub status: StreamOrderStatus,
    /// Size Remaining - the amount of the order that is remaining unmatched
    pub sr: rust_decimal::Decimal,
}

/// Side - the side of the order. For Line markets a 'B' bet refers to a SELL line and an 'L' bet
/// refers to a BUY line.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    B,
    L,
}

impl Default for Side {
    fn default() -> Side {
        Self::B
    }
}
/// Persistence Type - whether the order will persist at in play or not (L = LAPSE, P = PERSIST, MOC
/// = Market On Close)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Pt {
    L,
    P,
    Moc,
}

impl Default for Pt {
    fn default() -> Pt {
        Self::L
    }
}
/// Order Type - the type of the order (L = LIMIT, MOC = MARKET_ON_CLOSE, LOC = LIMIT_ON_CLOSE)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Ot {
    L,
    Loc,
    Moc,
}

impl Default for Ot {
    fn default() -> Ot {
        Self::L
    }
}
/// Status - the status of the order (E = EXECUTABLE, EC = EXECUTION_COMPLETE)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamOrderStatus {
    E,
    Ec,
}

impl Default for StreamOrderStatus {
    fn default() -> StreamOrderStatus {
        Self::E
    }
}
