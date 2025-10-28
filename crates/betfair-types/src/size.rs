use serde::{Deserialize, Serialize};

use crate::numeric::{NumericOps, NumericPrimitive};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default, Deserialize, Serialize)]
#[cfg_attr(not(feature = "fast-floats"), derive(Eq, Hash, Ord))]
pub struct Size(NumericPrimitive);

#[cfg(feature = "fast-floats")]
impl Eq for Size {}

#[cfg(feature = "fast-floats")]
impl Ord for Size {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

#[cfg(feature = "fast-floats")]
impl core::hash::Hash for Size {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Size {
    #[must_use]
    pub const fn new(size: NumericPrimitive) -> Self {
        Self(size)
    }

    #[must_use]
    pub fn zero() -> Self {
        Self(NumericPrimitive::zero())
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

impl From<NumericPrimitive> for Size {
    fn from(val: NumericPrimitive) -> Self {
        Self(val.round_2dp())
    }
}

impl From<Size> for NumericPrimitive {
    fn from(value: Size) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;
    use crate::num;

    #[rstest]
    #[case(num!(1.022192999293999))]
    fn size_gets_rounded_to_two_decimal_places(#[case] size_raw: NumericPrimitive) {
        let size = Size::from(size_raw);

        #[cfg(not(feature = "fast-floats"))]
        {
            assert_eq!(size.0, num!(1.02));
        }

        #[cfg(feature = "fast-floats")]
        {
            let expected = num!(1.02);
            let diff = (size.0 - expected).abs();
            assert!(
                diff < 1e-9,
                "Expected size to be rounded to 1.02, but got {}",
                size.0
            );
        }
    }
}
