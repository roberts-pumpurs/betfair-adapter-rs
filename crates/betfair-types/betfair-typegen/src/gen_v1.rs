//! The first version of BetfairTypeGen implementation
mod common;
mod data_type;
mod exception_type;
mod operation;
mod simple_type;

use proc_macro2::TokenStream;
use quote::quote;

use crate::BetfairTypeGen;

/// The first version of BetfairTypeGen implementation
pub struct GenV1;

impl BetfairTypeGen for GenV1 {
    fn generate(&self, interface: impl Into<betfair_xml_parser::Interface>) -> TokenStream {
        let interface: betfair_xml_parser::Interface = interface.into();

        // Generate the top level documentation
        let top_level_docs = interface.items.iter().find_map(|x| match x {
            betfair_xml_parser::InterfaceItems::Description(x) => Some(x),
            _ => None,
        });
        let top_level_docs = top_level_docs.unwrap();
        let _ = common::description::docs(top_level_docs);

        // Generate the top level simple types
        let _simple_types = interface.items.iter().filter_map(|x| match x {
            betfair_xml_parser::InterfaceItems::SimpleType(x) => Some(x),
            _ => None,
        });

        // Generate the top level data types
        let _data_types = interface.items.iter().filter_map(|x| match x {
            betfair_xml_parser::InterfaceItems::DataType(x) => Some(x),
            _ => None,
        });

        // Generate the top level exception types
        let _exception_types = interface.items.iter().filter_map(|x| match x {
            betfair_xml_parser::InterfaceItems::ExceptionType(x) => Some(x),
            _ => None,
        });

        // Generate the top level traits (every operation is a trait)

        quote!()
    }
}

// /// generate methods from attributes on top of struct or enum
// pub fn top_level_methods(&self) -> TokenStream {
//     let author = &self.author;
//     let about = &self.about;
//     let methods = &self.methods;
//     let doc_comment = &self.doc_comment;

//     quote!( #(#doc_comment)* #author #about #(#methods)*  )
// }

// /// generate methods on top of a field
// pub fn field_methods(&self) -> TokenStream {
//     let methods = &self.methods;
//     let doc_comment = &self.doc_comment;
//     quote!( #(#doc_comment)* #(#methods)* )
// }

// pub fn version(&self) -> TokenStream {
//     match (&self.no_version, &self.version) {
//         (None, Some(m)) => m.to_token_stream(),

//         (None, None) => std::env::var("CARGO_PKG_VERSION")
//             .map(|version| quote!( .version(#version) ))
//             .unwrap_or_default(),

//         _ => quote!(),
//     }
// }

#[cfg(test)]
mod test {

    use super::*;
    use crate::BetfairTypeGen;

    #[test]
    fn test_gen_v1() {
        let interface = include_str!("../../assets/HeartbeatAPING.xml");
        let gen = GenV1;
        let _ = gen.generate(interface);

        // TODO: assert the generated code
    }
}
