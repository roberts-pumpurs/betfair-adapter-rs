use serde::{Deserialize, Serialize};

// An optional reference the customer can set to identify instructions and it will be returned on
// order change messages via the stream API. No validation will be done on uniqueness and the string
// is limited to 32 characters. If an empty string is provided it will be treated as null.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct CustomerOrderRef(String);

impl CustomerOrderRef {
    pub fn new(customer_order_ref: String) -> Result<Self, CustomerOrderRefParseError> {
        if customer_order_ref.len() > 32 {
            return Err(CustomerOrderRefParseError::TooLong)
        }
        const VALID_CHARS: &[char] = &['-', '.', '_', '+', '*', ':', ';', '~'];

        if !customer_order_ref
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || VALID_CHARS.contains(&c))
        {
            return Err(CustomerOrderRefParseError::InvalidCharacters)
        }
        Ok(CustomerOrderRef(customer_order_ref))
    }

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
