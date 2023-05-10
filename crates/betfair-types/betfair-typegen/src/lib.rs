//! This crate is used to generate the types for the betfair API.
//! Input: Betfair API-NG WSDL file
//! Output: Rust types for the Betfair API

#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]
pub mod gen_v1;

use betfair_xml_parser::Interface;
pub use proc_macro2::TokenStream;
pub use rust_decimal;

/// The trait that is used to generate the types for the Betfair API
pub trait BetfairTypeGen {
    /// Generate the types for the Betfair API
    /// # Arguments
    /// * `interface` - The Betfair API interface
    /// # Returns
    /// The generated types for the Betfair API that can be written to a file
    fn generate(&self, interface: impl Into<Interface>) -> TokenStream;
}
