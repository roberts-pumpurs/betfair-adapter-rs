//! The first version of BetfairTypeGen implementation

mod data_types;
mod documentation;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::ast::{Aping, Comment, Name};
use crate::GeneratorStrategy;

/// The first version of BetfairTypeGen implementation
pub struct GenV1;

impl GeneratorStrategy for GenV1 {
    fn generate(&self, aping: impl Into<Aping>) -> TokenStream {
        let aping = aping.into();
        let top_level_docs = self.generate_top_level_docs(&aping);
        let data_types = aping.data_types().iter().fold(quote! {}, |acc, (_name, data)| {
            let iter_data_type = self.generate_data_type(data);

            quote! {
                #acc

                #iter_data_type
            }
        });

        quote!(
            #top_level_docs

            #data_types
        )
    }
}

impl Comment {
    pub fn module_comment(&self) -> TokenStream {
        const MODULE_COMMENT: &str = "///";
        pad_with(MODULE_COMMENT, &self.item)
    }

    pub fn object_comment(&self) -> TokenStream {
        const OBJECT_COMMENT: &str = "///";
        pad_with(OBJECT_COMMENT, &self.item)
    }
}

impl Name {
    pub fn ident(&self) -> Ident {
        Ident::new(&self.0, Span::call_site())
    }

    pub fn module_comment(&self) -> TokenStream {
        const MODULE_COMMENT: &str = "///";
        pad_with(MODULE_COMMENT, &self.0)
    }
}

fn pad_with(pad: &str, text: impl AsRef<str>) -> proc_macro2::TokenStream {
    let text = pad.to_string() + text.as_ref();
    syn::parse_str::<proc_macro2::TokenStream>(&text).unwrap()
}

#[cfg(test)]
mod test {

    use betfair_xml_parser::Interface;

    use super::*;
    use crate::GeneratorStrategy;

    pub(crate) const GEN_V1: GenV1 = GenV1;

    #[rstest::fixture]
    pub fn aping() -> Aping {
        let interface: Interface = include_str!("../../assets/HeartbeatAPING.xml").into();
        interface.into()
    }

    #[rstest::rstest]
    fn test_gen_v1(aping: Aping) {
        let _generated_code = GEN_V1.generate(aping);

        // TODO: assert the generated code
    }
}
