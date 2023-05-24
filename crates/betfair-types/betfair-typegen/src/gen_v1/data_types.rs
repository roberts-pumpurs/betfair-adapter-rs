use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use super::GenV1;
use crate::ast::data_type::{
    DataType, EnumValue, StructField, StructValue, TypeAlias, ValidEnumValue,
};
use crate::gen_v1::documentation::CommentParse;

impl GenV1 {
    pub(crate) fn generate_data_type(&self, data_type: &DataType) -> TokenStream {
        let description = data_type.description.as_slice().object_comment();

        let inner = match &data_type.variant {
            crate::ast::data_type::DataTypeVariant::EnumValue(x) => self.generate_enum_value(x),
            crate::ast::data_type::DataTypeVariant::StructValue(x) => self.generate_struct_value(x),
            crate::ast::data_type::DataTypeVariant::TypeAlias(x) => self.generate_type_alias(x),
        };

        quote! {
            #description
            #inner
        }
    }

    fn generate_enum_value(&self, enum_value: &EnumValue) -> TokenStream {
        fn generate_valid_enum_value(valid_enum_value: &ValidEnumValue) -> TokenStream {
            let name = valid_enum_value.name.ident();
            if valid_enum_value.id.is_empty() {
                let description = valid_enum_value.description.as_slice().object_comment();

                quote! {
                    #description
                    #name,
                }
            } else {
                let id = &valid_enum_value.id.parse::<i128>();
                let name = match id {
                    Ok(id) => {
                        quote! {
                            #name = #id
                        }
                    }
                    Err(_) => {
                        let id = &valid_enum_value.id;
                        quote! {
                            #name = #id
                        }
                    }
                };
                let description = valid_enum_value.description.as_slice().object_comment();

                quote! {
                    #description
                    #name,
                }
            }
        }

        let name = enum_value.name.ident();
        let valid_values = enum_value.valid_values.iter().map(generate_valid_enum_value).fold(
            quote! {},
            |acc, i| {
                quote! {
                    #acc
                    #i
                }
            },
        );

        quote! {
            #[derive(Debug, Clone, PartialEq)]
            pub enum #name {
                #valid_values
            }
        }
    }

    fn generate_struct_value(&self, struct_value: &StructValue) -> TokenStream {
        fn generate_struct_field(struct_field: &StructField) -> TokenStream {
            let description = struct_field.description.iter().map(|x| x.object_comment()).fold(
                quote! {},
                |acc, i| {
                    quote! {
                        #acc
                        #i
                    }
                },
            );

            let name = struct_field.name.ident();
            let data_type = Ident::new(&struct_field.data_type, Span::call_site()); // TODO we need to parse the data type here
            if struct_field.mandatory {
                quote! {
                    #description
                    pub #name: #data_type,
                }
            } else {
                quote! {
                    #description
                    pub #name: Option<#data_type>,
                }
            }
        }
        let name = struct_value.name.ident();
        let fields =
            struct_value.fields.iter().map(generate_struct_field).fold(quote! {}, |acc, i| {
                quote! {
                    #acc
                    #i
                }
            });

        quote! {
            #[derive(Debug, Clone, PartialEq)]
            pub struct #name {
                #fields
            }
        }
    }

    fn generate_type_alias(&self, type_alias: &TypeAlias) -> TokenStream {
        let name = type_alias.name.ident();
        let data_type = Ident::new(&type_alias.data_type, Span::call_site()); // TODO we need to parse the data type here

        quote! {
            #[derive(Debug, Clone, PartialEq)]
            pub struct #name (pub #data_type);
        }
    }
}

#[cfg(test)]
mod test {

    use super::super::test::GEN_V1;
    use super::*;
    use crate::ast::{Comment, Name};

