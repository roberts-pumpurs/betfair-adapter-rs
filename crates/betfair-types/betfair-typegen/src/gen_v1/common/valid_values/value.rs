use betfair_xml_parser::common::Value;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::gen_v1::common::description::docs;

/// Create a single enum variant
pub(crate) fn create_enum_variant(value: Value) -> TokenStream {
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
    use betfair_xml_parser::common::Description;
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
