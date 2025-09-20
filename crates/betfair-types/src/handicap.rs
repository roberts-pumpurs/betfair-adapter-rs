use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Serialize, Deserialize, Eq, Hash, Ord)]
pub struct Handicap(pub rust_decimal::Decimal);
