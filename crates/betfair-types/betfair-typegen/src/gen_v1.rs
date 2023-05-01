mod exception_type;
mod data_type;
mod operation;
mod simple_type;
mod common;

use crate::BetfairTypeGen;

pub struct GenV1;

impl BetfairTypeGen for GenV1 {
    fn generate(&self, interface: impl Into<betfair_xml_parser::Interface>) -> String {
        let interface = interface.into();
        // Generate the top level documentation
        // Generate the top level simple types
        // Generate the top level data types
        // Generate the top level exception types
        // Generate the top level traits (every operation is a trait)

        todo!()
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
