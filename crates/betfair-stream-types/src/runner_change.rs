use serde::{Deserialize, Serialize};


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

impl RunnerChange {
    #[allow(dead_code)]
    pub fn new() -> RunnerChange {
        RunnerChange {
            tv: None,
            batb: None,
            spb: None,
            bdatl: None,
            trd: None,
            spf: None,
            ltp: None,
            atb: None,
            spl: None,
            spn: None,
            atl: None,
            batl: None,
            id: None,
            hc: None,
            bdatb: None,
        }
    }
}
