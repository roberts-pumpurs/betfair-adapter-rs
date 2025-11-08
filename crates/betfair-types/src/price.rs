use core::ops::{Add, Div, Mul, Sub};

use serde::{Deserialize, Serialize};

use crate::numeric::{NumericOps, NumericOrdPrimitive, NumericPrimitive};

#[derive(thiserror::Error, Clone, Debug, PartialEq, Eq)]
pub enum PriceParseError {
    #[error("InvalidPriceSpecified: {0}")]
    InvalidPriceSpecified(NumericOrdPrimitive),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Price(NumericPrimitive);

impl PartialEq for Price {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for Price {}

impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Price {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl core::hash::Hash for Price {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
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

impl Sub for Price {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let lhs = self.0;
        let rhs = rhs.0;
        Self(lhs - rhs)
    }
}

impl Mul for Price {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let lhs = self.0;
        let rhs = rhs.0;
        Self(lhs * rhs)
    }
}

impl Div<NumericPrimitive> for Price {
    type Output = Self;

    fn div(self, rhs: NumericPrimitive) -> Self::Output {
        let lhs = self.0;
        Self(lhs / rhs)
    }
}

impl From<Price> for NumericPrimitive {
    fn from(value: Price) -> Self {
        value.0
    }
}

impl Price {
    pub fn new(price: NumericPrimitive) -> Result<Self, PriceParseError> {
        let price = Self(Self::adjust_price_to_betfair_boundaries(price)?);
        Ok(price)
    }

    /// This function is unsafe because it does not check if the price is within the Betfair
    /// boundaries. Use `Price::new` instead.
    /// # Safety
    /// The caller must ensure that the price is within the Betfair boundaries.
    #[must_use]
    pub const unsafe fn new_unchecked(price: NumericPrimitive) -> Self {
        Self(price)
    }

    pub const fn as_f64(&self) -> f64 {
        self.0
    }

