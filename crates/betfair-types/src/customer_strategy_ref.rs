use std::fmt::{self, Display};

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// An optional reference customers can use to specify which strategy has sent the order.
///
/// The reference will be returned on order change messages through the stream API. The string
/// is limited to 15 characters. If an empty string is provided it will be treated as null.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CustomerStrategyRef([char; 15]);

impl CustomerStrategyRef {
    pub const EMPTY: CustomerStrategyRef = CustomerStrategyRef::new(['\0'; 15]);

    #[must_use]
    pub const fn new(customer_strategy_ref: [char; 15]) -> Self {
        Self(customer_strategy_ref)
    }
}

impl Display for CustomerStrategyRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = self.0.iter().collect();
        let s_trimmed = s.trim_end_matches('\0');
        write!(f, "{}", s_trimmed)
    }
}

impl Default for CustomerStrategyRef {
    fn default() -> Self {
        Self::EMPTY
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

        impl Visitor<'_> for CustomerStrategyRefVisitor {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_full_string() {
        let chars = [
            'H', 'e', 'l', 'l', 'o', 'W', 'o', 'r', 'l', 'd', '1', '2', '3', '4', '5',
        ];
        let csr = CustomerStrategyRef::new(chars);
        assert_eq!(csr.to_string(), "HelloWorld12345");
    }

    #[test]
    fn test_display_short_string() {
        let mut chars = ['\0'; 15];
        chars[0] = 'H';
        chars[1] = 'e';
        chars[2] = 'l';
        chars[3] = 'l';
        chars[4] = 'o';
        let csr = CustomerStrategyRef::new(chars);
        assert_eq!(csr.to_string(), "Hello");
    }

    #[test]
    fn test_display_empty_string() {
        let chars = ['\0'; 15];
        let csr = CustomerStrategyRef::new(chars);
        assert_eq!(csr.to_string(), "");
    }

    #[test]
    fn test_display_trims_trailing_nulls() {
        let mut chars = ['\0'; 15];
        chars[0] = 'A';
        chars[1] = 'B';
        chars[2] = 'C';
        let csr = CustomerStrategyRef::new(chars);
        let displayed = csr.to_string();
        assert_eq!(displayed, "ABC");
        assert_eq!(displayed.len(), 3);
    }
}
