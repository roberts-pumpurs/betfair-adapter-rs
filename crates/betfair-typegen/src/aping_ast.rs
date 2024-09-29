use std::collections::HashMap;
pub(crate) mod data_type;
pub(crate) mod rpc_calls;
pub(crate) mod types;

use betfair_xml_parser::Interface;
use heck::ToPascalCase;
use typed_builder::TypedBuilder;

use self::data_type::{
    DataType, DataTypeVariant, EnumValue, StructField, StructValue, TypeAlias, ValidEnumValue,
};
use self::rpc_calls::{Exception, Param, Returns, RpcCall};
use self::types::{Comment, Name};

#[derive(Debug, Clone, TypedBuilder)]
pub(crate) struct Aping {
    #[builder(default)]
    name: Name,
    #[builder(default)]
    owner: Name,
    #[builder(default)]
    version: Name,
    #[builder(default)]
    date: Name,
    #[builder(default)]
    namespace: Name,
    #[builder(default)]
    top_level_docs: Vec<Comment>,
    #[builder(default)]
    rpc_calls: HashMap<Name, rpc_calls::RpcCall>,
    #[builder(default)]
    data_types: HashMap<Name, data_type::DataType>,
}

impl From<Interface> for Aping {
    fn from(val: Interface) -> Self {
        let aping_default = Self {
            name: Name(val.name),
            owner: Name(val.owner),
            version: Name(val.version),
            date: Name(val.date),
            namespace: Name(val.namespace),
            top_level_docs: vec![],
            rpc_calls: HashMap::new(),
            data_types: HashMap::new(),
        };
        let aping = val.items.iter().fold(aping_default, |mut aping, x| {
            match *x {
                betfair_xml_parser::InterfaceItems::Description(ref x) => {
                    aping.insert_top_level_docs(x);
                }
                betfair_xml_parser::InterfaceItems::SimpleType(ref x) => {
                    aping.insert_simple_data_type(x);
                }
                betfair_xml_parser::InterfaceItems::DataType(ref x) => aping.insert_data_type(x),
                betfair_xml_parser::InterfaceItems::ExceptionType(ref x) => {
                    aping.insert_exception_type(x);
                }
                betfair_xml_parser::InterfaceItems::Operation(ref x) => aping.insert_operation(x),
            };
            aping
        });

        aping
    }
}

impl Aping {
    pub(self) fn insert_top_level_docs(&mut self, desc: &betfair_xml_parser::common::Description) {
        if let Some(x) = desc.value.as_ref() {
            self.top_level_docs.push(Comment::new(x.to_string()));
        }
    }
    pub(self) fn insert_simple_data_type(
        &mut self,
        sdt: &betfair_xml_parser::simple_type::SimpleType,
    ) {
        // We either have a type alias or an enum
        let data_type_variant = sdt.valid_values.as_ref().map_or_else(
            || {
                // Parse as a type alias (this is the None case)
                DataTypeVariant::TypeAlias(TypeAlias {
                    name: Name(sdt.name.clone()),
                    data_type: sdt.r#type.clone().into(),
                })
            },
            |valid_values| {
                // Parse as an enum (this is the Some case)
                let valid_values = valid_values
                    .items
                    .iter()
                    .map(|x| {
                        let description = x
                            .description
                            .value
                            .clone()
                            .map(|data_type_description| vec![Comment::new(data_type_description)])
                            .unwrap_or_default();
                        ValidEnumValue {
                            id: x.id.clone().unwrap_or_else(|| x.name.clone()),
                            name: Name(x.name.clone()),
                            description,
                        }
                    })
                    .collect::<Vec<_>>();
                DataTypeVariant::EnumValue(EnumValue {
                    name: Name(sdt.name.clone()),
                    valid_values,
                })
            },
        );

        let data_type = data_type::DataType {
            name: Name(sdt.name.clone()),
            variant: data_type_variant,
            description: vec![],
        };
        self.data_types.insert(data_type.name.clone(), data_type);
    }

