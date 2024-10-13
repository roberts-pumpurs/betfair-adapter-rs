use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A struct representing a size value using a Decimal.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Default,
)]
pub struct Size(Decimal);

impl Size {
    /// Creates a new `Size` instance with the given Decimal value.
    #[must_use]
    pub const fn new(size: Decimal) -> Self {
        Self(size)
    }

    /// Adds two `Size` instances, returning `None` if the operation overflows.
    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    /// Subtracts one `Size` from another, returning `None` if the operation overflows.
    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }

    /// Multiplies two `Size` instances, returning `None` if the operation overflows.
    pub fn checked_mul(&self, other: &Self) -> Option<Self> {
        self.0.checked_mul(other.0).map(Self)
    }

    /// Divides one `Size` by another, returning `None` if the operation overflows.
    pub fn checked_div(&self, other: &Self) -> Option<Self> {
        self.0.checked_div(other.0).map(Self)
    }

    /// Adds two `Size` instances, saturating at the numeric limits.
    #[must_use]
    pub fn saturating_add(&self, other: &Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Subtracts one `Size` from another, saturating at the numeric limits.
    #[must_use]
    pub fn saturating_sub(&self, other: &Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// Multiplies two `Size` instances, saturating at the numeric limits.
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
