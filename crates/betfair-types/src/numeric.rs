//! Numeric primitive abstraction
//!
//! This module provides a unified interface for numeric operations using `f64`.

/// Wrapper around f64 that implements Eq, Ord, and Hash using total_cmp
/// This allows f64 to be used in contexts that require these traits
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct F64Ord(pub f64);

impl F64Ord {
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    pub const fn zero() -> Self {
        Self(0.0)
    }

    pub const fn as_f64(&self) -> f64 {
        self.0
    }
}

impl From<f64> for F64Ord {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<F64Ord> for f64 {
    fn from(value: F64Ord) -> Self {
        value.0
    }
}

impl PartialEq for F64Ord {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for F64Ord {}

impl PartialOrd for F64Ord {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for F64Ord {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl core::hash::Hash for F64Ord {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl core::fmt::Display for F64Ord {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cmp::Ordering;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn f64ord_should_use_bitwise_equality() {
        let nan = F64Ord::new(f64::NAN);
        assert_eq!(nan, F64Ord::new(f64::NAN)); // This would fail for a normal f64.

        assert_eq!(F64Ord::new(-0.0).cmp(&F64Ord::new(0.0)), Ordering::Less);
        assert_ne!(F64Ord::new(-0.0), F64Ord::new(0.0));

        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        nan.hash(&mut h1);
        nan.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }
}

// Re-export for convenience
pub use core::ops::{Add, Div, Mul, Sub};

/// Trait for creating numeric literals in a type-agnostic way
pub trait NumericLiteral {
    fn literal_from_f64(value: f64) -> Self;
    fn literal_from_str(value: &str) -> Result<Self, String>
    where
        Self: Sized;
}

impl NumericLiteral for f64 {
    fn literal_from_f64(value: f64) -> Self {
        value
    }

    fn literal_from_str(value: &str) -> Result<Self, String> {
        value
            .parse()
            .map_err(|e| format!("Failed to parse f64: {}", e))
    }
}

/// Helper trait for numeric operations that work across both Decimal and f64
pub trait NumericOps: Copy + Clone + PartialOrd + PartialEq {
    fn checked_add(&self, other: Self) -> Option<Self>;
    fn checked_sub(&self, other: Self) -> Option<Self>;
    fn checked_mul(&self, other: Self) -> Option<Self>;
    fn checked_div(&self, other: Self) -> Option<Self>;
    fn checked_rem(&self, other: Self) -> Option<Self>;
    fn saturating_add(&self, other: Self) -> Self;
    fn saturating_sub(&self, other: Self) -> Self;
    fn saturating_mul(&self, other: Self) -> Self;
    fn round_2dp(self) -> Self;
    fn zero() -> Self;
    fn is_sign_positive(&self) -> bool;
    fn is_sign_negative(&self) -> bool;
}

impl NumericOps for f64 {
    fn checked_add(&self, other: Self) -> Option<Self> {
        let result = self + other;
        if result.is_finite() {
            Some(result)
        } else {
            None
        }
    }

    fn checked_sub(&self, other: Self) -> Option<Self> {
        let result = self - other;
        if result.is_finite() {
            Some(result)
        } else {
            None
        }
    }

    fn checked_mul(&self, other: Self) -> Option<Self> {
        let result = self * other;
        if result.is_finite() {
            Some(result)
        } else {
            None
        }
    }

    fn checked_div(&self, other: Self) -> Option<Self> {
        if other == 0.0 {
            None
        } else {
            let result = self / other;
            if result.is_finite() {
                Some(result)
            } else {
                None
            }
        }
    }

    fn checked_rem(&self, other: Self) -> Option<Self> {
        if other == 0.0 {
            None
        } else {
            let result = self % other;
            if result.is_finite() {
                Some(result)
            } else {
                None
            }
        }
    }

    fn saturating_add(&self, other: Self) -> Self {
        let result = self + other;
        if result.is_finite() {
            result
        } else if result.is_infinite() && result.is_sign_positive() {
            f64::MAX
        } else {
            f64::MIN
        }
    }

    fn saturating_sub(&self, other: Self) -> Self {
        let result = self - other;
        if result.is_finite() {
            result
        } else if result.is_infinite() && result.is_sign_positive() {
            f64::MAX
        } else {
            f64::MIN
        }
    }

    fn saturating_mul(&self, other: Self) -> Self {
        let result = self * other;
        if result.is_finite() {
            result
        } else if result.is_infinite() && result.is_sign_positive() {
            f64::MAX
        } else {
            f64::MIN
        }
    }

    #[inline(always)]
    fn round_2dp(self) -> Self {
        (self * 100.0).round() / 100.0
    }

    fn zero() -> Self {
        0.0
    }

    fn is_sign_positive(&self) -> bool {
        f64::is_sign_positive(*self)
    }

    fn is_sign_negative(&self) -> bool {
        f64::is_sign_negative(*self)
    }
}

/// Create a numeric constant (f64)
#[macro_export]
macro_rules! num {
    ($lit:literal) => {{ $lit as f64 }};
}

/// Create a numeric constant (F64Ord)
#[macro_export]
macro_rules! num_ord {
    ($lit:literal) => {{ $crate::numeric::F64Ord::from($lit as f64) }};
}

/// Create a numeric constant (u8)
#[macro_export]
macro_rules! num_u8 {
    ($lit:literal) => {{ $lit as u8 }};
}
