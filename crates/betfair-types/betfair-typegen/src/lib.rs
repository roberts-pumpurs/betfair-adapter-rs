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

use ast::Aping;
use betfair_xml_parser::Interface;
pub use proc_macro2::TokenStream;
mod ast;
pub use rust_decimal;

/// # The Betfair type generator
/// This is the main entry point for the crate
pub struct BetfairTypeGenerator;

impl BetfairTypeGenerator {
    /// # Generate the types for the Betfair API
    /// Provide the strategy to use to generate the types and the Betfair API interface
    pub fn generate(
        &self,
        strategy: impl GeneratorStrategy,
        interface: impl Into<Interface>,
    ) -> TokenStream {
        let interface: Interface = interface.into();
        let aping_ast: Aping =
            interface.try_into().expect("Failed to convert the interface into the AST");

        strategy.generate(aping_ast)
    }
}

/// The trait that is used to generate the types for the Betfair API
pub trait GeneratorStrategy {
    /// Generate the types for the Betfair API
    /// # Arguments
    /// * `aping` - The Betfair API interface
    /// # Returns
    /// The generated types for the Betfair API that can be written to a file
    fn generate(&self, aping: impl Into<Aping>) -> TokenStream;
}
