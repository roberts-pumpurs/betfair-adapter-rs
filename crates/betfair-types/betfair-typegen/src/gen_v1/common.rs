pub mod valid_values {
    use betfair_xml_parser::common::{Description, ValidValues, Value};
    use proc_macro2::{Ident, Span, TokenStream};
    use quote::quote;
    use syn::parse::Parse;
    use syn::{Attribute, Meta};

    use super::description::docs;
    use crate::gen_v1::common::value::create_enum_variant;

    pub fn create_enum_variants(valid_values: ValidValues) -> TokenStream {
        let variants = valid_values
            .items
            .into_iter()
            .map(|x| create_enum_variant(x))
            .collect::<Vec<TokenStream>>();

        quote! {
            #(#variants)*
        }
    }

    #[cfg(test)]
    mod tests {
        use rstest::rstest;

        use super::*;

        #[rstest]
        fn test_enum_value_all_provided() {
            // Setup
            let input = ValidValues {
                items: vec![
                    Value {
                        id: Some("1".to_string()),
                        name: "TOO_MUCH_DATA".to_string(),
                        description: Description {
                            value: Some("The operation requested too much data".to_string()),
                        },
                    },
                    Value {
                        id: Some("2".to_string()),
                        name: "INVALID_INPUT_DATA".to_string(),
                        description: Description { value: Some("Invalid input data".to_string()) },
                    },
                ],
            };

            // Action
            let actual = create_enum_variants(input).to_string();

            // Assert
            let expected = quote! {
                #[doc = "The operation requested too much data"]
                TOO_MUCH_DATA = 1u32,
                #[doc = "Invalid input data"]
                INVALID_INPUT_DATA = 2u32,
            };
            assert_eq!(actual, expected.to_string());
        }
    }
}

pub mod value {
    use betfair_xml_parser::common::{Description, Value};
    use proc_macro2::{Ident, Span, TokenStream};
    use quote::quote;
    use syn::parse::Parse;
    use syn::{Attribute, Meta};

    use super::description::docs;

    pub fn create_enum_variant(value: Value) -> TokenStream {
        let doc = docs(&value.description);
        let id = value
            .id
            .as_ref()
            .map(|x| {
                let id = x.parse::<u32>().unwrap_or_default();
                quote! {
                    = #id
                }
            })
            .unwrap_or_default();
        let name = Ident::new(&value.name, Span::call_site());
        quote! {
            #doc
            #name #id,
        }
    }

    #[cfg(test)]
    mod tests {
        use rstest::rstest;

        use super::*;

        #[rstest]
        fn test_enum_value_all_provided() {
            // Setup
            let input = Value {
                id: Some("1".to_string()),
                name: "TOO_MUCH_DATA".to_string(),
                description: Description {
                    value: Some("The operation requested too much data".to_string()),
                },
            };

            // Action
            let actual = create_enum_variant(input).to_string();

            // Assert
            let expected = quote! {
                #[doc = "The operation requested too much data"]
                TOO_MUCH_DATA = 1u32,
            };
            assert_eq!(actual, expected.to_string());
        }

        #[rstest]
        fn test_enum_value_id_only() {
            // Setup
            let input = Value {
                id: Some("1".to_string()),
                name: "TOO_MUCH_DATA".to_string(),
                description: Description { value: None },
            };

            // Action
            let actual = create_enum_variant(input).to_string();

            // Assert
            let expected = quote! {
                TOO_MUCH_DATA = 1u32,
            };
            assert_eq!(actual, expected.to_string());
        }

        #[rstest]
        fn test_enum_value_name_only() {
            // Setup
            let input = Value {
                id: None,
                name: "TOO_MUCH_DATA".to_string(),
                description: Description { value: None },
            };

            // Action
            let actual = create_enum_variant(input).to_string();

            // Assert
            let expected = quote! {
                TOO_MUCH_DATA,
            };
            assert_eq!(actual, expected.to_string());
        }

