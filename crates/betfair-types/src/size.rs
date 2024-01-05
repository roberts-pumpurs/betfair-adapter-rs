use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Size(Decimal);

impl Size {
    pub fn new(size: Decimal) -> Self {
        Self(size)
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

impl std::ops::Add for Size {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0 + rhs.0)
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
