use serde::{Deserialize, Serialize};

use crate::numeric::F64Ord;

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Serialize, Deserialize, Eq, Hash, Ord)]
pub struct Handicap(pub F64Ord);
