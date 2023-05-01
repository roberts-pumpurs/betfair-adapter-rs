pub mod gen_v1;

pub use rust_decimal;
use betfair_xml_parser::Interface;


pub trait BetfairTypeGen {
    fn generate(&self, interface: impl Into<Interface>) -> String;
}
