use betfair_xml_parser::common::Description;
use proc_macro2::TokenStream;
use quote::quote;

/// Convert the description field into a doc comment
/// # Input
/// * `desc` - The description field
/// # Output
/// A doc comment
pub(crate) fn docs(desc: &Description) -> TokenStream {
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
