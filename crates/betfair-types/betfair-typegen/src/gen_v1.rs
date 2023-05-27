//! The first version of BetfairTypeGen implementation

mod data_types;
mod documentation;
mod rpc_calls;
mod type_resolver;

use betfair_xml_parser::Interface;
use proc_macro2::TokenStream;
use quote::quote;

use self::type_resolver::TypeResolverV1;
use crate::ast::Aping;
use crate::GeneratorStrategy;

/// The first version of BetfairTypeGen implementation
pub struct GenV1 {
    /// The type resolver
    pub(crate) type_resolver: TypeResolverV1,
}

impl GenV1 {
    /// # Instantiate a new `GenV1`
    /// This is the strategy that will be used to generate the code
    pub const fn new() -> Self {
        Self { type_resolver: TypeResolverV1::new() }
    }
}

impl GeneratorStrategy for GenV1 {
    fn generate(&self, interface: impl Into<Interface>) -> TokenStream {
        let interface = interface.into();
        let aping: Aping =
            interface.try_into().expect("Failed to convert the interface into the AST");

        let aping = aping;
        let top_level_docs = self.generate_top_level_docs(&aping);
        let data_types = aping.data_types().iter().fold(quote! {}, |acc, (_name, data)| {
            let iter_data_type = self.generate_data_type(data);

            quote! {
                #acc

                #iter_data_type
            }
        });
        let rpc_calls = aping.rpc_calls().iter().fold(quote! {}, |acc, (_name, data)| {
            let iter_rpc_call = self.generate_rpc_call(data);

            quote! {
                #acc

                #iter_rpc_call
            }
        });
        let transport_layer = self.generate_transport_layer();

        quote!(
            #top_level_docs

            #transport_layer

            #data_types

            #rpc_calls
        )
    }
}

#[cfg(test)]
mod test {

    use betfair_xml_parser::Interface;

    use super::*;
    use crate::GeneratorStrategy;

    pub(crate) const GEN_V1: GenV1 = GenV1::new();

    #[rstest::fixture]
    pub fn interface() -> Interface {
        let interface: Interface = include_str!("../../assets/HeartbeatAPING.xml").into();
        interface
    }

    #[rstest::rstest]
    fn test_gen_v1(interface: Interface) {
        let _generated_code = GEN_V1.generate(interface);

        // TODO: assert the generated code
    }
}