    pub(self) fn insert_data_type(&mut self, data_type: &betfair_xml_parser::data_type::DataType) {
        // Extract documentation from the data type
        let doc_comment = data_type
            .values
            .iter()
            .filter_map(|x| match *x {
                betfair_xml_parser::data_type::DataTypeItems::Description(ref x) => x.value.clone(),
                betfair_xml_parser::data_type::DataTypeItems::Parameter(_) => None,
            })
            .map(Comment::new)
            .collect::<Vec<_>>();
        let fields = data_type
            .values
            .iter()
            .filter_map(|x| match *x {
                betfair_xml_parser::data_type::DataTypeItems::Parameter(ref param) => {
                    Some(param.clone())
                }
                betfair_xml_parser::data_type::DataTypeItems::Description(_) => None,
            })
            .map(|x| {
                let description = x
                    .items
                    .iter()
                    .filter_map(|parameter_item| match *parameter_item {
                        betfair_xml_parser::common::ParameterItem::Description(ref desc) => {
                            desc.value.clone()
                        }
                        betfair_xml_parser::common::ParameterItem::ValidValues(_) => None,
                    })
                    .map(Comment::new)
                    .collect::<Vec<_>>();
                StructField {
                    name: Name(x.name.clone()),
                    mandatory: x.mandatory.unwrap_or(false),
                    data_type: x.r#type.into(),
                    description,
                }
            })
            .collect::<Vec<_>>();
        let data_type_variant = StructValue {
            name: Name(data_type.name.clone()),
            fields,
        };

        let data_type = data_type::DataType {
            name: Name(data_type.name.clone()),
            variant: DataTypeVariant::StructValue(data_type_variant),
            description: doc_comment,
        };
        self.data_types.insert(data_type.name.clone(), data_type);
    }

    pub(self) fn insert_operation(&mut self, operation: &betfair_xml_parser::operation::Operation) {
        // Extract documentation from the operation
        let doc_comment = operation.lense();

        // Extract the parameters from the operation
        let param_items = operation
            .values
            .iter()
            .filter_map(|x| match *x {
                betfair_xml_parser::operation::OperationItem::Parameters(ref x) => Some(x.clone()),
                betfair_xml_parser::operation::OperationItem::Description(_) => None,
            })
            .flat_map(|x| x.values)
            .collect::<Vec<_>>();

        let returns = (&param_items).lense();
        let exception = (&param_items).lense();
        let params = (&param_items).lense();
        let data_types: Vec<DataType> = (&param_items).lense();
        let rpc_call = RpcCall {
            name: Name(operation.name.clone()),
            params,
            description: doc_comment,
            exception,
            returns,
        };
        self.rpc_calls.insert(rpc_call.name.clone(), rpc_call);
        for x in data_types {
            self.data_types.insert(x.name.clone(), x);
        }
    }

