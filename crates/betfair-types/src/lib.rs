//! # Betfair Types library

#![allow(clippy::all)]

pub use rust_decimal;

pub mod types {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}
