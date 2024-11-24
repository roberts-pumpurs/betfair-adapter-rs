use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// An optional reference customers can use to specify which strategy has sent the order.
///
/// The reference will be returned on order change messages through the stream API. The string
/// is limited to 15 characters. If an empty string is provided it will be treated as null.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CustomerStrategyRef([char; 15]);

impl CustomerStrategyRef {
    #[must_use]
    pub const fn new(customer_strategy_ref: [char; 15]) -> Self {
        Self(customer_strategy_ref)
    }
}

impl Serialize for CustomerStrategyRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert the `[char; 15]` array into a `String`, trimming any trailing null characters.
        let s: String = self.0.iter().collect();
        let s_trimmed = s.trim_end_matches('\0');
        serializer.serialize_str(s_trimmed)
    }
}

impl<'de> Deserialize<'de> for CustomerStrategyRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CustomerStrategyRefVisitor;

        impl<'de> Visitor<'de> for CustomerStrategyRefVisitor {
            type Value = CustomerStrategyRef;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string up to 15 characters")
            }

            fn visit_str<E>(self, v: &str) -> Result<CustomerStrategyRef, E>
            where
                E: de::Error,
            {
                let chars: Vec<char> = v.chars().collect();
                if chars.len() > 15 {
                    return Err(E::custom(format!(
                        "expected at most 15 characters, got {}",
                        chars.len()
                    )));
                }
                let mut array = ['\0'; 15];
                for (i, c) in chars.into_iter().enumerate() {
                    array[i] = c;
                }
                Ok(CustomerStrategyRef(array))
            }
        }

        deserializer.deserialize_str(CustomerStrategyRefVisitor)
    }
}
