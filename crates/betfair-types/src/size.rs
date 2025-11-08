use serde::{Deserialize, Serialize};

use crate::numeric::{NumericOps, NumericPrimitive};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Size(NumericPrimitive);

impl PartialEq for Size {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for Size {}

impl PartialOrd for Size {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Size {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl core::hash::Hash for Size {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Size {
    #[must_use]
    pub fn new(size: NumericPrimitive) -> Self {
        let size = size.round_2dp();
        Self(size)
    }

    /// This function is unsafe because it does not round the size to 2dp.
    /// # Safety
    /// The caller must ensure that the size is valid on Betfair.
    #[must_use]
    pub const unsafe fn new_unchecked(size: NumericPrimitive) -> Self {
        Self(size)
    }

    pub const fn as_f64(&self) -> f64 {
        self.0
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
        Self::new(val)
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
    use std::cmp::Ordering;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn size_should_use_bitwise_equality() {
        let nan = Size(f64::NAN);
        assert_eq!(nan, Size(f64::NAN)); // This would fail for a normal f64.

        assert_eq!(Size(-0.0).cmp(&Size(0.0)), Ordering::Less);
        assert_ne!(Size(-0.0), Size(0.0));

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        nan.hash(&mut h1);
        nan.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[rstest]
    #[case(num!(1.022192999293999))]
    fn size_gets_rounded_to_two_decimal_places(#[case] size_raw: NumericPrimitive) {
        let size = Size::from(size_raw);

        let expected = num!(1.02);
        let diff = (size.0 - expected).abs();
        assert!(
            diff < 1e-9,
            "Expected size to be rounded to 1.02, but got {}",
            size.0
        );
    }
}

#[cfg(test)]
mod size_serialization_tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    fn get_many_sizes() -> Vec<f64> {
        // These sizes are not exactly representable in an f64.
        vec![1.1, 1.2, 1.3, 1.5, 1.7, 2.1, 2.3, 3.14, 10.01, 9999.99]
    }

    #[test]
    fn get_many_sizes_generates_valid_sizes() {
        for size in get_many_sizes() {
            let valid_size = Size::new(size);
            assert_eq!(valid_size.0, size);
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
    fn all_sizes_should_serialize_to_two_decimal_places() {
        for size in get_many_sizes() {
            let size_as_string = size.to_string();
            check_decimal_places(&size_as_string, 2);
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct SizeContainer {
        size: Size,
    }

    #[test]
    fn struct_with_size_should_serialize_to_two_decimal_places() {
        for size in get_many_sizes() {
            let container = SizeContainer {
                size: Size::new(size),
            };

            let json = serde_json::to_string(&container)
                .unwrap_or_else(|e| panic!("Failed to serialize size {}: {}", size, e));

            // Extract the size string directly from JSON (format is {"size":123.45})
            // We need to verify what's actually in the serialized JSON.
            let size_str = json
                .strip_prefix("{\"size\":")
                .and_then(|s| s.strip_suffix("}"))
                .unwrap_or_else(|| panic!("Unexpected JSON format for size {}: {}", size, json));

            check_decimal_places(size_str, 2);
        }
    }
}
