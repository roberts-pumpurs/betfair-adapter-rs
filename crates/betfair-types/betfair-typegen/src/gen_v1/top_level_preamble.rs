use proc_macro2::TokenStream;
use quote::quote;

use super::injector::CodeInjector;
use super::GenV1GeneratorStrategy;

impl<T: CodeInjector> GenV1GeneratorStrategy<T> {
    pub(crate) fn generate_transport_layer(&self) -> TokenStream {
        quote! {
            pub trait TransportLayer<T, V> {
                fn send_request(&self, request: T) -> V;
            }

            pub trait BetfairRpcCall<Req, Res> {
                fn call(&self, request: Req) -> Res;
            }

        }
    }
}
