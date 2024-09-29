//! This crate is used to generate the types for the betfair API.
//! Input: Betfair API-NG WSDL file
//! Output: Rust types for the Betfair API

#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]
#![allow(incomplete_features)]

mod aping_ast;
pub mod gen_v1;
mod generator;

use betfair_xml_parser::Interface;
use generator::settings::GeneratorSettings;
pub use generator::{output, settings, BetfairTypeGenerator};
pub use proc_macro2::TokenStream;
pub use rust_decimal;

/// The trait that is used to generate the types for the Betfair API
pub trait GeneratorStrategy {
    /// Generate the types for the Betfair API
    /// # Arguments
    /// * `aping` - The Betfair API interface
    /// # Returns
    /// The generated types for the Betfair API that can be written to a file
    fn generate_submodule<T: Into<Interface>>(&self, aping: T) -> TokenStream;

    /// Generate the top level documentation and types
    fn generate_mod<T: GeneratorSettings>(&self, settings: T) -> TokenStream;
}
