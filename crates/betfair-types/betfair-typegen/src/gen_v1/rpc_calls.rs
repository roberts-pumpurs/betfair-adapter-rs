use proc_macro2::TokenStream;
use quote::quote;

use super::GenV1;
use crate::ast::rpc_calls::RpcCall;
use crate::gen_v1::documentation::CommentParse;

impl GenV1 {
    pub(crate) fn generate_rpc_call(&self, data_type: &RpcCall) -> TokenStream {
        let description = data_type.description.as_slice().object_comment();
        let module_name = data_type.name.ident_snake();
        let return_type = self.return_type(data_type);
        let parameter = self.parameter(data_type);
        let call_traits = self.generate_call_traits(data_type);
        quote! {
            #description
            pub mod #module_name {
                #return_type
                #parameter
                #call_traits
            }
        }
    }

    pub(crate) fn generate_transport_layer(&self) -> TokenStream {
        quote! {
            trait TransportLayer<T: Debug, V: Debug> {
                fn send_request(&self, request: T) -> V;
            }

            trait BetfairRpcCall<Req, Res> {
                fn call(&self, request: Req) -> Res;
            }

        }
    }

    fn generate_call_traits(&self, data_type: &RpcCall) -> TokenStream {
        let description = data_type.description.as_slice().object_comment();
        quote! {
            impl<Any> BetfairRpcCall<Parameters, ReturnType> for Any
            where
                Self: TransportLayer<Parameters, ReturnType>,
            {
                #description
                fn call(&self, request: Parameters) -> ReturnType {
                    self.send_request(request)
                }
            }
        }
    }

    fn return_type(&self, data_type: &RpcCall) -> TokenStream {
        let description = data_type.returns.description.as_slice().object_comment();
        if let Some(exception) = &data_type.exception {
            let err_data_type = self.type_resolver.resolve_type(&exception.data_type);
            let error_docs = exception.description.as_slice().object_comment();
            let ok_type = self.type_resolver.resolve_type(&data_type.returns.data_type);
            quote! {
                #error_docs
                pub type Exception = #err_data_type;

                #description
                pub type ReturnType = Result<#ok_type, Exception>;
            }
        } else {
            let return_type = self.type_resolver.resolve_type(&data_type.returns.data_type);
            quote! {
                #description
                pub type ReturnType = #return_type;
            }
        }
    }

    fn parameter(&self, data_type: &RpcCall) -> TokenStream {
        let fields = data_type
            .params
            .iter()
            .map(|field| {
                let description = field.description.as_slice().object_comment();
                let name = field.name.ident_snake();
                let data_type = self.type_resolver.resolve_type(&field.data_type);
                let data_type = if !field.mandatory {
                    quote! {
                        Option<#data_type>
                    }
                } else {
                    quote! {
                        #data_type
                    }
                };

                quote! {
                    #description
                    pub #name: #data_type,
                }
            })
            .collect::<Vec<_>>();
        quote! {
            pub struct Parameters {
                #(#fields),*
            }
        }
    }
}

#[cfg(test)]
mod test {

    use proptest::prelude::*;

    use super::super::test::GEN_V1;
    use super::*;
    use crate::ast::rpc_calls::{Exception, Param, Returns};
    use crate::ast::types::{Comment, DataTypeParameter, Name};

