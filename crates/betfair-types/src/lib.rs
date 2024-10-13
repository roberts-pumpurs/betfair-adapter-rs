//! # Betfair Types library

pub use rust_decimal;

/// Module for bot login functionalities.
pub mod bot_login;

/// Module for handling customer order references.
pub mod customer_order_ref;

/// Module for customer references.
pub mod customer_ref;

/// Module for customer strategy references.
pub mod customer_strategy_ref;

/// Module for handicap-related types and functions.
pub mod handicap;

/// Module for price-related types and functions.
pub mod price;

/// Module for size-related types and functions.
pub mod size;

/// Module for keep-alive functionalities.
pub mod keep_alive {
    pub use crate::shared::*;
}

/// Module for logout functionalities.
pub mod logout {
    pub use crate::shared::*;
}
mod shared;

/// Module for various types used in the library.
#[expect(clippy::all)]
pub mod types {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}
