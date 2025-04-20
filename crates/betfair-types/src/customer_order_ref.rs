use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// An optional reference the customer can set to identify instructions and it will be returned on
/// order change messages via the stream API. No validation will be done on uniqueness and the
/// string is limited to 32 characters. If an empty string is provided it will be treated as null.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CustomerOrderRef(String);

impl CustomerOrderRef {
    /// Creates a new `CustomerOrderRef` after validating the input string.
    pub fn new(s: &str) -> Result<Self, CustomerOrderRefParseError> {
        const VALID_CHARS: &[char] = &['-', '.', '_', '+', '*', ':', ';', '~'];

        if s.len() > 32 {
            return Err(CustomerOrderRefParseError::TooLong);
        }

        if !s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || VALID_CHARS.contains(&c))
        {
            return Err(CustomerOrderRefParseError::InvalidCharacters);
        }

        Ok(Self(s.to_owned()))
    }

    /// Returns a reference to the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CustomerOrderRefParseError {
    #[error("Customer order reference too long")]
    TooLong,
    #[error("Customer order reference contains invalid characters")]
    InvalidCharacters,
}

impl Serialize for CustomerOrderRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the inner string directly.
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for CustomerOrderRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CustomerOrderRefVisitor;

        impl Visitor<'_> for CustomerOrderRefVisitor {
            type Value = CustomerOrderRef;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string up to 32 characters containing valid characters")
            }

            fn visit_str<E>(self, v: &str) -> Result<CustomerOrderRef, E>
            where
                E: de::Error,
            {
                // Use the `new` method to construct the instance, mapping errors appropriately.
                CustomerOrderRef::new(v).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(CustomerOrderRefVisitor)
    }
}