        #[rstest]
        fn test_enum_value_name_and_description() {
            // Setup
            let input = Value {
                id: None,
                name: "TOO_MUCH_DATA".to_string(),
                description: Description {
                    value: Some("The operation requested too much data".to_string()),
                },
            };
            // Action
            let actual = create_enum_variant(input).to_string();

            // Assert
            let expected = quote! {
                #[doc = "The operation requested too much data"]
                TOO_MUCH_DATA,
            };
            assert_eq!(actual, expected.to_string());
        }
    }
}

pub mod description {
    use betfair_xml_parser::common::Description;
    use quote::__private::TokenStream;
    use quote::quote;

    pub fn docs(desc: &Description) -> TokenStream {
        desc.value
            .as_ref()
            .map(|desc| {
                quote! {
                    #[doc = #desc]
                }
            })
            .unwrap_or_default()
    }

    #[cfg(test)]
    mod tests {
        use rstest::rstest;

        use super::*;

        #[rstest]
        fn test_top_level_docs() {
            // Setup
            let desc = Description { value: Some("This is a test description".to_string()) };

            // Action
            let actual = docs(&desc).to_string();

            // Assert
            let expected = quote! {
                #[doc = "This is a test description"]
            };
            assert_eq!(actual, expected.to_string());
        }
    }
}

/// It's up to the caller to decide if he needs to reference the parameter or create a new one.
pub mod parameter {
    use std::slice::Iter;

    use betfair_xml_parser::common::{Description, Parameter, ValidValues, Value, parameter::Items};
    use proc_macro2::{Ident, Span, TokenStream};
    use quote::quote;
    use syn::parse::Parse;
    use syn::{Attribute, Meta};

    use super::description::docs;
    use crate::gen_v1::common::{value::create_enum_variant, valid_values::create_enum_variants};

    fn primitive_type_lookup(input: &str) -> Option<TokenStream> {
        match input {
            "string" => Some(quote! { String }),
            "double" => Some(quote! { Decimal }),
            "i32" => Some(quote! { i32 }),
            "i64" => Some(quote! { i64 }),
            "dateTime" => Some(quote! { DateTime<Utc> }),
            "bool" => Some(quote! { bool }),
            _ => None,
        }
    }

