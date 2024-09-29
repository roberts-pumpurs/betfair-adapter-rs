//! Betfair XML file <operation> tag parser

use serde::{Deserialize, Serialize};

use crate::common::{Description, Parameter};

/// Representation of the <operation> tag
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    /// The name of the operation
    pub name: String,
    /// Version specifier of when the operation was introduced
    pub since: Option<String>,
    /// Potential children of the tag
    #[serde(rename = "$value")]
    pub values: Vec<OperationItem>,
}

/// A child item of the <operation> tag
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[expect(clippy::module_name_repetitions)]
pub enum OperationItem {
    /// Description tag
    Description(Description),
    /// Parameters tag
    Parameters(Parameters),
}

/// Representation of the <parameters> tag
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Parameters {
    /// Vector of potential children of the tag
    #[serde(rename = "$value")]
    pub values: Vec<ParametersItems>,
}

/// A child item of the <parameters> tag
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ParametersItems {
    /// Request tag
    Request(Request),
    /// `SimpleResponse` tag
    SimpleResponse(SimpleResponse),
    /// Exceptions tag
    Exceptions(Exceptions),
}

/// Representation of the <request> tag
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Request {
    /// Vector of potential children of the tag
    #[serde(rename = "$value")]
    pub values: Option<Vec<Parameter>>,
}

/// Representation of the <simpleResponse> tag
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SimpleResponse {
    /// The type of the response
    pub r#type: String,
    /// The description of the response
    pub description: Description,
}

/// Representation of the <exceptions> tag
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Exceptions {
    /// Vector of potential exact Exception tags
    #[serde(rename = "$value")]
    pub values: Vec<Exception>,
}

