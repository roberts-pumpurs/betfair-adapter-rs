use proc_macro2::TokenStream;
use quote::quote;

use super::GenV1GeneratorStrategy;
use super::injector::CodeInjector;
use crate::aping_ast::data_type::{
    DataType, EnumValue, StructField, StructValue, TypeAlias, ValidEnumValue,
};
use crate::aping_ast::types::Name;
use crate::gen_v1::documentation::CommentParse as _;

impl<T: CodeInjector> GenV1GeneratorStrategy<T> {
    pub(crate) fn generate_data_type(&self, data_type: &DataType) -> TokenStream {
        let description = data_type.description.as_slice().object_comment();

        let inner = match data_type.variant {
            crate::aping_ast::data_type::DataTypeVariant::EnumValue(ref x) => {
                self.generate_enum_value(x)
            }
            crate::aping_ast::data_type::DataTypeVariant::StructValue(ref x) => {
                self.generate_struct_value(x)
            }
            crate::aping_ast::data_type::DataTypeVariant::TypeAlias(ref x) => {
                match self.generate_type_alias(x) {
                    Some(x) => x,
                    None => return quote! {},
                }
            }
        };

        quote! {
            #description
            #inner
        }
    }

    fn generate_enum_value(&self, enum_value: &EnumValue) -> TokenStream {
        fn generate_valid_enum_value(
            enum_variant_derives: &TokenStream,
            valid_enum_value: &ValidEnumValue,
        ) -> TokenStream {
            let name = valid_enum_value.name.ident_pascal();
            let serde_version = valid_enum_value.name.0.clone();
            if valid_enum_value.id.is_empty() {
                let description = valid_enum_value.description.as_slice().object_comment();

                quote! {
                    #description
                    #[serde(rename = #serde_version)]
                    #enum_variant_derives
                    #name,
                }
            } else {
                let parsed_id = valid_enum_value.id.parse::<i128>();
                let name_with_attributes = parsed_id.as_ref().map_or_else(
                    |_| {
                        let string_id = &valid_enum_value.id;
                        quote! {
                            #[serde(rename = #string_id)]
                            #name
                        }
                    },
                    |numeric_id| {
                        quote! {
                            #[serde(rename = #numeric_id)]
                            #name = #numeric_id
                        }
                    },
                );
                let description = valid_enum_value.description.as_slice().object_comment();

                quote! {
                    #description
                    #name_with_attributes,
                }
            }
        }

        let enum_variant_derives = self.code_injector.enum_variant_derives();
        let name = enum_value.name.ident_pascal();
        let valid_values = enum_value
            .valid_values
            .iter()
            .map(|x| generate_valid_enum_value(&enum_variant_derives.clone(), x))
            .fold(quote! {}, |acc, i| {
                quote! {
                    #acc
                    #i
                }
            });

        let enum_derives = self.code_injector.enum_derives();
        quote! {
            #enum_derives
            pub enum #name {
                #valid_values
            }
        }
    }

    fn generate_struct_value(&self, struct_value: &StructValue) -> TokenStream {
        fn generate_struct_field<T: CodeInjector>(
            generator: &GenV1GeneratorStrategy<T>,
            struct_field: &StructField,
        ) -> TokenStream {
            let description = struct_field
                .description
                .iter()
                .map(super::super::aping_ast::types::Comment::object_comment)
                .fold(quote! {}, |acc, i| {
                    quote! {
                        #acc
                        #i
                    }
                });

            let name = {
                if struct_field.name.0.as_str() == "type" {
                    Name("r#type".to_owned()).ident_snake()
                } else if struct_field.name.0.as_str() == "async" {
                    Name("r#async".to_owned()).ident_snake()
                } else {
                    struct_field.name.ident_snake()
                }
            };
            let original_name = struct_field.name.0.as_str();
            let data_type = match generator
                .type_resolver
                .resolve_type(&struct_field.data_type)
            {
                Ok(type_) => type_,
                Err(err) => {
                    let err_msg = err.to_string();
                    return quote! { compile_error!(#err_msg); };
                }
            };
            let struct_parameter_derives = generator.code_injector.struct_parameter_derives();
            if struct_field.mandatory {
                quote! {
                    #description
                    #struct_parameter_derives
                    #[serde(rename = #original_name)]
                    pub #name: #data_type,
                }
            } else {
                let extra = if struct_field.data_type.as_str().eq("double")
                    || struct_field.data_type.as_str().eq("float")
                {
                    quote! {
                        #[serde(deserialize_with = "super::deserialize_decimal_option", default)]
                    }
                } else {
                    quote! {}
                };
                quote! {
                    #description
                    #struct_parameter_derives
                    #[serde(skip_serializing_if = "Option::is_none")]
                    #[serde(rename = #original_name)]
                    #extra
                    #[builder(default, setter(strip_option))]
                    pub #name: Option<#data_type>,
                }
            }
        }
        let name = struct_value.name.ident_pascal();
        let fields = struct_value
            .fields
            .iter()
            .map(|x| generate_struct_field(self, x))
            .fold(quote! {}, |acc, i| {
                quote! {
                    #acc
                    #i
                }
            });

        let struct_derives = self.code_injector.struct_derives();
        let is_error_type = struct_value.name.0.ends_with("Exception");
        let append = if is_error_type {
            quote! {
                impl std::error::Error for #name {}
                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{:?}", self)
                    }
                }
            }
        } else {
            quote! {}
        };

        quote! {
            #struct_derives
            pub struct #name {
                #fields
            }
            #append
        }
    }

    fn generate_type_alias(&self, type_alias: &TypeAlias) -> Option<TokenStream> {
        let name = type_alias.name.ident_pascal();
        let data_type = match self.type_resolver.resolve_type(&type_alias.data_type) {
            Ok(type_) => type_,
            Err(err) => {
                let err_msg = err.to_string();
                return Some(quote! {
                    compile_error!(#err_msg);
                });
            }
        };
        let type_alias_derives = self.code_injector.type_alias_derives();

        let types_to_skip = [
            "Price",
            "Size",
            "CustomerOrderRef",
            "CustomerRef",
            "CustomerStrategyRef",
        ];

        if types_to_skip.contains(&type_alias.name.0.as_str()) {
            return None;
        }

        let extra = if type_alias.data_type.as_str().eq("i64") {
            quote! {
                #[derive(Copy)]
            }
        } else {
            quote! {}
        };

        Some(quote! {
            #type_alias_derives
            #extra
            pub struct #name (pub #data_type);
        })
    }
}