    pub(self) fn insert_exception_type(
        &mut self,
        exception: &betfair_xml_parser::exception_type::ExceptionType,
    ) {
        // Retrieve the comments
        let doc_comment = exception
            .values
            .iter()
            .filter_map(|x| match *x {
                betfair_xml_parser::exception_type::ExceptionTypeItems::Description(ref x) => {
                    x.value.clone()
                }
                betfair_xml_parser::exception_type::ExceptionTypeItems::Parameter(_) => None,
            })
            .map(Comment::new)
            .collect::<Vec<_>>();

        let fields = exception
            .values
            .iter()
            .filter_map(|x| match *x {
                betfair_xml_parser::exception_type::ExceptionTypeItems::Parameter(ref x) => {
                    Some(x.clone())
                }
                betfair_xml_parser::exception_type::ExceptionTypeItems::Description(_) => None,
            })
            .map(|x| {
                let description = x
                    .items
                    .iter()
                    .filter_map(|parameter_item| match *parameter_item {
                        betfair_xml_parser::common::ParameterItem::Description(ref description) => {
                            description.value.clone()
                        }
                        betfair_xml_parser::common::ParameterItem::ValidValues(_) => None,
                    })
                    .map(Comment::new)
                    .collect::<Vec<_>>();
                let enum_values = x
                    .items
                    .iter()
                    .filter_map(|parameter_item| match *parameter_item {
                        betfair_xml_parser::common::ParameterItem::ValidValues(ref vv) => {
                            Some(vv.items.clone())
                        }
                        betfair_xml_parser::common::ParameterItem::Description(_) => None,
                    })
                    .flatten()
                    .map(|value| {
                        let desc = value
                            .description
                            .value
                            .iter()
                            .map(|item| Comment::new(item.clone()))
                            .collect();

                        ValidEnumValue {
                            id: value.id.map_or_else(
                                || format!("{}-0000", exception.prefix),
                                |id| format!("{}-{:0>4}", exception.prefix, id),
                            ),
                            name: Name(value.name),
                            description: desc,
                        }
                    })
                    .collect::<Vec<_>>();

                if enum_values.is_empty() {
                    StructField {
                        name: Name(x.name.clone()),
                        mandatory: x.mandatory.unwrap_or(false),
                        data_type: x.r#type.into(),
                        description,
                    }
                } else {
                    let enum_name = Name(x.name.to_pascal_case());
                    let error_enum = data_type::DataType {
                        name: enum_name.clone(),
                        variant: data_type::DataTypeVariant::EnumValue(EnumValue {
                            name: enum_name.clone(),
                            valid_values: enum_values,
                        }),
                        description: vec![],
                    };
                    self.data_types.insert(error_enum.name.clone(), error_enum);

                    StructField {
                        name: Name(x.name.clone()),
                        mandatory: x.mandatory.unwrap_or(false),
                        data_type: enum_name.0.into(),
                        description,
                    }
                }
            })
            .collect::<Vec<_>>();

        let wrapper_data_type = data_type::DataType {
            name: Name(exception.name.clone()),
            variant: data_type::DataTypeVariant::StructValue(StructValue {
                name: Name(exception.name.clone()),
                fields,
            }),
            description: doc_comment,
        };
        self.data_types
            .insert(wrapper_data_type.name.clone(), wrapper_data_type);
    }

    pub(crate) const fn name(&self) -> &Name {
        &self.name
    }

    pub(crate) const fn owner(&self) -> &Name {
        &self.owner
    }

    pub(crate) const fn version(&self) -> &Name {
        &self.version
    }

    pub(crate) const fn date(&self) -> &Name {
        &self.date
    }

    pub(crate) const fn namespace(&self) -> &Name {
        &self.namespace
    }

    pub(crate) fn top_level_docs(&self) -> &[Comment] {
        self.top_level_docs.as_ref()
    }

    pub(crate) const fn rpc_calls(&self) -> &HashMap<Name, rpc_calls::RpcCall> {
        &self.rpc_calls
    }

    pub(crate) const fn data_types(&self) -> &HashMap<Name, data_type::DataType> {
        &self.data_types
    }
}

trait Prism<T> {
    fn lense(&self) -> T;
}
mod prism_impls {
    use super::*;
    impl Prism<Returns> for &Vec<betfair_xml_parser::operation::ParametersItems> {
        fn lense(&self) -> Returns {
            let returns = self
                .iter()
                .filter_map(|x| match *x {
                    betfair_xml_parser::operation::ParametersItems::SimpleResponse(ref x) => {
                        Some(x.clone())
                    }
                    betfair_xml_parser::operation::ParametersItems::Request(_) |
                    betfair_xml_parser::operation::ParametersItems::Exceptions(_) => None,
                })
                .map(|x| Returns {
                    data_type: x.r#type.clone().into(),
                    description: x.description.value.map(Comment::new).into_iter().collect(),
                })
                .collect::<Vec<_>>();
            assert!(
                returns.len() <= 1,
                "More than one returns found for operation!"
            );

            returns.into_iter().next().map_or_else(
                || Returns {
                    data_type: "void".to_owned().into(),
                    description: vec![],
                },
                |x| x,
            )
        }
    }