    #[rstest::rstest]
    fn test_generate_rpc_module_mandatory_parameter() {
        // Setup
        let rpc_call = RpcCall {
            name: Name("createDeveloperAppKeys".to_string()),
            params: vec![Param {
                name: Name("appName".to_string()),
                data_type: DataTypeParameter::new("string".to_string()),
                description: vec![Comment::new("A Display name for the application.".to_string())],
                mandatory: true,
            }],
            returns: Returns {
                data_type: DataTypeParameter::new("DeveloperApp".to_string()),
                description: vec![Comment::new(
                    "A map of application keys, one marked ACTIVE, and the other DELAYED"
                        .to_string(),
                )],
            },
            exception: Some(Exception {
                data_type: DataTypeParameter::new("AccountAPINGException".to_string()),
                description: vec![Comment::new(
                    "Generic exception that is thrown if this operation fails for any reason."
                        .to_string(),
                )],
            }),
            description: vec![Comment::new(
                "Create 2 application keys for given user; one active and the other delayed"
                    .to_string(),
            )],
        };

        // Execute
        let actual = GEN_V1.generate_rpc_call(&rpc_call);

        // Assert
        let expected = quote! {
            #[doc = "Create 2 application keys for given user; one active and the other delayed"]
            pub mod create_developer_app_keys {
                #[doc = "Generic exception that is thrown if this operation fails for any reason."]
                pub type Exception = AccountApingException;

                #[doc = "A map of application keys, one marked ACTIVE, and the other DELAYED"]
                pub type ReturnType = Result<DeveloperApp, Exception>;

                pub struct Parameters {
                    #[doc = "A Display name for the application."]
                    pub app_name: String,
                }

                impl<Any> BetfairRpcCall<Parameters, ReturnType> for Any
                where
                Self: TransportLayer<Parameters, ReturnType>,
                {
                    #[doc = "Create 2 application keys for given user; one active and the other delayed"]
                    fn call(&self, request: Parameters) -> ReturnType {
                        self.send_request(request)
                    }
                }
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    prop_compose! {
        fn exceptions()(
            present in any::<bool>()
        ) -> Option<Exception> {
            if present {
                Some(Exception {
                    data_type: DataTypeParameter::new("AccountAPINGException".to_string()),
                    description: vec![Comment::new(
                        "Generic exception that is thrown if this operation fails for any reason."
                            .to_string(),
                    )],
                })
            } else {
                None
            }
      }
    }

    use crate::gen_v1::type_resolver::tests::valid_data_types;

    prop_compose! {
        fn single_parameter()(
            mandatory in any::<bool>(),
            amount_comments in 0_usize..10_usize,
            data_type in valid_data_types(),
            param_name in proptest::sample::select(vec!["argument1", "argument2", "argument3"]),
        ) -> Param {
            Param {
                name: Name(param_name.to_string()),
                data_type,
                description: vec![Comment::new("A Display name for the application.".to_string()); amount_comments],
                mandatory,
            }
        }
    }

    proptest! {
        #[test]
        fn generate_rpc_calls(exception in exceptions(), params in proptest::collection::vec(single_parameter(), 0..100)) {
            // Setup
            let params_len = params.len();
            let rpc_call = RpcCall {
                name: Name("createDeveloperAppKeys".to_string()),
                params,
                returns: Returns {
                    data_type: DataTypeParameter::new("DeveloperApp".to_string()),
                    description: vec![Comment::new(
                        "A map of application keys, one marked ACTIVE, and the other DELAYED"
                            .to_string(),
                    )],
                },
                exception: exception.clone(),
                description: vec![Comment::new(
                    "Create 2 application keys for given user; one active and the other delayed"
                        .to_string(),
                )],
            };

            // Execute
            let actual = GEN_V1.generate_rpc_call(&rpc_call);
            let actual = actual.to_string();

            // Assert
            prop_assert!(actual.contains("pub mod create_developer_app_keys"));
            match exception {
                Some(_) => {
                    let expected_exception = quote! { pub type Exception = AccountApingException; };
                    prop_assert!(actual.contains(&expected_exception.to_string()));
                    let expected_return_type =
                        quote! { pub type ReturnType = Result<DeveloperApp, Exception>; };
                    prop_assert!(actual.contains(&expected_return_type.to_string()));
                }
                None => {
                    let expected_exception = quote! { pub type Exception };
                    prop_assert!(!actual.contains(&expected_exception.to_string()));
                    let expected_return_type = quote! { pub type ReturnType = DeveloperApp; };
                    prop_assert!(actual.contains(&expected_return_type.to_string()));
                }
            }

            let empty_parameters = quote! { pub struct Parameters {} };
            prop_assert!((params_len == 0) == actual.contains(&empty_parameters.to_string()));
        }
    }
}
