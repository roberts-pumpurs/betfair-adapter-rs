use proc_macro2::TokenStream;
use quote::quote;

use super::injector::CodeInjector;
use super::GenV1GeneratorStrategy;

impl<T: CodeInjector> GenV1GeneratorStrategy<T> {
    pub(crate) fn generate_transport_layer(&self) -> TokenStream {
        quote! {
            use std::future::Future;
            use serde::{Serialize, de::DeserializeOwned};

            pub trait BetfairRpcRequest {
                type Res;
                type Error;

                fn method() -> &'static str;
            }
        }
    }
}
