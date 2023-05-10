mod value;

use betfair_xml_parser::common::ValidValues;
use proc_macro2::TokenStream;
use quote::quote;
use value::create_enum_variant;

/// Create the top level enum variants for the enum
pub(crate) fn create_enum_variants(valid_values: ValidValues) -> TokenStream {
    let variants =
        valid_values.items.into_iter().map(create_enum_variant).collect::<Vec<TokenStream>>();

    quote! {
        #(#variants)*
    }
}

#[cfg(test)]
mod tests {
    use betfair_xml_parser::common::{Description, Value};
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
