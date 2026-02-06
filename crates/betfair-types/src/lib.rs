//! # Betfair Types library

pub mod bot_login;
pub mod customer_order_ref;
pub mod customer_ref;
pub mod customer_strategy_ref;
pub mod handicap;
pub mod numeric;
pub mod price;
pub mod size;

#[cfg(test)]
pub mod tests;

// Re-export numeric types for convenience
pub use numeric::F64Ord;

pub mod keep_alive {
    pub use crate::shared::*;
}

pub mod logout {
    pub use crate::shared::*;
}
mod shared;

/// The generated types
#[expect(clippy::all)]
pub mod types {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}

impl types::sports_aping::MarketId {
    /// Construct a new market id
    pub fn new(id: impl Into<String>) -> Self {
        Self(std::sync::Arc::new(id.into()))
    }
}

impl types::sports_aping::BetId {
    /// construct a new bet id
    pub fn new(id: impl Into<String>) -> Self {
        Self(std::sync::Arc::new(id.into()))
    }
}
