#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]
pub mod gen_v1;

pub use rust_decimal;
use betfair_xml_parser::Interface;


pub trait BetfairTypeGen {
    fn generate(&self, interface: impl Into<Interface>) -> String;
}
