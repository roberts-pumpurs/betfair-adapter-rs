use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Default,
)]
pub struct Size(Decimal);

impl Size {
    #[must_use]
    pub const fn new(size: Decimal) -> Self {
        Self(size)
    }

    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }

    pub fn checked_mul(&self, other: &Self) -> Option<Self> {
        self.0.checked_mul(other.0).map(Self)
    }

    pub fn checked_div(&self, other: &Self) -> Option<Self> {
        self.0.checked_div(other.0).map(Self)
    }

    #[must_use]
    pub fn saturating_add(&self, other: &Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    #[must_use]
    pub fn saturating_sub(&self, other: &Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    #[must_use]
    pub fn saturating_mul(&self, other: &Self) -> Self {
        Self(self.0.saturating_mul(other.0))
    }
}

impl From<Decimal> for Size {
    fn from(val: Decimal) -> Self {
        const ROUND_TO: u32 = 2;
        Self(val.round_dp(ROUND_TO))
    }
}

impl From<Size> for Decimal {
    fn from(value: Size) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use rust_decimal_macros::*;

    use super::*;

    #[rstest]
    #[case(dec!(1.022192999293999))]
    fn size_gets_rounded_to_two_decimal_places(#[case] size_raw: Decimal) {
        let size = Size::from(size_raw);
        assert_eq!(size.0, dec!(1.02));
    }
}
