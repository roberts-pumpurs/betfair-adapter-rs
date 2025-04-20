use serde::{Deserialize, Serialize};

/// Optional parameter allowing the client to pass a unique string (up to 32 chars) that is
/// used to de-dupe mistaken re-submissions. customerRef can contain: upper/lower chars,
/// digits, chars : - . _ + * : ; ~ only. Please note: There is a time window associated
/// with the de-duplication of duplicate submissions which is 60 seconds. NB:  This field
/// does not persist into the placeOrders response/Order Stream API and should not be confused
/// with customerOrderRef, which is separate field that can be sent in the `PlaceInstruction`.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct CustomerRef(String);

impl CustomerRef {
    pub fn new(customer_ref: String) -> Result<Self, CustomerRefParseError> {
        if customer_ref.len() > 32 {
            return Err(CustomerRefParseError::TooLong);
        }
        const VALID_CHARS: &[char] = &['-', '.', '_', '+', '*', ':', ';', '~'];

        if !customer_ref
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || VALID_CHARS.contains(&c))
        {
            return Err(CustomerRefParseError::InvalidCharacters);
        }
        Ok(Self(customer_ref))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CustomerRefParseError {
    #[error("Customer reference too long")]
    TooLong,
    #[error("Customer reference contains invalid characters")]
    InvalidCharacters,
}