    #[rstest::rstest]
    fn test_generate_structure() {
        // Setup
        let data_type = DataType {
            name: Name("MarketFilter".to_string()),
            variant: crate::ast::data_type::DataTypeVariant::StructValue(StructValue {
                name: Name("MarketFilter".to_string()),
                fields: vec![
                    StructField {
                        name: Name("textQuery".to_string()),
                        mandatory: false,
                        data_type: "String".to_string(),
                        description: vec![
                            Comment::new("Restrict markets by any text associated with the market such as the Name, Event, Competition, etc. You can include a wildcard (*) character as long as it is not the first character.".to_string()),
                            Comment::new("Comment 2.".to_string()),
                        ],
                    },
                ],
            }),
            description: vec![
                Comment::new("The filter to select desired markets. All markets that match the criteria in the filter are selected.".to_string()),
                Comment::new("Comment 2.".to_string()),
            ],
        };

        // Execute
        let actual = GEN_V1.generate_data_type(&data_type);

        // Assert
        let expected = quote! {
            #[doc = "The filter to select desired markets. All markets that match the criteria in the filter are selected."]
            #[doc = "Comment 2."]
            #[derive(Debug, Clone, PartialEq)]
            pub struct MarketFilter {
                #[doc = "Restrict markets by any text associated with the market such as the Name, Event, Competition, etc. You can include a wildcard (*) character as long as it is not the first character."]
                #[doc = "Comment 2."]
                pub textQuery: Option<String>,
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[rstest::rstest]
    fn test_generate_enum() {
        // Setup
        let data_type = DataType {
            name: Name("MarketProjection".to_string()),
            variant: crate::ast::data_type::DataTypeVariant::EnumValue(EnumValue {
                name: Name("MarketProjection".to_string()),
                valid_values: vec![
                    crate::ast::data_type::ValidEnumValue {
                        id: "0".to_string(),
                        name: Name("COMPETITION".to_string()),
                        description: vec![Comment::new("If no value is passed into the marketProjections parameter then the competition will not be returned with marketCatalogue".to_string())],
                    },
                    crate::ast::data_type::ValidEnumValue {
                        id: "1".to_string(),
                        name: Name("EVENT".to_string()),
                        description: vec![Comment::new("If no value is passed into the marketProjections parameter then the event will not be returned with marketCatalogue".to_string())],
                    },
                    crate::ast::data_type::ValidEnumValue {
                        id: "2".to_string(),
                        name: Name("EVENT_TYPE".to_string()),
                        description: vec![Comment::new("If no value is passed into the marketProjections parameter then the eventType will not be returned with marketCatalogue".to_string())],
                    },
                    crate::ast::data_type::ValidEnumValue {
                        id: "3".to_string(),
                        name: Name("MARKET_START_TIME".to_string()),
                        description: vec![Comment::new("If no value is passed into the marketProjections parameter then the marketStartTime will not be returned with marketCatalogue".to_string())],
                    },
                ],
            }),
            description: vec![Comment::new("Type of price data returned by listMarketBook operation".to_string())],
        };

        // Execute
        let actual = GEN_V1.generate_data_type(&data_type);

        // Assert
        let expected = quote! {
            #[doc = "Type of price data returned by listMarketBook operation"]
            #[derive(Debug, Clone, PartialEq)]
            pub enum MarketProjection {
                #[doc = "If no value is passed into the marketProjections parameter then the competition will not be returned with marketCatalogue"]
                COMPETITION = 0i128,
                #[doc = "If no value is passed into the marketProjections parameter then the event will not be returned with marketCatalogue"]
                EVENT = 1i128,
                #[doc = "If no value is passed into the marketProjections parameter then the eventType will not be returned with marketCatalogue"]
                EVENT_TYPE = 2i128,
                #[doc = "If no value is passed into the marketProjections parameter then the marketStartTime will not be returned with marketCatalogue"]
                MARKET_START_TIME = 3i128,
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[rstest::rstest]
    fn test_generate_enum_2() {
        // Setup
        let data_type = DataType {
            name: Name("MarketProjection".to_string()),
            variant: crate::ast::data_type::DataTypeVariant::EnumValue(EnumValue {
                name: Name("MarketProjection".to_string()),
                valid_values: vec![
                    crate::ast::data_type::ValidEnumValue {
                        id: "".to_string(),
                        name: Name("COMPETITION".to_string()),
                        description: vec![Comment::new("If no value is passed into the marketProjections parameter then the competition will not be returned with marketCatalogue".to_string())],
                    },
                    crate::ast::data_type::ValidEnumValue {
                        id: "".to_string(),
                        name: Name("EVENT".to_string()),
                        description: vec![Comment::new("If no value is passed into the marketProjections parameter then the event will not be returned with marketCatalogue".to_string())],
                    },
                    crate::ast::data_type::ValidEnumValue {
                        id: "".to_string(),
                        name: Name("EVENT_TYPE".to_string()),
                        description: vec![Comment::new("If no value is passed into the marketProjections parameter then the eventType will not be returned with marketCatalogue".to_string())],
                    },
                    crate::ast::data_type::ValidEnumValue {
                        id: "".to_string(),
                        name: Name("MARKET_START_TIME".to_string()),
                        description: vec![Comment::new("If no value is passed into the marketProjections parameter then the marketStartTime will not be returned with marketCatalogue".to_string())],
                    },
                ],
            }),
            description: vec![Comment::new("Type of price data returned by listMarketBook operation".to_string())],
        };

        // Execute
        let actual = GEN_V1.generate_data_type(&data_type);

        // Assert
        let expected = quote! {
            #[doc = "Type of price data returned by listMarketBook operation"]
            #[derive(Debug, Clone, PartialEq)]
            pub enum MarketProjection {
                #[doc = "If no value is passed into the marketProjections parameter then the competition will not be returned with marketCatalogue"]
                COMPETITION,
                #[doc = "If no value is passed into the marketProjections parameter then the event will not be returned with marketCatalogue"]
                EVENT,
                #[doc = "If no value is passed into the marketProjections parameter then the eventType will not be returned with marketCatalogue"]
                EVENT_TYPE ,
                #[doc = "If no value is passed into the marketProjections parameter then the marketStartTime will not be returned with marketCatalogue"]
                MARKET_START_TIME,
            }
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[rstest::rstest]
    fn test_generate_type_alias() {
        // Setup
        let data_type = DataType {
            name: Name("MarketProjection".to_string()),
            variant: crate::ast::data_type::DataTypeVariant::TypeAlias(TypeAlias {
                name: Name("MarketProjection".to_string()),
                data_type: "String".to_string(),
            }),
            description: vec![Comment::new(
                "Type of price data returned by listMarketBook operation".to_string(),
            )],
        };

        // Execute
        let actual = GEN_V1.generate_data_type(&data_type);

        // Assert
        let expected = quote! {
            #[doc = "Type of price data returned by listMarketBook operation"]
            #[derive(Debug, Clone, PartialEq)]
            pub struct MarketProjection(pub String);
        };
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
