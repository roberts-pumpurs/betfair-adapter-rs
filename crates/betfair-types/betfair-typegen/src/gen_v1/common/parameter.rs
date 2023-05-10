//! It's up to the caller to decide if he needs to reference the parameter or create a new one.

use betfair_xml_parser::common::{Parameter, ParameterItems};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use super::description::docs;
use crate::gen_v1::common::valid_values::create_enum_variants;

/// # Enforce the newtype pattern for a type that has not yet been seen.
///
/// ## Input
/// - `item` - the parameter to create
/// ## Output
/// - `ParameterName` - the name of the parameter
/// - `TokenStream` - the definition of the parameter that can be added at the necessary scope.
///
/// based from the input parameter, create a struct or an enum that follows the parameter
/// description. It can be used to enforce the newtype pattern.
pub(crate) fn create_parameter(item: Parameter) -> (ParameterName, TokenStream) {
    let docs = utils::extract_docs(item.items.iter());
    let capitalized_name = utils::capitalize_first_letter(&item.name);
    let type_name = Ident::new(&capitalized_name, Span::call_site());
    let valid_values = item.items.into_iter().find_map(|x| match x {
        ParameterItems::ValidValues(valid_values) => Some(valid_values),
        _ => None,
    });

    // Either create an enum or a struct
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
        let data_type = utils::primitive_type_lookup(&data_type).unwrap_or_else(|| {
            panic!("Unable to find a primitive type for {}. Please add it to the lookup", data_type)
        });
        // create a struct using the newtype pattern
        quote! {
            #docs
            pub struct #type_name(pub(crate) #data_type);
        }
    };

    (ParameterName(capitalized_name), definition)
}

/// A wrapper type for the ParameterName
pub(crate) struct ParameterName(String);

impl std::ops::Deref for ParameterName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// # Get the parameter in the form of a function argument
///
/// Example: `total: Option<Total>`
pub(crate) fn reference_parameter(item: Parameter) -> (ParameterName, TokenStream) {
    let variable_name = Ident::new(&item.name, Span::call_site());
    let capitalized_name = utils::capitalize_first_letter(&item.name);
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
    let docs = utils::extract_docs(item.items.iter());
    (
        ParameterName(capitalized_name),
        quote! {
            #docs
            #data_type
        },
    )
}

mod utils {
    use super::*;
    pub(super) fn primitive_type_lookup(input: &str) -> Option<TokenStream> {
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

    pub(super) fn extract_docs<'a>(items: impl Iterator<Item = &'a ParameterItems>) -> TokenStream {
        items
            .filter_map(|x| match x {
                betfair_xml_parser::common::ParameterItems::Description(desc) => Some(docs(desc)),
                betfair_xml_parser::common::ParameterItems::ValidValues(_) => None,
            })
            .collect::<TokenStream>()
    }
    pub(super) fn capitalize_first_letter(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use betfair_xml_parser::common::{Description, Parameter, ValidValues, Value};
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_create_basic_struct() {
        // Setup
        let input = Parameter {
            mandatory: Some(false),
            name: "total".to_string(),
            r#type: "double".to_string(),
            items: vec![ParameterItems::Description(Description {
                value: Some(
                    "Set a limit on total (matched + unmatched) bet exposure on market group"
                        .to_string(),
                ),
            })],
        };

        // Action
        let (_type_name, actual) = create_parameter(input);
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
                ParameterItems::Description(Description {
                    value: Some("the unique code for this error".to_string()),
                }),
                ParameterItems::ValidValues(ValidValues {
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
                            description: Description {
                                value: Some("Invalid input data".to_string()),
                            },
                        },
                    ],
                }),
            ],
        };

        // Action
        let (_type_name, actual) = create_parameter(input);
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
            items: vec![ParameterItems::Description(Description {
                value: Some(
                    "Set a limit on total (matched + unmatched) bet exposure on market group"
                        .to_string(),
                ),
            })],
        };

        // Action
        let (_type_name, actual) = reference_parameter(input);
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
            items: vec![ParameterItems::Description(Description {
                value: Some(
                    "Set a limit on total (matched + unmatched) bet exposure on market group"
                        .to_string(),
                ),
            })],
        };

        // Action
        let (_type_name, actual) = reference_parameter(input);
        let actual = actual.to_string();

        // Assert
        let expected = quote! {
            #[doc = "Set a limit on total (matched + unmatched) bet exposure on market group"]
            total: Total
        };
        assert_eq!(actual, expected.to_string());
    }
}
