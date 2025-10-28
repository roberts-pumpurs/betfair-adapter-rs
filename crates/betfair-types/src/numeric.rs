//! Numeric primitive abstraction
//!
//! This module provides a unified interface for numeric operations that can use either
//! `rust_decimal::Decimal` (default, precise) or `f64` (fast, when `fast-floats` feature is enabled).

#[cfg(not(feature = "fast-floats"))]
pub use rust_decimal::Decimal as NumericPrimitive;

#[cfg(feature = "fast-floats")]
pub type NumericPrimitive = f64;

/// Type alias for general decimal values (not Price/Size specific)
/// This is used for fields like handicap, market rates, etc.
///
/// When using fast-floats, this is a wrapper around f64 that implements Eq/Ord/Hash
/// using total_cmp, which allows it to be used in structs that derive Eq.
#[cfg(not(feature = "fast-floats"))]
pub use rust_decimal::Decimal as NumericOrdPrimitive;

#[cfg(feature = "fast-floats")]
pub type NumericOrdPrimitive = F64Ord;

/// Wrapper around f64 that implements Eq, Ord, and Hash using total_cmp
/// This allows f64 to be used in contexts that require these traits
#[cfg(feature = "fast-floats")]
#[derive(
    Debug, Clone, Copy, PartialEq, PartialOrd, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct F64Ord(pub f64);

#[cfg(feature = "fast-floats")]
impl F64Ord {
    pub const fn new(value: f64) -> Self {
        Self(value)
    }
}

#[cfg(feature = "fast-floats")]
impl From<f64> for F64Ord {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

#[cfg(feature = "fast-floats")]
impl From<F64Ord> for f64 {
    fn from(value: F64Ord) -> Self {
        value.0
    }
}

#[cfg(feature = "fast-floats")]
impl Eq for F64Ord {}

#[cfg(feature = "fast-floats")]
impl Ord for F64Ord {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

#[cfg(feature = "fast-floats")]
impl core::hash::Hash for F64Ord {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

#[cfg(feature = "fast-floats")]
impl core::ops::Deref for F64Ord {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "fast-floats")]
impl core::fmt::Display for F64Ord {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
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

#[cfg(not(feature = "fast-floats"))]
impl NumericLiteral for rust_decimal::Decimal {
    fn literal_from_f64(value: f64) -> Self {
        <rust_decimal::Decimal as rust_decimal::prelude::FromPrimitive>::from_f64(value)
            .expect("Should be a valid f64 value")
    }

    fn literal_from_str(value: &str) -> Result<Self, String> {
        use core::str::FromStr;
        Self::from_str(value).map_err(|e| format!("Failed to parse Decimal: {}", e))
    }
}

#[cfg(feature = "fast-floats")]
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

#[cfg(not(feature = "fast-floats"))]
impl NumericOps for rust_decimal::Decimal {
    fn checked_add(&self, other: Self) -> Option<Self> {
        rust_decimal::Decimal::checked_add(*self, other)
    }

    fn checked_sub(&self, other: Self) -> Option<Self> {
        rust_decimal::Decimal::checked_sub(*self, other)
    }

    fn checked_mul(&self, other: Self) -> Option<Self> {
        rust_decimal::Decimal::checked_mul(*self, other)
    }

    fn checked_div(&self, other: Self) -> Option<Self> {
        rust_decimal::Decimal::checked_div(*self, other)
    }

    fn checked_rem(&self, other: Self) -> Option<Self> {
        rust_decimal::Decimal::checked_rem(*self, other)
    }

    fn saturating_add(&self, other: Self) -> Self {
        rust_decimal::Decimal::saturating_add(*self, other)
    }

    fn saturating_sub(&self, other: Self) -> Self {
        rust_decimal::Decimal::saturating_sub(*self, other)
    }

    fn saturating_mul(&self, other: Self) -> Self {
        rust_decimal::Decimal::saturating_mul(*self, other)
    }

    fn round_2dp(self) -> Self {
        rust_decimal::Decimal::round_dp(&self, 2)
    }

    fn zero() -> Self {
        rust_decimal::Decimal::ZERO
    }

    fn is_sign_positive(&self) -> bool {
        rust_decimal::Decimal::is_sign_positive(self)
    }

    fn is_sign_negative(&self) -> bool {
        rust_decimal::Decimal::is_sign_negative(self)
    }
}

#[cfg(feature = "fast-floats")]
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
        (self * 100.0).round() * 0.01
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

/// Create a numeric constant from a string literal at compile time
/// This macro helps create constants that work with both Decimal and f64
#[cfg(not(feature = "fast-floats"))]
#[macro_export]
macro_rules! num {
    ($lit:literal) => {{ ::rust_decimal_macros::dec!($lit) }};
}
#[cfg(feature = "fast-floats")]
#[macro_export]
macro_rules! num {
    ($lit:literal) => {{ $lit as f64 }};
}

/// Create a numeric constant which implements Ord from a string literal at compile time
#[cfg(not(feature = "fast-floats"))]
#[macro_export]
macro_rules! num_ord {
    ($lit:literal) => {{ ::rust_decimal_macros::dec!($lit) }};
}

/// Create a numeric constant which implements Ord from a string literal at compile time
#[cfg(feature = "fast-floats")]
#[macro_export]
macro_rules! num_ord {
    ($lit:literal) => {{ $crate::numeric::F64Ord::from($lit as f64) }};
}