#[cfg(test)]
mod test {

    use pretty_assertions::assert_eq;

    use super::super::test::gen_v1;
    use super::*;
    use crate::aping_ast::types::{Comment, DataTypeParameter, Name};
    use crate::gen_v1::injector::CodeInjectorV1;

    #[rstest::rstest]
    fn test_generate_structure(gen_v1: GenV1GeneratorStrategy<CodeInjectorV1>) {
        // Setup
        let data_type = DataType {
            name: Name("MarketFilter".to_owned()),
            variant: crate::aping_ast::data_type::DataTypeVariant::StructValue(StructValue {
                name: Name("MarketFilter".to_owned()),
                fields: vec![
                    StructField {
                        name: Name("textQuery".to_owned()),
                        mandatory: false,
                        data_type: DataTypeParameter::new("string".to_owned()),
                        description: vec![
                            Comment::new("Restrict markets by any text associated with the market such as the Name, Event, Competition, etc. You can include a wildcard (*) character as long as it is not the first character.".to_owned()),
                            Comment::new("Comment 2.".to_owned()),
                        ],
                    },
                ],
            }),
            description: vec![
                Comment::new("The filter to select desired markets. All markets that match the criteria in the filter are selected.".to_owned()),
                Comment::new("Comment 2.".to_owned()),
            ],
        };

        // Execute
        let actual = gen_v1.generate_data_type(&data_type);

        // Assert
        let expected = quote! {
            #[doc = "The filter to select desired markets. All markets that match the criteria in the filter are selected."]
            #[doc = "Comment 2."]
            #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, TypedBuilder)]
            #[serde(rename_all = "camelCase")]
            pub struct MarketFilter {
                #[doc = "Restrict markets by any text associated with the market such as the Name, Event, Competition, etc. You can include a wildcard (*) character as long as it is not the first character."]
                #[doc = "Comment 2."]
                #[serde(skip_serializing_if="Option::is_none")]
                #[serde (rename = "textQuery")]
                #[builder(default, setter(strip_option))]
                pub text_query: Option<std::sync::Arc<String> >,
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[rstest::rstest]
    fn test_generate_enum(gen_v1: GenV1GeneratorStrategy<CodeInjectorV1>) {
        // Setup
        let data_type = DataType {
            name: Name("MarketProjection".to_owned()),
            variant: crate::aping_ast::data_type::DataTypeVariant::EnumValue(EnumValue {
                name: Name("MarketProjection".to_owned()),
                valid_values: vec![
                    crate::aping_ast::data_type::ValidEnumValue {
                        id: "0".to_owned(),
                        name: Name("COMPETITION".to_owned()),
                        description: vec![Comment::new("If no value is passed into the marketProjections parameter then the competition will not be returned with marketCatalogue".to_owned())],
                    },
                    crate::aping_ast::data_type::ValidEnumValue {
                        id: "1".to_owned(),
                        name: Name("EVENT".to_owned()),
                        description: vec![Comment::new("If no value is passed into the marketProjections parameter then the event will not be returned with marketCatalogue".to_owned())],
                    },
                    crate::aping_ast::data_type::ValidEnumValue {
                        id: "2".to_owned(),
                        name: Name("EVENT_TYPE".to_owned()),
                        description: vec![Comment::new("If no value is passed into the marketProjections parameter then the eventType will not be returned with marketCatalogue".to_owned())],
                    },
                    crate::aping_ast::data_type::ValidEnumValue {
                        id: "3".to_owned(),
                        name: Name("MARKET_START_TIME".to_owned()),
                        description: vec![Comment::new("If no value is passed into the marketProjections parameter then the marketStartTime will not be returned with marketCatalogue".to_owned())],
                    },
                ],
            }),
            description: vec![Comment::new("Type of price data returned by listMarketBook operation".to_owned())],
        };

        // Execute
        let actual = gen_v1.generate_data_type(&data_type);

        // Assert
        let expected = quote! {
            #[doc = "Type of price data returned by listMarketBook operation"]
            #[derive(Clone , Copy , Debug , Eq , PartialEq , Ord , PartialOrd , Hash , Serialize , Deserialize)]
            pub enum MarketProjection {
                #[doc = "If no value is passed into the marketProjections parameter then the competition will not be returned with marketCatalogue"]
                #[serde(rename = 0i128)]
                Competition = 0i128,
                #[doc = "If no value is passed into the marketProjections parameter then the event will not be returned with marketCatalogue"]
                #[serde(rename = 1i128)]
                Event = 1i128,
                #[doc = "If no value is passed into the marketProjections parameter then the eventType will not be returned with marketCatalogue"]
                #[serde(rename = 2i128)]
                EventType = 2i128,
                #[doc = "If no value is passed into the marketProjections parameter then the marketStartTime will not be returned with marketCatalogue"]
                #[serde(rename = 3i128)]
                MarketStartTime = 3i128,
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[rstest::rstest]
    fn test_generate_enum_2(gen_v1: GenV1GeneratorStrategy<CodeInjectorV1>) {
        // Setup
        let data_type = DataType {
            name: Name("MarketProjection".to_owned()),
            variant: crate::aping_ast::data_type::DataTypeVariant::EnumValue(EnumValue {
                name: Name("MarketProjection".to_owned()),
                valid_values: vec![
                    crate::aping_ast::data_type::ValidEnumValue {
                        id: String::new(),
                        name: Name("COMPETITION".to_owned()),
                        description: vec![Comment::new("If no value is passed into the marketProjections parameter then the competition will not be returned with marketCatalogue".to_owned())],
                    },
                    crate::aping_ast::data_type::ValidEnumValue {
                        id: String::new(),
                        name: Name("EVENT".to_owned()),
                        description: vec![Comment::new("If no value is passed into the marketProjections parameter then the event will not be returned with marketCatalogue".to_owned())],
                    },
                    crate::aping_ast::data_type::ValidEnumValue {
                        id: String::new(),
                        name: Name("EVENT_TYPE".to_owned()),
                        description: vec![Comment::new("If no value is passed into the marketProjections parameter then the eventType will not be returned with marketCatalogue".to_owned())],
                    },
                    crate::aping_ast::data_type::ValidEnumValue {
                        id: String::new(),
                        name: Name("MARKET_START_TIME".to_owned()),
                        description: vec![Comment::new("If no value is passed into the marketProjections parameter then the marketStartTime will not be returned with marketCatalogue".to_owned())],
                    },
                ],
            }),
            description: vec![Comment::new("Type of price data returned by listMarketBook operation".to_owned())],
        };

        // Execute
        let actual = gen_v1.generate_data_type(&data_type);

        // Assert
        let expected = quote! {
            #[doc = "Type of price data returned by listMarketBook operation"]
            #[derive(Clone , Copy , Debug , Eq , PartialEq , Ord , PartialOrd , Hash , Serialize , Deserialize)]
            pub enum MarketProjection {
                #[doc = "If no value is passed into the marketProjections parameter then the competition will not be returned with marketCatalogue"]
                #[serde(rename = "COMPETITION")]
                Competition,
                #[doc = "If no value is passed into the marketProjections parameter then the event will not be returned with marketCatalogue"]
                #[serde(rename = "EVENT")]
                Event,
                #[doc = "If no value is passed into the marketProjections parameter then the eventType will not be returned with marketCatalogue"]
                #[serde(rename = "EVENT_TYPE")]
                EventType,
                #[doc = "If no value is passed into the marketProjections parameter then the marketStartTime will not be returned with marketCatalogue"]
                #[serde(rename = "MARKET_START_TIME")]
                MarketStartTime,
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[rstest::rstest]
    fn test_generate_type_alias(gen_v1: GenV1GeneratorStrategy<CodeInjectorV1>) {
        // Setup
        let data_type = DataType {
            name: Name("MarketProjection".to_owned()),
            variant: crate::aping_ast::data_type::DataTypeVariant::TypeAlias(TypeAlias {
                name: Name("MarketProjection".to_owned()),
                data_type: DataTypeParameter::new("string".to_owned()),
            }),
            description: vec![Comment::new(
                "Type of price data returned by listMarketBook operation".to_owned(),
            )],
        };

        // Execute
        let actual = gen_v1.generate_data_type(&data_type);

        // Assert
        let expected = quote! {
            #[doc = "Type of price data returned by listMarketBook operation"]
            #[derive(Debug , Deserialize , Serialize , Clone , PartialEq , Eq , Hash)]
            #[serde (rename_all = "camelCase")]
            pub struct MarketProjection(pub std::sync::Arc<String>);
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