    /// Betfair docs: <https://docs.developer.betfair.com/pages/viewpage.action?pageId=6095894>
    /// Below is a list of price increments per price 'group'.  Placing a bet outside of these
    /// increments will result in an `INVALID_ODDS` error
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
        current_price: NumericPrimitive,
    ) -> Result<NumericPrimitive, PriceParseError> {
        #[inline]
        fn round_to_nearest(
            x: NumericPrimitive,
            lower_range: NumericPrimitive,
            increment: NumericPrimitive,
        ) -> NumericPrimitive {
            // For f64, round to nearest increment to avoid floating-point precision issues
            let steps_raw = (x - lower_range) / increment;
            let steps = steps_raw.round();
            let rounded = (lower_range + (steps * increment)).round_2dp();

            // Check if the original value is already very close to the rounded value
            // (within floating-point tolerance), if so, use the rounded value
            let diff = (x - rounded).abs();
            if diff < 1e-9 {
                return rounded;
            }

            // Otherwise, check if we need to round down
            let steps_down = steps_raw.floor();
            let rounded_down = (lower_range + (steps_down * increment)).round_2dp();

            // Ensure we don't go below the lower range
            if rounded_down < lower_range {
                lower_range
            } else {
                rounded_down
            }
        }

        use crate::num;

        match current_price {
            x if (num!(1.01)..num!(2.0)).contains(&x) => {
                Ok(round_to_nearest(x, num!(1.01), num!(0.01)))
            }
            x if (num!(2.0)..num!(3.0)).contains(&x) => {
                Ok(round_to_nearest(x, num!(2.0), num!(0.02)))
            }
            x if (num!(3.0)..num!(4.0)).contains(&x) => {
                Ok(round_to_nearest(x, num!(3.0), num!(0.05)))
            }
            x if (num!(4.0)..num!(6.0)).contains(&x) => {
                Ok(round_to_nearest(x, num!(4.0), num!(0.1)))
            }
            x if (num!(6.0)..num!(10.0)).contains(&x) => {
                Ok(round_to_nearest(x, num!(6.0), num!(0.2)))
            }
            x if (num!(10.0)..num!(20.0)).contains(&x) => {
                Ok(round_to_nearest(x, num!(10.0), num!(0.5)))
            }
            x if (num!(20.0)..num!(30.0)).contains(&x) => {
                Ok(round_to_nearest(x, num!(20.0), num!(1.0)))
            }
            x if (num!(30.0)..num!(50.0)).contains(&x) => {
                Ok(round_to_nearest(x, num!(30.0), num!(2.0)))
            }
            x if (num!(50.0)..num!(100.0)).contains(&x) => {
                Ok(round_to_nearest(x, num!(50.0), num!(5.0)))
            }
            x if (num!(100.0)..=num!(1000.0)).contains(&x) => {
                Ok(round_to_nearest(x, num!(100.0), num!(10.0)))
            }
            x => Err(PriceParseError::InvalidPriceSpecified(
                crate::numeric::F64Ord(x),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;
    use crate::num;
    use std::cmp::Ordering;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn price_should_use_bitwise_equality() {
        let nan = Price(f64::NAN);
        assert_eq!(nan, Price(f64::NAN)); // This would fail for a normal f64.

        assert_eq!(Price(-0.0).cmp(&Price(0.0)), Ordering::Less);
        assert_ne!(Price(-0.0), Price(0.0));

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        nan.hash(&mut h1);
        nan.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[rstest]
    #[case(num!(0.99))]
    #[case(num!(1.00))]
    #[case(num!(1000.01))]
    #[case(num!(11000.00))]
    fn correctly_detects_price_adjustment_errors(#[case] price: NumericPrimitive) {
        let actual = Price::adjust_price_to_betfair_boundaries(price).unwrap_err();

        let expected = PriceParseError::InvalidPriceSpecified(price.into());
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case(num!(1.01), num!(1.01))]
    #[case(num!(1.03), num!(1.03))]
    #[case(num!(1.034), num!(1.03))]
    #[case(num!(1.05432111), num!(1.05))]
    // Second scale
    #[case(num!(2.00032111), num!(2.0))]
    #[case(num!(2.13), num!(2.12))]
    #[case(num!(2.487), num!(2.48))]
    #[case(num!(2.4), num!(2.4))]
    // Third scale
    #[case(num!(3.00032111), num!(3.0))]
    #[case(num!(3.13), num!(3.1))]
    #[case(num!(3.487), num!(3.45))]
    #[case(num!(3.55), num!(3.55))]
    // Fourth scale
    #[case(num!(4.00032111), num!(4.0))]
    #[case(num!(4.13), num!(4.1))]
    #[case(num!(4.487), num!(4.4))]
    #[case(num!(5.00032111), num!(5.0))]
    #[case(num!(5.13), num!(5.1))]
    #[case(num!(5.487), num!(5.4))]
    #[case(num!(999.0), num!(990.0))]
    #[case(num!(1000.0), num!(1000.0))]
    fn correctly_adjusts_prices(
        #[case] input_price: NumericPrimitive,
        #[case] expected: NumericPrimitive,
    ) {
        let actual = Price::adjust_price_to_betfair_boundaries(input_price).unwrap();

        let diff = (expected - actual).abs();
        assert!(
            diff < 1e-9,
            "Expected {input_price} to be adjusted to {expected}, but got {actual} (diff: {diff})"
        );
    }
}

#[cfg(test)]
mod price_serialization_tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    fn get_all_prices() -> Vec<f64> {
        let mut prices = Vec::new();

        let price_ranges = [
            (1.01_f64, 2.0_f64, 0.01_f64),     // 1.01 → 2 | 0.01
            (2.0_f64, 3.0_f64, 0.02_f64),      // 2 → 3 | 0.02
            (3.0_f64, 4.0_f64, 0.05_f64),      // 3 → 4 | 0.05
            (4.0_f64, 6.0_f64, 0.1_f64),       // 4 → 6 | 0.1
            (6.0_f64, 10.0_f64, 0.2_f64),      // 6 → 10 | 0.2
            (10.0_f64, 20.0_f64, 0.5_f64),     // 10 → 20 | 0.5
            (20.0_f64, 30.0_f64, 1.0_f64),     // 20 → 30 | 1
            (30.0_f64, 50.0_f64, 2.0_f64),     // 30 → 50 | 2
            (50.0_f64, 100.0_f64, 5.0_f64),    // 50 → 100 | 5
            (100.0_f64, 1001.0_f64, 10.0_f64), // 100 → 1000 | 10 (inclusive)
        ];

        for (start, end, step) in price_ranges {
            let mut price = start;
            while price < end {
                prices.push(price.round_2dp());
                price += step;
            }
        }

        prices
    }

    #[test]
    fn get_all_prices_generates_valid_prices() {
        for price in get_all_prices() {
            let valid_price = Price::new(price).unwrap();
            assert_eq!(valid_price.0, price);
        }
    }

    fn check_decimal_places(value_str: &str, max_decimal_places: usize) {
        let parts: Vec<&str> = value_str.split('.').collect();
        assert!(parts.len() <= 2);
        if parts.len() == 2 && parts[1].len() > max_decimal_places {
            panic!(
                "Unexpected serialization: expected {} decimal places, value was {})",
                max_decimal_places, value_str,
            );
        }
    }

    #[test]
    fn all_prices_should_serialize_to_two_decimal_places() {
        for price in get_all_prices() {
            let price_as_string = price.to_string();
            check_decimal_places(&price_as_string, 2);
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct PriceContainer {
        price: Price,
    }

    #[test]
    fn struct_with_price_should_serialize_to_two_decimal_places() {
        for price in get_all_prices() {
            let container = PriceContainer {
                price: Price::new(price).unwrap(),
            };
            let json = serde_json::to_string(&container)
                .unwrap_or_else(|e| panic!("Failed to serialize price {}: {}", price, e));

            // Extract the price string directly from JSON (format is {"price":123.45})
            // We need to verify what's actually in the serialized JSON.
            let price_str = json
                .strip_prefix("{\"price\":")
                .and_then(|s| s.strip_suffix("}"))
                .unwrap_or_else(|| panic!("Unexpected JSON format for price {}: {}", price, json));

            check_decimal_places(price_str, 2);
        }
    }
}
