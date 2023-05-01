use betfair_xml_parser::simple_type::SimpleType;
use quote::__private::TokenStream;
use quote::quote;

// pub fn generate_simple_type(item: &SimpleType) -> TokenStream {
//     let desc = desc.value.as_ref().map(|x| x.as_str()).unwrap_or("");
//     let module_doc = add_static_header(desc);

//     let generated_code = quote! {
//         #![doc = #module_doc]
//     };
//     generated_code
// }

// #[cfg(test)]
// mod tests {
//     use rstest::rstest;

//     use super::*;

//     #[rstest]
//     fn test_top_level_docs() {
//         let desc = "This is a test description".to_string();
//         let expected =
//             r#"#![doc = "THIS IS A GENERATED FILE. DO NOT EDIT. \n\nThis is a test description"]"#;
//         let desc = Description { value: Some(desc) };
//         let actual = generate_top_level_documentation(&desc).to_string();
//         assert_eq!(actual, expected);
//     }
// }
