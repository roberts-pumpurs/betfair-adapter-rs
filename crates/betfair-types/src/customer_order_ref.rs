use serde::{Deserialize, Serialize};

// An optional reference the customer can set to identify instructions and it will be returned on
// order change messages via the stream API. No validation will be done on uniqueness and the string
// is limited to 32 characters. If an empty string is provided it will be treated as null.

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
/// A struct representing a customer order reference.
pub struct CustomerOrderRef([char; 32]);

impl CustomerOrderRef {
    /// Creates a new `CustomerOrderRef` from a 32-character array of characters.
    ///
    /// # Arguments
    ///
    /// * `customer_order_ref` - A 32-character array of characters.
    pub fn new(customer_order_ref: [char; 32]) -> Result<Self, CustomerOrderRefParseError> {
        const VALID_CHARS: &[char] = &['-', '.', '_', '+', '*', ':', ';', '~'];

        if !customer_order_ref
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || VALID_CHARS.contains(c))
        {
            return Err(CustomerOrderRefParseError::InvalidCharacters)
        }
        Ok(Self(customer_order_ref))
    }
}

/// An error that occurs when parsing a customer order reference.
#[derive(Debug, thiserror::Error)]
pub enum CustomerOrderRefParseError {
    /// Customer order reference too long
    #[error("Customer order reference too long")]
    TooLong,
    /// Customer order reference contains invalid characters
    #[error("Customer order reference contains invalid characters")]
    InvalidCharacters,
}