    impl Prism<Option<Exception>> for &Vec<betfair_xml_parser::operation::ParametersItems> {
        fn lense(&self) -> Option<Exception> {
            let exceptions = self
                .iter()
                .filter_map(|x| match *x {
                    betfair_xml_parser::operation::ParametersItems::Exceptions(ref x) => {
                        Some(x.clone())
                    }
                    betfair_xml_parser::operation::ParametersItems::Request(_) |
                    betfair_xml_parser::operation::ParametersItems::SimpleResponse(_) => None,
                })
                .flat_map(|x| x.values)
                .map(|x| Exception {
                    data_type: x.r#type.clone().into(),
                    description: x.description.value.map(Comment::new).into_iter().collect(),
                })
                .collect::<Vec<_>>();
            assert!(
                exceptions.len() <= 1,
                "More than one exception found for operation"
            );

            exceptions.into_iter().next()
        }
    }

    impl Prism<Vec<Comment>> for betfair_xml_parser::operation::Operation {
        fn lense(&self) -> Vec<Comment> {
            let doc_comment = self
                .values
                .iter()
                .filter_map(|x| match *x {
                    betfair_xml_parser::operation::OperationItem::Description(ref x) => {
                        x.value.clone()
                    }
                    betfair_xml_parser::operation::OperationItem::Parameters(_) => None,
                })
                .map(Comment::new)
                .collect();
            doc_comment
        }
    }

    impl Prism<Vec<Param>> for &Vec<betfair_xml_parser::operation::ParametersItems> {
        fn lense(&self) -> Vec<Param> {
            self.iter()
                .filter_map(|x| match *x {
                    betfair_xml_parser::operation::ParametersItems::Request(ref x) => {
                        Some(x.clone())
                    }
                    betfair_xml_parser::operation::ParametersItems::SimpleResponse(_) |
                    betfair_xml_parser::operation::ParametersItems::Exceptions(_) => None,
                })
                .flat_map(|x| x.values.unwrap_or_default())
                .map(|x| {
                    let doc_comments = (&x).lense();

                    Param {
                        name: Name(x.name.clone()),
                        data_type: x.r#type.clone().into(),
                        mandatory: x.mandatory.unwrap_or(false),
                        description: doc_comments,
                    }
                })
                .collect()
        }
    }

    impl Prism<Vec<Comment>> for &betfair_xml_parser::common::Parameter {
        fn lense(&self) -> Vec<Comment> {
            let doc_comments = self
                .items
                .iter()
                .filter_map(|x| match *x {
                    betfair_xml_parser::common::ParameterItem::Description(ref x) => {
                        x.value.clone()
                    }
                    betfair_xml_parser::common::ParameterItem::ValidValues(_) => None,
                })
                .map(Comment::new)
                .collect::<Vec<_>>();
            doc_comments
        }
    }

