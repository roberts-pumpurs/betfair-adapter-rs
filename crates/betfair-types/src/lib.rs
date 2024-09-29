//! # Betfair Types library

pub use rust_decimal;
pub mod bot_login;
pub mod customer_order_ref;
pub mod customer_ref;
pub mod customer_strategy_ref;
pub mod handicap;
pub mod price;
pub mod size;

pub mod keep_alive {
    pub use crate::shared::*;
}

pub mod logout {
    pub use crate::shared::*;
}
mod shared;

#[expect(clippy::all)]
pub mod types {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}
