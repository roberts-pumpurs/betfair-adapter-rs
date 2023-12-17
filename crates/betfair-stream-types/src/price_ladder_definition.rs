use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceLadderDefinition {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _type: Option<Type>,
}

impl PriceLadderDefinition {
    #[allow(dead_code)]
    pub fn new() -> PriceLadderDefinition {
        PriceLadderDefinition { _type: None }
    }
}

///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Type {
    Classic,
    Finest,
    LineRange,
}

impl Default for Type {
    fn default() -> Type {
        Self::Classic
    }
}