/// Representation of the <exception> tag
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Exception {
    /// The type of the exception
    pub r#type: String,
    /// The description of the exception
    pub description: Description,
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
#[expect(clippy::indexing_slicing)]
mod tests {
    use rstest::rstest;
    use serde_xml_rs::from_str;

    use super::*;
    use crate::common::ParameterItem;

    #[rstest]
    fn test_parse_request() {
        let xml = r#"
            <request>
                <parameter mandatory="true" name="filter" type="MarketFilter">
                    <description>The filter to select desired markets. All markets that match the criteria in the filter
                        are selected.
                    </description>
                </parameter>
                <parameter name="locale" type="string">
                    <description>The language used for the response. If not specified, the default is returned.
                    </description>
                </parameter>
            </request>
    "#;

        let req = from_str::<Request>(xml).unwrap();
        assert_eq!(req.values.unwrap().len(), 2);
    }

    #[rstest]
    fn test_parse_simple_response() {
        let xml = r#"
            <simpleResponse type="list(EventTypeResult)">
                <description>output data</description>
            </simpleResponse>
    "#;

        let sr = from_str::<SimpleResponse>(xml).unwrap();
        assert_eq!(sr.r#type, "list(EventTypeResult)");
        assert_eq!(sr.description.value.unwrap().as_str(), "output data");
    }

    #[rstest]
    fn test_parse_exceptions() {
        let xml = r#"
            <exceptions>
                <exception type="APINGException">
                    <description>Generic exception that is thrown if this operation fails for any reason.</description>
                </exception>
            </exceptions>
    "#;

        let exceptions = from_str::<Exceptions>(xml).unwrap();
        assert_eq!(exceptions.values.len(), 1);
        assert_eq!(exceptions.values[0].r#type, "APINGException".to_owned());
    }

    #[rstest]
    fn test_parse_parameters() {
        let xml = r#"
        <parameters>
            <request>
                <parameter mandatory="true" name="filter" type="MarketFilter">
                    <description>The filter to select desired markets. All markets that match the criteria in the filter
                        are selected.
                    </description>
                </parameter>
                <parameter name="locale" type="string">
                    <description>The language used for the response. If not specified, the default is returned.
                    </description>
                </parameter>
            </request>
            <simpleResponse type="list(EventTypeResult)">
                <description>output data</description>
            </simpleResponse>
            <exceptions>
                <exception type="APINGException">
                    <description>Generic exception that is thrown if this operation fails for any reason.</description>
                </exception>
            </exceptions>
        </parameters>
    "#;

        let params = from_str::<Parameters>(xml).unwrap();
        assert_eq!(params.values.len(), 3);
        assert!(matches!(params.values[0], ParametersItems::Request(_)));
        assert!(matches!(
            params.values[1],
            ParametersItems::SimpleResponse(_)
        ));
        assert!(matches!(params.values[2], ParametersItems::Exceptions(_)));
    }

    #[rstest]
    fn test_parse_operation() {
        let xml = r#"
    <operation name="listEventTypes" since="1.0.0">
        <description>Returns a list of Event Types (i.e. Sports) associated with the markets selected by the MarketFilter.
        </description>
        <parameters>
            <request>
                <parameter mandatory="true" name="filter" type="MarketFilter">
                    <description>The filter to select desired markets. All markets that match the criteria in the filter are selected.
                    </description>
                </parameter>
                <parameter name="locale" type="string">
                    <description>The language used for the response. If not specified, the default is returned.
                    </description>
                </parameter>
            </request>
            <simpleResponse type="list(EventTypeResult)">
                <description>output data</description>
            </simpleResponse>
            <exceptions>
                <exception type="APINGException">
                    <description>Generic exception that is thrown if this operation fails for any reason.</description>
                </exception>
            </exceptions>
        </parameters>
    </operation>
    "#;

        let op = from_str::<Operation>(xml).unwrap();
        let expected = Operation {
            name: "listEventTypes".to_owned(),
            since: Some("1.0.0".to_owned()),
            values: vec![
                OperationItem::Description(Description {
                    value: Some("Returns a list of Event Types (i.e. Sports) associated with the markets selected by the MarketFilter.".to_owned())
                }),
                OperationItem::Parameters(Parameters {
                    values: vec![
                        ParametersItems::Request(Request {
                            values: Some(vec![
                                Parameter {
                                    mandatory: Some(true),
                                    name: "filter".to_owned(),
                                    r#type: "MarketFilter".to_owned(),
                                    items: vec![
                                        ParameterItem::Description(Description {
                                            value: Some("The filter to select desired markets. All markets that match the criteria in the filter are selected.".to_owned())
                                        })
                                    ]
                                },
                                Parameter {
                                    mandatory: None,
                                    name: "locale".to_owned(),
                                    r#type: "string".to_owned(),
                                    items: vec![
                                        ParameterItem::Description(Description {
                                            value: Some("The language used for the response. If not specified, the default is returned.".to_owned())
                                        })
                                    ]
                                }
                            ])
                        }),
                        ParametersItems::SimpleResponse(SimpleResponse {
                            r#type: "list(EventTypeResult)".to_owned(),
                            description: Description {
                                value: Some("output data".to_owned())
                            }
                        }),
                        ParametersItems::Exceptions(Exceptions {
                            values: vec![
                                Exception {
                                    r#type: "APINGException".to_owned(),
                                    description: Description {
                                        value: Some("Generic exception that is thrown if this operation fails for any reason.".to_owned())
                                    }
                                }
                            ]
                        })
                    ]
                })
            ]
        };
        assert_eq!(op, expected);
    }

    #[rstest]
    fn test_parse_operation_2() {
        let xml = r#"
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

        let op = from_str::<Operation>(xml).unwrap();
        let expected = Operation {
            name: "getDeveloperAppKeys".to_owned(),
            since: Some("1.0.0".to_owned()),
            values: vec![
                OperationItem::Description(Description {
                    value: Some("Get all application keys owned by the given developer/vendor".to_owned())
                }),
                OperationItem::Parameters(Parameters {
                    values: vec![
                        ParametersItems::Request(Request {
                            values: None
                        }),
                        ParametersItems::SimpleResponse(SimpleResponse {
                            description: Description {
                                value: Some("A list of application keys owned by the given developer/vendor".to_owned())
                            },
                            r#type: "list(DeveloperApp)".to_owned()
                        }),
                        ParametersItems::Exceptions(Exceptions {
                            values: vec![
                                Exception {
                                    r#type: "AccountAPINGException".to_owned(),
                                    description: Description {
                                        value: Some("Generic exception that is thrown if this operation fails for any reason.".to_owned())
                                    }
                                }
                            ]
                        })
                    ]
                })
            ]
        };
        assert_eq!(op, expected);
    }
}