    fn extract_docs<'a>(items: impl Iterator<Item = &'a Items>) -> TokenStream {
        items
            .filter_map(|x| match x {
                betfair_xml_parser::common::parameter::Items::Description(desc) => Some(docs(desc)),
                betfair_xml_parser::common::parameter::Items::ValidValues(_) => None,
            })
            .collect::<TokenStream>()
    }
    fn capitalize_first_letter(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
        }
    /// based from the input parameter, create a struct or an enum that follows the parameter
    /// description. It can be used to enforce the newtype pattern.
    pub fn create_parameter(item: Parameter) -> (ParameterName, TokenStream) {
        let docs = extract_docs(item.items.iter());
        let capitalized_name = capitalize_first_letter(&item.name);
        let type_name = Ident::new(&capitalized_name, Span::call_site());
        let valid_values = item
            .items
            .into_iter()
            .find_map(|x| match x {
                Items::ValidValues(valid_values) => Some(valid_values),
                _ => None,
            });
        let definition = if let Some(valid_values) = valid_values {
            // create an enum
            let enum_variants = create_enum_variants(valid_values);
            quote! {
                #docs
                pub enum #type_name {
                    #enum_variants
                }
            }
        } else {
            let data_type = item.r#type;
            let data_type = primitive_type_lookup(&data_type).unwrap_or_else(|| {
                panic!(
                    "Unable to find a primitive type for {}. Please add it to the lookup",
                    data_type
                )
            });
            // create a struct using the newtype pattern
            quote! {
                #docs
                pub struct #type_name(pub(crate) #data_type);
            }
        };

        (ParameterName(capitalized_name) , definition)
    }

    pub struct ParameterName(pub String);
    /// Get the typename for the parameter, in the appropriate format
    pub fn reference_parameter(item: Parameter) -> (ParameterName, TokenStream) {
        let variable_name = Ident::new(&item.name, Span::call_site());
        let capitalized_name = capitalize_first_letter(&item.name);
        let type_name = Ident::new(&capitalized_name, Span::call_site());
        let data_type = match item.mandatory {
            Some(true) => {
                quote! {
                    #variable_name: #type_name
                }
            }
            _ => {
                quote! {
                    #variable_name: Option<#type_name>
                }
            }
        };
        let docs = extract_docs(item.items.iter());
        (
            ParameterName(capitalized_name),
            quote! {
                #docs
                #data_type
            },
        )
    }

    #[cfg(test)]
    mod tests {
        use betfair_xml_parser::common::{parameter, Parameter};
        use rstest::rstest;

        use super::*;

        #[rstest]
        fn test_create_basic_struct() {
            // Setup
            let input = Parameter {
                mandatory: Some(false),
                name: "total".to_string(),
                r#type: "double".to_string(),
                items: vec![parameter::Items::Description(Description {
                    value: Some(
                        "Set a limit on total (matched + unmatched) bet exposure on market group"
                            .to_string(),
                    ),
                })],
            };

            // Action
            let (type_name, actual) = create_parameter(input);
            let actual = actual.to_string();

            // Assert
            let expected = quote! {
                #[doc = "Set a limit on total (matched + unmatched) bet exposure on market group"]
                pub struct Total(pub(crate) Decimal);
            };
            assert_eq!(actual, expected.to_string());
        }

        #[rstest]
        fn test_create_basic_enum() {
            // Setup
            let input = Parameter {
                mandatory: None,
                name: "errorCode".to_string(),
                r#type: "string".to_string(),
                items: vec![
                    parameter::Items::Description(Description {
                        value: Some("the unique code for this error".to_string()),
                    }),
                    parameter::Items::ValidValues(ValidValues {
                        items: vec![
                            Value {
                                id: Some("1".to_string()),
                                name: "TOO_MUCH_DATA".to_string(),
                                description: Description {
                                    value: Some(
                                        "The operation requested too much data".to_string(),
                                    ),
                                },
                            },
                            Value {
                                id: Some("2".to_string()),
                                name: "INVALID_INPUT_DATA".to_string(),
                                description: Description {
                                    value: Some("Invalid input data".to_string()),
                                },
                            },
                        ],
                    }),
                ],
            };

            // Action
            let (type_name, actual) = create_parameter(input);
            let actual = actual.to_string();

            // Assert
            let expected = quote! {
                #[doc = "the unique code for this error"]
                pub enum ErrorCode {
                    #[doc = "The operation requested too much data"]
                    TOO_MUCH_DATA = 1u32,
                    #[doc = "Invalid input data"]
                    INVALID_INPUT_DATA = 2u32,
                }
            };
            assert_eq!(actual, expected.to_string());
        }

        #[rstest]
        fn test_reference_parameter() {
            // Setup
            let input = Parameter {
                mandatory: Some(false),
                name: "total".to_string(),
                r#type: "double".to_string(),
                items: vec![parameter::Items::Description(Description {
                    value: Some(
                        "Set a limit on total (matched + unmatched) bet exposure on market group"
                            .to_string(),
                    ),
                })],
            };

            // Action
            let (type_name, actual) = reference_parameter(input);
            let actual = actual.to_string();

            // Assert
            let expected = quote! {
                #[doc = "Set a limit on total (matched + unmatched) bet exposure on market group"]
                total: Option<Total>
            };
            assert_eq!(actual, expected.to_string());
        }

        #[rstest]
        fn test_reference_parameter_mandatory() {
            // Setup
            let input = Parameter {
                mandatory: Some(true),
                name: "total".to_string(),
                r#type: "double".to_string(),
                items: vec![parameter::Items::Description(Description {
                    value: Some(
                        "Set a limit on total (matched + unmatched) bet exposure on market group"
                            .to_string(),
                    ),
                })],
            };

            // Action
            let (type_name, actual) = reference_parameter(input);
            let actual = actual.to_string();

            // Assert
            let expected = quote! {
                #[doc = "Set a limit on total (matched + unmatched) bet exposure on market group"]
                total: Total
            };
            assert_eq!(actual, expected.to_string());
        }
    }
}
