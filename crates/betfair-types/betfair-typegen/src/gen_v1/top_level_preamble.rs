use proc_macro2::TokenStream;
use quote::quote;

use super::injector::CodeInjector;
use super::GenV1GeneratorStrategy;

impl<T: CodeInjector> GenV1GeneratorStrategy<T> {
    pub(crate) fn generate_transport_layer(&self) -> TokenStream {
        quote! {
            use std::future::Future;
            use serde::{Serialize, de::DeserializeOwned};

            pub trait TransportLayer<T> where
                T: BetfairRpcRequest + Serialize + std::marker::Send + 'static,
                T::Res: DeserializeOwned + 'static
            {
                type Error;
                fn send_request(&self, request: T) -> impl Future<Output = Result<T::Res, Self::Error>> + Send + '_;
            }

            pub trait BetfairRpcRequest {
                type Res;
                type Error;

                fn method() -> &'static str;
            }
        }
    }
}
