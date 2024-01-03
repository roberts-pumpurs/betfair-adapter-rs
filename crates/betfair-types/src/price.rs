use std::ops::{Add, Deref, Div};

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum PriceParseError {
    #[error("InvalidPriceSpecified: {0}")]
    InvalidPriceSpecified(Decimal),
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Eq, Hash, Ord)]
pub struct Price(rust_decimal::Decimal);

impl Deref for Price {
    type Target = Decimal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Add for Price {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let lhs = self.0;
        let rhs = rhs.0;
        Self(lhs + rhs)
    }
}

impl Div<rust_decimal::Decimal> for Price {
    type Output = Self;

    fn div(self, rhs: rust_decimal::Decimal) -> Self::Output {
        let lhs = self.0;
        Self(lhs / rhs)
    }
}

impl From<Price> for rust_decimal::Decimal {
    fn from(value: Price) -> Self {
        value.0
    }
}

impl Price {
    pub fn new(price: rust_decimal::Decimal) -> Result<Self, PriceParseError> {
        let price = Price(Self::adjust_price_to_betfair_boundaries(price)?);
        Ok(price)
    }

    /// This function is unsafe because it does not check if the price is within the Betfair
    /// boundaries. Use `Price::new` instead.
    pub unsafe fn new_unchecked(price: rust_decimal::Decimal) -> Self {
        Price(price)
    }

    /// Betfair docs: https://docs.developer.betfair.com/pages/viewpage.action?pageId=6095894
    /// Below is a list of price increments per price 'group'.  Placing a bet outside of these
    /// increments will result in an INVALID_ODDS error
    ///
    /// Odds Markets
    /// ```markdown
    /// | Range      | Increment |
    /// | ---------- | --------- |
    /// | 1.01 → 2   | 0.01      |
    /// | 2→ 3       | 0.02      |
    /// | 3 → 4      | 0.05      |
    /// | 4 → 6      | 0.1       |
    /// | 6 → 10     | 0.2       |
    /// | 10 → 20    | 0.5       |
    /// | 20 → 30    | 1         |
    /// | 30 → 50    | 2         |
    /// | 50 → 100   | 5         |
    /// | 100 → 1000 | 10        |
    /// ```
    fn adjust_price_to_betfair_boundaries(
        current_price: rust_decimal::Decimal,
    ) -> Result<rust_decimal::Decimal, PriceParseError> {
        #[inline]
        fn round_to_nearest(
            x: rust_decimal::Decimal,
            lower_range: rust_decimal::Decimal,
            increment: rust_decimal::Decimal,
        ) -> rust_decimal::Decimal {
            // check if we need to round down
            let Some(remainder) = x.checked_rem(increment) else {
                panic!("Invalid price");
            };

            if remainder != dec!(0.0) {
                // check if we need to settle for the lowest range (to not underflow to the next
                // bottom range)
                if x - increment <= lower_range {
                    lower_range
                } else {
                    (x - remainder).round_dp(2)
                }
            } else {
                // TODO: remove the magic constant
                x.round_dp(2)
            }
        }
        match current_price {
            x if (dec!(1.01)..dec!(2.0)).contains(&x) => {
                Ok(round_to_nearest(x, dec!(1.01), dec!(0.01)))
            }
            x if (dec!(2.0)..dec!(3.0)).contains(&x) => {
                Ok(round_to_nearest(x, dec!(2.0), dec!(0.02)))
            }
            x if (dec!(3.0)..dec!(4.0)).contains(&x) => {
                Ok(round_to_nearest(x, dec!(3.0), dec!(0.05)))
            }
            x if (dec!(4.0)..dec!(6.0)).contains(&x) => {
                Ok(round_to_nearest(x, dec!(4.0), dec!(0.1)))
            }
            x if (dec!(6.0)..dec!(10.0)).contains(&x) => {
                Ok(round_to_nearest(x, dec!(6.0), dec!(0.2)))
            }
            x if (dec!(10.0)..dec!(20.0)).contains(&x) => {
                Ok(round_to_nearest(x, dec!(10.0), dec!(0.5)))
            }
            x if (dec!(20.0)..dec!(30.0)).contains(&x) => {
                Ok(round_to_nearest(x, dec!(20.0), dec!(1.0)))
            }
            x if (dec!(30.0)..dec!(50.0)).contains(&x) => {
                Ok(round_to_nearest(x, dec!(30.0), dec!(2.0)))
            }
            x if (dec!(50.0)..dec!(100.0)).contains(&x) => {
                Ok(round_to_nearest(x, dec!(50.0), dec!(5.0)))
            }
            x if (dec!(100.0)..dec!(1000.0)).contains(&x) => {
                Ok(round_to_nearest(x, dec!(100.0), dec!(10.0)))
            }
            x => Err(PriceParseError::InvalidPriceSpecified(x)),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use rust_decimal_macros::*;

    use super::*;

    #[rstest]
    #[case(dec!(0.99))]
    #[case(dec!(1.00))]
    #[case(dec!(11000.00))]
    fn correctly_detects_price_adjustment_errors(#[case] price: Decimal) {
        let actual = Price::adjust_price_to_betfair_boundaries(price).unwrap_err();

        let expected = PriceParseError::InvalidPriceSpecified(price);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(dec!(1.03), dec!(1.03))]
    #[case(dec!(1.034), dec!(1.03))]
    #[case(dec!(1.05432111), dec!(1.05))]
    // Second scale
    #[case(dec!(2.00032111), dec!(2.0))]
    #[case(dec!(2.13), dec!(2.12))]
    #[case(dec!(2.487), dec!(2.48))]
    #[case(dec!(2.4), dec!(2.4))]
    // Third scale
    #[case(dec!(3.00032111), dec!(3.0))]
    #[case(dec!(3.13), dec!(3.1))]
    #[case(dec!(3.487), dec!(3.45))]
    #[case(dec!(3.55), dec!(3.55))]
    // Fourth scale
    #[case(dec!(4.00032111), dec!(4.0))]
    #[case(dec!(4.13), dec!(4.1))]
    #[case(dec!(4.487), dec!(4.4))]
    #[case(dec!(5.00032111), dec!(5.0))]
    #[case(dec!(5.13), dec!(5.1))]
    #[case(dec!(5.487), dec!(5.4))]
    fn correctly_adjusts_prices(#[case] input_price: Decimal, #[case] expected: Decimal) {
        let actual = Price::adjust_price_to_betfair_boundaries(input_price).unwrap();
        assert_eq!(
            expected, actual,
            "Expected {input_price} to be adjusted to {expected}, but got {actual}"
        );
    }
}
