use proc_macro2::TokenStream;
use quote::quote;

use super::injector::CodeInjector;
use super::GenV1GeneratorStrategy;
use crate::aping_ast::types::Comment;
use crate::aping_ast::Aping;

impl<T: CodeInjector> GenV1GeneratorStrategy<T> {
    pub(crate) fn generate_top_level_docs(&self, aping: &Aping) -> TokenStream {
        let description = aping.top_level_docs().module_comment();
        let namespace = aping.namespace().module_comment();
        let owner = aping.owner().module_comment();
        let date = aping.date().module_comment();
        let version = aping.version().module_comment();
        let name = aping.name().module_comment();

        quote! {
            #[doc = "# This document was generated by BetfairTypeGen GenV1"]
            #[doc = "## Name"]
            #name
            #[doc = "## Description"]
            #description
            #[doc = "## Version"]
            #version
            #[doc = "## Date"]
            #date
            #[doc = "## Owner"]
            #owner
            #[doc = "## Namespace"]
            #namespace
        }
    }
}

pub(crate) trait CommentParse {
    fn module_comment(&self) -> TokenStream;
    fn object_comment(&self) -> TokenStream;
}

impl CommentParse for &[Comment] {
    fn module_comment(&self) -> TokenStream {
        self.iter().map(|x| x.module_comment()).fold(quote! {}, |acc, i| {
            quote! {
                #acc
                #i
            }
        })
    }

    fn object_comment(&self) -> TokenStream {
        self.iter().map(|x| x.object_comment()).fold(quote! {}, |acc, i| {
            quote! {
                #acc
                #i
            }
        })
    }
}

#[cfg(test)]
mod test {

    use super::super::test::gen_v1;
    use super::*;
    use crate::aping_ast::types::{Comment, Name};
    use crate::gen_v1::injector::CodeInjectorV1;

    #[rstest::rstest]
    fn module_docs(gen_v1: GenV1GeneratorStrategy<CodeInjectorV1>) {
        let aping = Aping::builder()
            .top_level_docs(vec![
                Comment::new("my custom text".to_string()),
                Comment::new("my custom text x2".to_string()),
            ])
            .version(Name("1.0.0".to_string()))
            .date(Name("2020-01-01".to_string()))
            .name(Name("MyName".to_string()))
            .owner(Name("MyOwner".to_string()))
            .namespace(Name("MyNamespace".to_string()))
            .build();

        // Action
        let generated_code = gen_v1.generate_top_level_docs(&aping);

        // Assert
        let expected = quote! {
            #[doc = "# This document was generated by BetfairTypeGen GenV1"]
            #[doc = "## Name"]
            #[doc = "MyName"]
            #[doc = "## Description"]
            #[doc = "my custom text"]
            #[doc = "my custom text x2"]
            #[doc = "## Version"]
            #[doc = "1.0.0"]
            #[doc = "## Date"]
            #[doc = "2020-01-01"]
            #[doc = "## Owner"]
            #[doc = "MyOwner"]
            #[doc = "## Namespace"]
            #[doc = "MyNamespace"]
        };
        assert_eq!(generated_code.to_string(), expected.to_string());
    }
}
