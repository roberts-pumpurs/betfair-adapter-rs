//! The first version of `BetfairTypeGen` implementation

mod data_types;
mod documentation;
mod injector;
mod rpc_calls;
mod top_level_preamble;
mod type_resolver;

use betfair_xml_parser::Interface;
pub use injector::CodeInjector;
use proc_macro2::TokenStream;
use quote::quote;

use self::type_resolver::TypeResolverV1;
use crate::aping_ast::Aping;
use crate::settings::GeneratorSettings;
use crate::GeneratorStrategy;

/// The first version of `BetfairTypeGen` implementation
#[derive(Debug)]
pub struct GenV1GeneratorStrategy<T: CodeInjector> {
    pub(crate) type_resolver: TypeResolverV1,
    pub(crate) code_injector: T,
}

impl<T: CodeInjector> GenV1GeneratorStrategy<T> {
    /// # Instantiate a new `GenV1`
    /// This is the strategy that will be used to generate the code
    pub const fn new(code_injector: T) -> Self {
        Self {
            type_resolver: TypeResolverV1::new(),
            code_injector,
        }
    }
}

impl GenV1GeneratorStrategy<injector::CodeInjectorV1> {
    /// Creates a new `GenV1GeneratorStrategy` with the given `CodeInjectorV1`.
    #[must_use]
    pub fn preconfigured() -> Self {
        Self::new(injector::CodeInjectorV1::new())
    }
}

impl<T: CodeInjector> GeneratorStrategy for GenV1GeneratorStrategy<T> {
    fn generate_submodule(&self, interface: impl Into<Interface>) -> TokenStream {
        let interface = interface.into();
        let aping: Aping = interface.into();

        let aping = aping;
        let top_level_docs = self.generate_top_level_docs(&aping);
        let data_types = aping
            .data_types()
            .iter()
            .fold(quote! {}, |acc, (_name, data)| {
                let iter_data_type = self.generate_data_type(data);

                quote! {
                    #acc

                    #iter_data_type
                }
            });
        let rpc_calls = aping
            .rpc_calls()
            .iter()
            .fold(quote! {}, |acc, (_name, data)| {
                let iter_rpc_call = self.generate_rpc_call(data);

                quote! {
                    #acc

                    #iter_rpc_call
                }
            });

        let preamble = self.code_injector.module_level_preamble();

        quote!(
            #top_level_docs

            use super::*;
            #preamble

            #data_types

            #rpc_calls
        )
    }

    // TODO: Implement the inside gen_1 generate_mod, proper preamble, and proper submodules
    fn generate_mod(&self, settings: &impl GeneratorSettings) -> TokenStream {
        let transport_layer = self.generate_transport_layer();

        let mut top_level_preamble = quote! {
            #transport_layer
        };
        if settings.account_aping() {
            top_level_preamble = quote! {
                #top_level_preamble
                pub mod account_aping;
            };
        }
        if settings.heartbeat_aping() {
            top_level_preamble = quote! {
                #top_level_preamble
                pub mod heartbeat_aping;
            };
        }
        if settings.sports_aping() {
            top_level_preamble = quote! {
                #top_level_preamble
                pub mod sports_aping;
            };
        }
        if settings.stream_api() {
            top_level_preamble = quote! {
                #top_level_preamble
                pub mod stream_api;
            };
        }
        top_level_preamble
    }
}

#[cfg(test)]
mod test {

    use super::injector::CodeInjectorV1;
    use super::*;

    #[rstest::fixture]
    pub fn gen_v1() -> GenV1GeneratorStrategy<CodeInjectorV1> {
        GenV1GeneratorStrategy::preconfigured()
    }
}
