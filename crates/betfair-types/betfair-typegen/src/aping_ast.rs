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
        let aping = Aping {
            name: Name(val.name),
            owner: Name(val.owner),
            version: Name(val.version),
            date: Name(val.date),
            namespace: Name(val.namespace),
            top_level_docs: vec![],
            rpc_calls: HashMap::new(),
            data_types: HashMap::new(),
        };
        let aping = val.items.iter().fold(aping, |mut aping, x| {
            match x {
                betfair_xml_parser::InterfaceItems::Description(x) => {
                    aping.insert_top_level_docs(x)
                }
                betfair_xml_parser::InterfaceItems::SimpleType(x) => {
                    aping.insert_simple_data_type(x)
                }
                betfair_xml_parser::InterfaceItems::DataType(x) => aping.insert_data_type(x),
                betfair_xml_parser::InterfaceItems::ExceptionType(x) => {
                    aping.insert_exception_type(x)
                }
                betfair_xml_parser::InterfaceItems::Operation(x) => aping.insert_operation(x),
            };
            aping
        });

        aping
    }
}

impl Aping {
    pub(self) fn insert_top_level_docs(&mut self, desc: &betfair_xml_parser::common::Description) {
        if let Some(x) = &desc.value {
            self.top_level_docs.push(Comment::new(x.clone()));
        }
    }
    pub(self) fn insert_simple_data_type(
        &mut self,
        sdt: &betfair_xml_parser::simple_type::SimpleType,
    ) {
        // We either have a type alias or an enum
        let data_type_variant = if let Some(valid_values) = &sdt.valid_values {
            // Parse as an enum
            let valid_values = valid_values
                .items
                .iter()
                .map(|x| {
                    let description = x
                        .description
                        .value
                        .clone()
                        .map(|x| vec![Comment::new(x)])
                        .unwrap_or_default();
                    ValidEnumValue {
                        id: x.id.clone().unwrap_or_else(|| x.name.clone()),
                        name: Name(x.name.clone()),
                        description,
                    }
                })
                .collect::<Vec<_>>();
            DataTypeVariant::EnumValue(EnumValue { name: Name(sdt.name.clone()), valid_values })
        } else {
            // Parse as a type alias
            DataTypeVariant::TypeAlias(TypeAlias {
                name: Name(sdt.name.clone()),
                data_type: sdt.r#type.clone().into(),
            })
        };

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
            .filter_map(|x| match x {
                betfair_xml_parser::data_type::DataTypeItems::Description(x) => x.value.clone(),
                _ => None,
            })
            .map(Comment::new)
            .collect::<Vec<_>>();
        let fields = data_type
            .values
            .iter()
            .filter_map(|x| match x {
                betfair_xml_parser::data_type::DataTypeItems::Parameter(param) => {
                    Some(param.clone())
                }
                _ => None,
            })
            .map(|x| {
                let description = x
                    .items
                    .iter()
                    .filter_map(|x| match x {
                        betfair_xml_parser::common::ParameterItem::Description(x) => {
                            x.value.clone()
                        }
                        _ => None,
                    })
                    .map(Comment::new)
                    .collect::<Vec<_>>();
                StructField {
                    name: Name(x.name.clone()),
                    mandatory: x.mandatory.unwrap_or(true),
                    data_type: x.r#type.into(),
                    description,
                }
            })
            .collect::<Vec<_>>();
        let data_type_variant = StructValue { name: Name(data_type.name.clone()), fields };

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
            .filter_map(|x| match x {
                betfair_xml_parser::operation::OperationItem::Parameters(x) => Some(x.clone()),
                _ => None,
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
            .filter_map(|x| match x {
                betfair_xml_parser::exception_type::ExceptionTypeItems::Description(x) => {
                    x.value.clone()
                }
                _ => None,
            })
            .map(Comment::new)
            .collect::<Vec<_>>();

        let fields = exception
            .values
            .iter()
            .filter_map(|x| match x {
                betfair_xml_parser::exception_type::ExceptionTypeItems::Parameter(x) => {
                    Some(x.clone())
                }
                _ => None,
            })
            .map(|x| {
                let description = x
                    .items
                    .iter()
                    .filter_map(|p| match p {
                        betfair_xml_parser::common::ParameterItem::Description(x) => {
                            x.value.clone()
                        }
                        _ => None,
                    })
                    .map(Comment::new)
                    .collect::<Vec<_>>();
                let enum_values = x
                    .items
                    .iter()
                    .filter_map(|p| match p {
                        betfair_xml_parser::common::ParameterItem::ValidValues(vv) => {
                            Some(vv.items.clone())
                        }
                        _ => None,
                    })
                    .flatten()
                    .map(|v| {
                        let description =
                            v.description.value.iter().map(|x| Comment::new(x.clone())).collect();

                        ValidEnumValue {
                            id: format!(
                                "{}-{:0>4}",
                                exception.prefix,
                                v.id.expect("Exception values require IDs")
                            ),
                            name: Name(v.name),
                            description,
                        }
                    })
                    .collect::<Vec<_>>();

                if !enum_values.is_empty() {
                    // create a new enum value type
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
                } else {
                    StructField {
                        name: Name(x.name.clone()),
                        mandatory: x.mandatory.unwrap_or(false),
                        data_type: x.r#type.into(),
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
        self.data_types.insert(wrapper_data_type.name.clone(), wrapper_data_type);
    }

    pub(crate) fn name(&self) -> &Name {
        &self.name
    }

    pub(crate) fn owner(&self) -> &Name {
        &self.owner
    }

    pub(crate) fn version(&self) -> &Name {
        &self.version
    }

    pub(crate) fn date(&self) -> &Name {
        &self.date
    }

    pub(crate) fn namespace(&self) -> &Name {
        &self.namespace
    }

    pub(crate) fn top_level_docs(&self) -> &[Comment] {
        self.top_level_docs.as_ref()
    }

    pub(crate) fn rpc_calls(&self) -> &HashMap<Name, rpc_calls::RpcCall> {
        &self.rpc_calls
    }

    pub(crate) fn data_types(&self) -> &HashMap<Name, data_type::DataType> {
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
                .filter_map(|x| match x {
                    betfair_xml_parser::operation::ParametersItems::SimpleResponse(x) => {
                        Some(x.clone())
                    }
                    _ => None,
                })
                .map(|x| Returns {
                    data_type: x.r#type.clone().into(),
                    description: x.description.value.map(Comment::new).into_iter().collect(),
                })
                .collect::<Vec<_>>();
            if returns.len() > 1 {
                panic!("More than one returns found for operation!");
            }

            returns.into_iter().next().expect("No return info found")
        }
    }

    impl Prism<Option<Exception>> for &Vec<betfair_xml_parser::operation::ParametersItems> {
        fn lense(&self) -> Option<Exception> {
            let exceptions = self
                .iter()
                .filter_map(|x| match x {
                    betfair_xml_parser::operation::ParametersItems::Exceptions(x) => {
                        Some(x.clone())
                    }
                    _ => None,
                })
                .flat_map(|x| x.values)
                .map(|x| Exception {
                    data_type: x.r#type.clone().into(),
                    description: x.description.value.map(Comment::new).into_iter().collect(),
                })
                .collect::<Vec<_>>();
            if exceptions.len() > 1 {
                panic!("More than one exception found for operation");
            }

            exceptions.into_iter().next()
        }
    }

    impl Prism<Vec<Comment>> for betfair_xml_parser::operation::Operation {
        fn lense(&self) -> Vec<Comment> {
            let doc_comment = self
                .values
                .iter()
                .filter_map(|x| match x {
                    betfair_xml_parser::operation::OperationItem::Description(x) => x.value.clone(),
                    _ => None,
                })
                .map(Comment::new)
                .collect();
            doc_comment
        }
    }

    impl Prism<Vec<Param>> for &Vec<betfair_xml_parser::operation::ParametersItems> {
        fn lense(&self) -> Vec<Param> {
            self.iter()
                .filter_map(|x| match x {
                    betfair_xml_parser::operation::ParametersItems::Request(x) => Some(x.clone()),
                    _ => None,
                })
                .flat_map(|x| x.values.unwrap_or_default())
                .map(|x| {
                    let doc_comments = (&x).lense();

                    Param {
                        name: Name(x.name.clone()),
                        data_type: x.r#type.clone().into(),
                        mandatory: x.mandatory.unwrap_or(true),
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
                .filter_map(|x| match x {
                    betfair_xml_parser::common::ParameterItem::Description(x) => x.value.clone(),
                    _ => None,
                })
                .map(Comment::new)
                .collect::<Vec<_>>();
            doc_comments
        }
    }

    impl Prism<Vec<DataType>> for &Vec<betfair_xml_parser::operation::ParametersItems> {
        fn lense(&self) -> Vec<DataType> {
            self.iter()
                .filter_map(|x| match x {
                    betfair_xml_parser::operation::ParametersItems::Request(x) => Some(x.clone()),
                    _ => None,
                })
                .flat_map(|x| x.values.unwrap_or_default())
                .filter_map(|x| {
                    let doc_comment = (&x).lense();

                    let param_specific_enum = x
                        .items
                        .iter()
                        .filter_map(|x| match x {
                            betfair_xml_parser::common::ParameterItem::ValidValues(x) => {
                                Some(x.items.clone())
                            }
                            _ => None,
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
                                    .map(|x| vec![Comment::new(x.clone())])
                                    .unwrap_or_default();

                                let value = data_type::ValidEnumValue {
                                    id: i.id.unwrap_or(i.name.clone()),
                                    name: Name(i.name),
                                    description,
                                };
                                acc.valid_values.push(value);
                                acc
                            },
                        );

                    if !param_specific_enum.valid_values.is_empty() {
                        let data_type = data_type::DataType {
                            name: param_specific_enum.name.clone(),
                            variant: data_type::DataTypeVariant::EnumValue(param_specific_enum),
                            description: doc_comment,
                        };
                        Some(data_type)
                    } else {
                        None
                    }
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[rstest::fixture]
    fn aping() -> Aping {
        Aping {
            name: Name("SportsAPING".to_string()),
            owner: Name("BDP".to_string()),
            version: Name("1.0.0".to_string()),
            date: Name("now()".to_string()),
            namespace: Name("com.betfair.sports.api".to_string()),
            top_level_docs: vec![],
            rpc_calls: HashMap::new(),
            data_types: HashMap::new(),
        }
    }

    #[rstest::rstest]
    fn parse_top_level_docs(mut aping: Aping) {
        // Setup
        let input = r#"
        <description>Account API-NG</description>
        "#;
        let input = serde_xml_rs::from_str(input).unwrap();
        let expected = vec![Comment::new("Account API-NG".to_string())];

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
            Name("SubscriptionStatus".to_string()),
            DataType {
                name: Name("SubscriptionStatus".to_string()),
                variant: DataTypeVariant::EnumValue(EnumValue {
                    name: Name("SubscriptionStatus".to_string()),
                    valid_values: vec![
                        ValidEnumValue {
                            id: "ALL".to_string(),
                            name: Name("ALL".to_string()),
                            description: vec![Comment::new("Any subscription status".to_string())],
                        },
                        ValidEnumValue {
                            id: "ACTIVATED".to_string(),
                            name: Name("ACTIVATED".to_string()),
                            description: vec![Comment::new(
                                "Only activated subscriptions".to_string(),
                            )],
                        },
                        ValidEnumValue {
                            id: "UNACTIVATED".to_string(),
                            name: Name("UNACTIVATED".to_string()),
                            description: vec![Comment::new(
                                "Only unactivated subscriptions".to_string(),
                            )],
                        },
                        ValidEnumValue {
                            id: "CANCELLED".to_string(),
                            name: Name("CANCELLED".to_string()),
                            description: vec![Comment::new(
                                "Only cancelled subscriptions".to_string(),
                            )],
                        },
                        ValidEnumValue {
                            id: "EXPIRED".to_string(),
                            name: Name("EXPIRED".to_string()),
                            description: vec![Comment::new(
                                "Only expired subscriptions".to_string(),
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
            Name("MarketType".to_string()),
            DataType {
                name: Name("MarketType".to_string()),
                variant: DataTypeVariant::TypeAlias(TypeAlias {
                    name: Name("MarketType".to_string()),
                    data_type: "string".to_string().into(),
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
            name: Name("ErrorCode".to_string()),
            valid_values: vec![
                ValidEnumValue {
                    id: "AANGX-0001".to_string(),
                    name: Name("INVALID_INPUT_DATA".to_string()),
                    description: vec![Comment::new("Invalid input data".to_string(),
                )],
                },
                ValidEnumValue {
                    id: "AANGX-0002".to_string(),
                    name: Name("INVALID_SESSION_INFORMATION".to_string()),
                    description: vec![Comment::new("The session token passed is invalid or expired".to_string(),
                )],
                },
                ValidEnumValue {
                    id: "AANGX-0013".to_string(),
                    name: Name("UNEXPECTED_ERROR".to_string()),
                    description: vec![Comment::new("An unexpected internal error occurred that prevented successful request processing.".to_string(),
                )],
                },
            ],
        };
        let struct_value = StructValue {
            name: Name("AccountAPINGException".to_string()),
            fields: vec![
                StructField {
                    name: Name("errorCode".to_string()),
                    data_type: "ErrorCode".to_string().into(),
                    description: vec![Comment::new("the unique code for this error".to_string())],
                    mandatory: false,
                },
                StructField {
                    name: Name("errorDetails".to_string()),
                    data_type: "string".to_string().into(),
                    description: vec![Comment::new("the stack trace of the error".to_string())],
                    mandatory: false,
                },
                StructField {
                    name: Name("requestUUID".to_string()),
                    data_type: "string".to_string().into(),
                    description: vec![],
                    mandatory: false,
                },
            ],
        };
        expected.insert(
            Name("AccountAPINGException".to_string()),
            DataType {
                description: vec![Comment::new(
                    "This exception is thrown when an operation fails".to_string(),
                )],
                name: Name("AccountAPINGException".to_string()),
                variant: DataTypeVariant::StructValue(struct_value),
            },
        );
        expected.insert(
            Name("ErrorCode".to_string()),
            DataType {
                description: vec![],
                name: Name("ErrorCode".to_string()),
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
            name: Name("AccountSubscription".to_string()),
            fields: vec![
                StructField {
                    name: Name("subscriptionTokens".to_string()),
                    data_type: "list(SubscriptionTokenInfo)".to_string().into(),
                    description: vec![Comment::new(
                        "List of subscription token details".to_string(),
                    )],
                    mandatory: true,
                },
                StructField {
                    name: Name("applicationName".to_string()),
                    data_type: "string".to_string().into(),
                    description: vec![Comment::new("Application name".to_string())],
                    mandatory: false,
                },
                StructField {
                    name: Name("applicationVersionId".to_string()),
                    data_type: "string".to_string().into(),
                    description: vec![Comment::new("Application version Id".to_string())],
                    mandatory: false,
                },
            ],
        };
        expected.insert(
            Name("AccountSubscription".to_string()),
            DataType {
                description: vec![Comment::new("Application subscription details".to_string())],
                name: Name("AccountSubscription".to_string()),
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
            name: Name("createDeveloperAppKeys".to_string()),
            params: vec![Param {
                name: Name("appName".to_string()),
                data_type: "string".to_string().into(),
                description: vec![Comment::new("A Display name for the application.".to_string())],
                mandatory: true,
            }],
            returns: Returns {
                data_type: "DeveloperApp".to_string().into(),
                description: vec![Comment::new(
                    "A map of application keys, one marked ACTIVE, and the other DELAYED"
                        .to_string(),
                )],
            },
            exception: Some(Exception {
                data_type: "AccountAPINGException".to_string().into(),
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
        expected_rpc_calls.insert(Name("createDeveloperAppKeys".to_string()), operation);

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
            name: Name("getDeveloperAppKeys".to_string()),
            params: vec![],
            returns: Returns {
                data_type: "list(DeveloperApp)".to_string().into(),
                description: vec![Comment::new(
                    "A list of application keys owned by the given developer/vendor".to_string(),
                )],
            },
            exception: Some(Exception {
                data_type: "AccountAPINGException".to_string().into(),
                description: vec![Comment::new(
                    "Generic exception that is thrown if this operation fails for any reason."
                        .to_string(),
                )],
            }),
            description: vec![Comment::new(
                "Get all application keys owned by the given developer/vendor".to_string(),
            )],
        };
        expected_rpc_calls.insert(Name("getDeveloperAppKeys".to_string()), operation);

        // Action
        aping.insert_operation(&input);

        // Assert
        assert_eq!(aping.rpc_calls, expected_rpc_calls);
    }
}
