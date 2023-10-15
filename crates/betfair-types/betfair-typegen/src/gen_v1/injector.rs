use proc_macro2::TokenStream;
use quote::quote;
pub trait CodeInjector {
    fn module_level_preamble(&self) -> TokenStream;
    fn enum_derives(&self) -> TokenStream;
    fn enum_variant_derives(&self) -> TokenStream;
    fn struct_derives(&self) -> TokenStream;
    fn type_alias_derives(&self) -> TokenStream;
    fn struct_parameter_derives(&self) -> TokenStream;
}

pub struct CodeInjectorV1 {
    module_level_preamble: TokenStream,
    enum_derives: TokenStream,
    enum_variant_derives: TokenStream,
    struct_derives: TokenStream,
    type_alias_derives: TokenStream,
    struct_parameter_derives: TokenStream,
}

impl CodeInjector for CodeInjectorV1 {
    fn module_level_preamble(&self) -> TokenStream {
        self.module_level_preamble.clone()
    }
    fn enum_derives(&self) -> TokenStream {
        self.enum_derives.clone()
    }
    fn enum_variant_derives(&self) -> TokenStream {
        self.enum_variant_derives.clone()
    }
    fn struct_derives(&self) -> TokenStream {
        self.struct_derives.clone()
    }
    fn type_alias_derives(&self) -> TokenStream {
        self.type_alias_derives.clone()
    }
    fn struct_parameter_derives(&self) -> TokenStream {
        self.struct_parameter_derives.clone()
    }
}

impl CodeInjectorV1 {
    pub fn new() -> Self {
        CodeInjectorV1 {
            module_level_preamble: quote! {
                use std::fmt::Debug;
                use serde::{Serialize, Deserialize};
                use chrono::{DateTime, Utc};
                use typed_builder::TypedBuilder;
            },
            enum_derives: quote! {
                #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
            },
            enum_variant_derives: quote! {},
            struct_derives: quote! {
                #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, TypedBuilder)]
                #[serde(rename_all = "camelCase")]
            },
            type_alias_derives: quote! {
                #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
                #[serde(rename_all = "camelCase")]
            },
            struct_parameter_derives: quote! {},
        }
    }
}
impl Default for CodeInjectorV1 {
    fn default() -> Self {
        Self::new()
    }
}