    impl Prism<Vec<DataType>> for &Vec<betfair_xml_parser::operation::ParametersItems> {
        fn lense(&self) -> Vec<DataType> {
            self.iter()
                .filter_map(|x| match *x {
                    betfair_xml_parser::operation::ParametersItems::Request(ref x) => {
                        Some(x.clone())
                    }
                    betfair_xml_parser::operation::ParametersItems::SimpleResponse(_) |
                    betfair_xml_parser::operation::ParametersItems::Exceptions(_) => None,
                })
                .flat_map(|x| x.values.unwrap_or_default())
                .filter_map(|x| {
                    let doc_comment = (&x).lense();

                    let param_specific_enum = x
                        .items
                        .iter()
                        .filter_map(|param| match *param {
                            betfair_xml_parser::common::ParameterItem::ValidValues(ref param) => {
                                Some(param.items.clone())
                            }
                            betfair_xml_parser::common::ParameterItem::Description(_) => None,
                        })
                        .flatten()
                        .fold(
                            data_type::EnumValue {
                                name: Name(x.name.clone()),
                                valid_values: vec![],
                            },
                            |mut acc, i| {
                                let description = i
                                    .description
                                    .value
                                    .as_ref()
                                    .map(|desc| vec![Comment::new(desc.clone())])
                                    .unwrap_or_default();

                                let value = data_type::ValidEnumValue {
                                    id: i.id.unwrap_or_else(|| i.name.clone()),
                                    name: Name(i.name),
                                    description,
                                };
                                acc.valid_values.push(value);
                                acc
                            },
                        );

                    if param_specific_enum.valid_values.is_empty() {
                        None
                    } else {
                        let data_type = data_type::DataType {
                            name: param_specific_enum.name.clone(),
                            variant: data_type::DataTypeVariant::EnumValue(param_specific_enum),
                            description: doc_comment,
                        };
                        Some(data_type)
                    }
                })
                .collect()
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
#[expect(clippy::indexing_slicing)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[rstest::fixture]
    fn aping() -> Aping {
        Aping {
            name: Name("SportsAPING".to_owned()),
            owner: Name("BDP".to_owned()),
            version: Name("1.0.0".to_owned()),
            date: Name("now()".to_owned()),
            namespace: Name("com.betfair.sports.api".to_owned()),
            top_level_docs: vec![],
            rpc_calls: HashMap::new(),
            data_types: HashMap::new(),
        }
    }

    #[rstest::rstest]
    fn parse_top_level_docs(mut aping: Aping) {
        // Setup
        let input = "
        <description>Account API-NG</description>
        ";
        let input = serde_xml_rs::from_str(input).unwrap();
        let expected = vec![Comment::new("Account API-NG".to_owned())];

        // Action
        aping.insert_top_level_docs(&input);

        // Assert
        assert_eq!(aping.top_level_docs, expected);
    }

    #[rstest::rstest]
    fn parse_simple_type_enum(mut aping: Aping) {
        // Setup
        let input = r#"
    <simpleType name="SubscriptionStatus" type="string">
        <validValues>
            <value name="ALL">
                <description>Any subscription status</description>
            </value>
            <value name="ACTIVATED">
                <description>Only activated subscriptions</description>
            </value>
            <value name="UNACTIVATED">
                <description>Only unactivated subscriptions</description>
            </value>
            <value name="CANCELLED">
                <description>Only cancelled subscriptions</description>
            </value>
            <value name="EXPIRED">
                <description>Only expired subscriptions</description>
            </value>
        </validValues>
    </simpleType>
        "#;
        let input = serde_xml_rs::from_str(input).unwrap();
        let mut expected = HashMap::new();
        expected.insert(
            Name("SubscriptionStatus".to_owned()),
            DataType {
                name: Name("SubscriptionStatus".to_owned()),
                variant: DataTypeVariant::EnumValue(EnumValue {
                    name: Name("SubscriptionStatus".to_owned()),
                    valid_values: vec![
                        ValidEnumValue {
                            id: "ALL".to_owned(),
                            name: Name("ALL".to_owned()),
                            description: vec![Comment::new("Any subscription status".to_owned())],
                        },
                        ValidEnumValue {
                            id: "ACTIVATED".to_owned(),
                            name: Name("ACTIVATED".to_owned()),
                            description: vec![Comment::new(
                                "Only activated subscriptions".to_owned(),
                            )],
                        },
                        ValidEnumValue {
                            id: "UNACTIVATED".to_owned(),
                            name: Name("UNACTIVATED".to_owned()),
                            description: vec![Comment::new(
                                "Only unactivated subscriptions".to_owned(),
                            )],
                        },
                        ValidEnumValue {
                            id: "CANCELLED".to_owned(),
                            name: Name("CANCELLED".to_owned()),
                            description: vec![Comment::new(
                                "Only cancelled subscriptions".to_owned(),
                            )],
                        },
                        ValidEnumValue {
                            id: "EXPIRED".to_owned(),
                            name: Name("EXPIRED".to_owned()),
                            description: vec![Comment::new(
                                "Only expired subscriptions".to_owned(),
                            )],
                        },
                    ],
                }),
                description: vec![],
            },
        );

        // Action
        aping.insert_simple_data_type(&input);

        // Assert
        assert_eq!(aping.data_types, expected);
    }

    #[rstest::rstest]
    fn parse_type_alias(mut aping: Aping) {
        // Setup
        let input = r#"
<simpleType name="MarketType" type="string"/>
        "#;
        let input = serde_xml_rs::from_str(input).unwrap();
        let mut expected = HashMap::new();
        expected.insert(
            Name("MarketType".to_owned()),
            DataType {
                name: Name("MarketType".to_owned()),
                variant: DataTypeVariant::TypeAlias(TypeAlias {
                    name: Name("MarketType".to_owned()),
                    data_type: "string".to_owned().into(),
                }),
                description: vec![],
            },
        );

        // Action
        aping.insert_simple_data_type(&input);

        // Assert
        assert_eq!(aping.data_types, expected);
    }

    #[rstest::rstest]
    fn parse_exception_type(mut aping: Aping) {
        // Setup
        let input = r#"
    <exceptionType name="AccountAPINGException" prefix="AANGX">
        <description>This exception is thrown when an operation fails</description>
        <parameter name="errorCode" type="string">
            <description>the unique code for this error</description>
            <validValues>
                <value id="1" name="INVALID_INPUT_DATA">
                    <description>Invalid input data</description>
                </value>
                <value id="2" name="INVALID_SESSION_INFORMATION">
                    <description>The session token passed is invalid or expired</description>
                </value>
                <value id="13" name="UNEXPECTED_ERROR">
                    <description>An unexpected internal error occurred that prevented successful request processing.
                    </description>
                </value>
            </validValues>
        </parameter>
        <parameter name="errorDetails" type="string">
            <description>the stack trace of the error</description>
        </parameter>
        <parameter name="requestUUID" type="string">
            <description/>
        </parameter>
    </exceptionType>"#;
        let input = serde_xml_rs::from_str(input).unwrap();
        let mut expected = HashMap::new();
        let error_code_enum = EnumValue {
            name: Name("ErrorCode".to_owned()),
            valid_values: vec![
                ValidEnumValue {
                    id: "AANGX-0001".to_owned(),
                    name: Name("INVALID_INPUT_DATA".to_owned()),
                    description: vec![Comment::new("Invalid input data".to_owned(),
                )],
                },
                ValidEnumValue {
                    id: "AANGX-0002".to_owned(),
                    name: Name("INVALID_SESSION_INFORMATION".to_owned()),
                    description: vec![Comment::new("The session token passed is invalid or expired".to_owned(),
                )],
                },
                ValidEnumValue {
                    id: "AANGX-0013".to_owned(),
                    name: Name("UNEXPECTED_ERROR".to_owned()),
                    description: vec![Comment::new("An unexpected internal error occurred that prevented successful request processing.".to_owned(),
                )],
                },
            ],
        };
        let struct_value = StructValue {
            name: Name("AccountAPINGException".to_owned()),
            fields: vec![
                StructField {
                    name: Name("errorCode".to_owned()),
                    data_type: "ErrorCode".to_owned().into(),
                    description: vec![Comment::new("the unique code for this error".to_owned())],
                    mandatory: false,
                },
                StructField {
                    name: Name("errorDetails".to_owned()),
                    data_type: "string".to_owned().into(),
                    description: vec![Comment::new("the stack trace of the error".to_owned())],
                    mandatory: false,
                },
                StructField {
                    name: Name("requestUUID".to_owned()),
                    data_type: "string".to_owned().into(),
                    description: vec![],
                    mandatory: false,
                },
            ],
        };
        expected.insert(
            Name("AccountAPINGException".to_owned()),
            DataType {
                description: vec![Comment::new(
                    "This exception is thrown when an operation fails".to_owned(),
                )],
                name: Name("AccountAPINGException".to_owned()),
                variant: DataTypeVariant::StructValue(struct_value),
            },
        );
        expected.insert(
            Name("ErrorCode".to_owned()),
            DataType {
                description: vec![],
                name: Name("ErrorCode".to_owned()),
                variant: DataTypeVariant::EnumValue(error_code_enum),
            },
        );

        // Action
        aping.insert_exception_type(&input);

        // Assert
        assert_eq!(aping.data_types, expected);
    }

    #[rstest::rstest]
    fn parse_data_type(mut aping: Aping) {
        // Setup
        let input = r#"
    <dataType name="AccountSubscription">
        <description>
            Application subscription details
        </description>
        <parameter name="subscriptionTokens" type="list(SubscriptionTokenInfo)" mandatory="true">
            <description>List of subscription token details</description>
        </parameter>
        <parameter name="applicationName" type="string" mandatory="false">
            <description>Application name</description>
        </parameter>
        <parameter name="applicationVersionId" type="string" mandatory="false">
            <description>Application version Id</description>
        </parameter>
    </dataType>"#;
        let input = serde_xml_rs::from_str(input).unwrap();
        let mut expected = HashMap::new();
        let struct_value = StructValue {
            name: Name("AccountSubscription".to_owned()),
            fields: vec![
                StructField {
                    name: Name("subscriptionTokens".to_owned()),
                    data_type: "list(SubscriptionTokenInfo)".to_owned().into(),
                    description: vec![Comment::new(
                        "List of subscription token details".to_owned(),
                    )],
                    mandatory: true,
                },
                StructField {
                    name: Name("applicationName".to_owned()),
                    data_type: "string".to_owned().into(),
                    description: vec![Comment::new("Application name".to_owned())],
                    mandatory: false,
                },
                StructField {
                    name: Name("applicationVersionId".to_owned()),
                    data_type: "string".to_owned().into(),
                    description: vec![Comment::new("Application version Id".to_owned())],
                    mandatory: false,
                },
            ],
        };
        expected.insert(
            Name("AccountSubscription".to_owned()),
            DataType {
                description: vec![Comment::new("Application subscription details".to_owned())],
                name: Name("AccountSubscription".to_owned()),
                variant: DataTypeVariant::StructValue(struct_value),
            },
        );

        // Action
        aping.insert_data_type(&input);

        // Assert
        assert_eq!(aping.data_types, expected);
    }

    #[rstest::rstest]
    fn parse_data_type_2(mut aping: Aping) {
        // Setup
        let input = r#"
    <dataType name="CancelInstruction">
        <description>Instruction to fully or partially cancel an order (only applies to LIMIT orders)</description>
        <parameter mandatory="true" name="betId" type="string">
            <description/>
        </parameter>
        <parameter name="sizeReduction" type="Size">
            <description>If supplied then this is a partial cancel</description>
        </parameter>
    </dataType>"#;
        let input = serde_xml_rs::from_str(input).unwrap();
        let mut expected = HashMap::new();
        let struct_value = StructValue {
            name: Name("CancelInstruction".to_owned()),
            fields: vec![
                StructField {
                    name: Name("betId".to_owned()),
                    data_type: "string".to_owned().into(),
                    description: vec![],
                    mandatory: true,
                },
                StructField {
                    name: Name("sizeReduction".to_owned()),
                    data_type: "Size".to_owned().into(),
                    description: vec![Comment::new(
                        "If supplied then this is a partial cancel".to_owned(),
                    )],
                    mandatory: false,
                },
            ],
        };
        expected.insert(
            Name("CancelInstruction".to_owned()),
            DataType {
                description: vec![Comment::new(
                    "Instruction to fully or partially cancel an order (only applies to LIMIT orders)".to_owned(),
                )],
                name: Name("CancelInstruction".to_owned()),
                variant: DataTypeVariant::StructValue(struct_value),
            },
        );

        // Action
        aping.insert_data_type(&input);

        // Assert
        assert_eq!(aping.data_types, expected);
    }

    #[rstest::rstest]
    fn parse_operation(mut aping: Aping) {
        // Setup
        let input = r#"
    <operation name="createDeveloperAppKeys" since="1.0.0">
        <description>
            Create 2 application keys for given user; one active and the other delayed
        </description>
        <parameters>
            <request>
                <parameter mandatory="true" name="appName" type="string">
                    <description>
                        A Display name for the application.
                    </description>
                </parameter>
            </request>
            <simpleResponse type="DeveloperApp">
                <description>
                    A map of application keys, one marked ACTIVE, and the other DELAYED
                </description>
            </simpleResponse>
            <exceptions>
                <exception type="AccountAPINGException">
                    <description>Generic exception that is thrown if this operation fails for any reason.</description>
                </exception>
            </exceptions>
        </parameters>
    </operation>
    "#;
        let input = serde_xml_rs::from_str(input).unwrap();
        let mut expected_rpc_calls = HashMap::new();
        let operation = RpcCall {
            name: Name("createDeveloperAppKeys".to_owned()),
            params: vec![Param {
                name: Name("appName".to_owned()),
                data_type: "string".to_owned().into(),
                description: vec![Comment::new(
                    "A Display name for the application.".to_owned(),
                )],
                mandatory: true,
            }],
            returns: Returns {
                data_type: "DeveloperApp".to_owned().into(),
                description: vec![Comment::new(
                    "A map of application keys, one marked ACTIVE, and the other DELAYED"
                        .to_owned(),
                )],
            },
            exception: Some(Exception {
                data_type: "AccountAPINGException".to_owned().into(),
                description: vec![Comment::new(
                    "Generic exception that is thrown if this operation fails for any reason."
                        .to_owned(),
                )],
            }),
            description: vec![Comment::new(
                "Create 2 application keys for given user; one active and the other delayed"
                    .to_owned(),
            )],
        };
        expected_rpc_calls.insert(Name("createDeveloperAppKeys".to_owned()), operation);

        // Action
        aping.insert_operation(&input);

        // Assert
        assert_eq!(aping.rpc_calls, expected_rpc_calls);
    }

    #[rstest::rstest]
    fn parse_operation_2(mut aping: Aping) {
        // Setup
        let input = r#"
    <operation name="getDeveloperAppKeys" since="1.0.0">
        <description>
            Get all application keys owned by the given developer/vendor
        </description>
        <parameters>
            <request/>
            <simpleResponse type="list(DeveloperApp)">
                <description>
                    A list of application keys owned by the given developer/vendor
                </description>
            </simpleResponse>
            <exceptions>
                <exception type="AccountAPINGException">
                    <description>Generic exception that is thrown if this operation fails for any reason.</description>
                </exception>
            </exceptions>
        </parameters>
    </operation>
    "#;
        let input = serde_xml_rs::from_str(input).unwrap();
        let mut expected_rpc_calls = HashMap::new();
        let operation = RpcCall {
            name: Name("getDeveloperAppKeys".to_owned()),
            params: vec![],
            returns: Returns {
                data_type: "list(DeveloperApp)".to_owned().into(),
                description: vec![Comment::new(
                    "A list of application keys owned by the given developer/vendor".to_owned(),
                )],
            },
            exception: Some(Exception {
                data_type: "AccountAPINGException".to_owned().into(),
                description: vec![Comment::new(
                    "Generic exception that is thrown if this operation fails for any reason."
                        .to_owned(),
                )],
            }),
            description: vec![Comment::new(
                "Get all application keys owned by the given developer/vendor".to_owned(),
            )],
        };
        expected_rpc_calls.insert(Name("getDeveloperAppKeys".to_owned()), operation);

        // Action
        aping.insert_operation(&input);

        // Assert
        assert_eq!(aping.rpc_calls, expected_rpc_calls);
    }
}
