use serde::{Deserialize, Serialize};

/// Represents a handicap value using a decimal.
#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Eq, Hash, Ord)]
pub struct Handicap(pub rust_decimal::Decimal);
