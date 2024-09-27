//! Common elements of the XML files
use serde::{Deserialize, Serialize};

/// Valid values - used to represent an enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ValidValues {
    /// Vector of possible values
    #[serde(rename = "$value")]
    pub items: Vec<Value>,
}

/// A value of a valid value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    /// The id of the value
    pub id: Option<String>,
    /// The name of the value
    pub name: String,
    /// The description of the value
    pub description: Description,
}

/// The description tag
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Description {
    /// The value of the description
    #[serde(rename = "$value")]
    pub value: Option<String>,
}

/// The parameter tag
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Parameter {
    /// Whether the parameter is mandatory
    pub mandatory: Option<bool>,
    /// The name of the parameter
    pub name: String,
    /// The type of the parameter
    pub r#type: String,
    /// Vector of possible values enclosed within the parameter
    #[serde(rename = "$value")]
    pub items: Vec<ParameterItem>,
}

/// A child item of the <parameter> tag
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ParameterItem {
    /// The description tag
    Description(Description),
    /// The valid values tag
    ValidValues(ValidValues),
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_xml_rs::from_str;

    use super::*;

    #[rstest]
    fn parameter_test() {
        let xml = r#"
        <parameter mandatory="false" name="total" type="double">
            <description>Set a limit on total (matched + unmatched) bet exposure on market group</description>
        </parameter>
        "#;

        let parameter = from_str::<Parameter>(xml);
        match parameter {
            Ok(req) => {
                assert_eq!(
                    req,
                    Parameter {
                        mandatory: Some(false),
                        name: "total".to_owned(),
                        r#type: "double".to_owned(),
                        items: vec![ParameterItem::Description(Description {
                            value: Some(
                                "Set a limit on total (matched + unmatched) bet exposure on market group"
                                    .to_owned()
                            )
                        })]
                    }
                );
            }
            Err(err) => {
                log::error!("Failed to parse XML: {err}");
            }
        }
    }

    #[rstest]
    fn parameter_test_2() {
        let xml = r#"
        <parameter name="errorCode" type="string">
            <description>the unique code for this error</description>
            <validValues>
                <value id="1" name="TOO_MUCH_DATA">
                    <description>The operation requested too much data</description>
                </value>
                <value id="2" name="INVALID_INPUT_DATA">
                    <description>Invalid input data</description>
                </value>
            </validValues>
        </parameter>
        "#;

        let parameter = from_str::<Parameter>(xml);
        match parameter {
            Ok(req) => {
                assert_eq!(
                    req,
                    Parameter {
                        mandatory: None,
                        name: "errorCode".to_owned(),
                        r#type: "string".to_owned(),
                        items: vec![
                            ParameterItem::Description(Description {
                                value: Some("the unique code for this error".to_owned())
                            }),
                            ParameterItem::ValidValues(ValidValues {
                                items: vec![
                                    Value {
                                        id: Some("1".to_owned()),
                                        name: "TOO_MUCH_DATA".to_owned(),
                                        description: Description {
                                            value: Some(
                                                "The operation requested too much data".to_owned()
                                            )
                                        }
                                    },
                                    Value {
                                        id: Some("2".to_owned()),
                                        name: "INVALID_INPUT_DATA".to_owned(),
                                        description: Description {
                                            value: Some("Invalid input data".to_owned())
                                        }
                                    }
                                ]
                            })
                        ]
                    }
                );
            }
            Err(err) => {
                log::error!("Failed to parse XML: {err}");
            }
        }
    }
}
