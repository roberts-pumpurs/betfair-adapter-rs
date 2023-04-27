use serde::{Deserialize, Serialize};

use crate::common::{Description, Parameter};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub name: String,
    pub since: String,
    #[serde(rename = "$value")]
    pub values: Vec<operation::Items>,
}
pub mod operation {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub enum Items {
        Description(Description),
        Parameters(Parameters),
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Parameters {
    #[serde(rename = "$value")]
    pub values: Vec<parameters::Items>,
}

mod parameters {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub enum Items {
        Request(Request),
        SimpleResponse(SimpleResponse),
        Exceptions(Exceptions),
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Request {
    #[serde(rename = "$value")]
    pub values: Vec<Parameter>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SimpleResponse {
    pub r#type: String,
    pub description: Description,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Exceptions {
    #[serde(rename = "$value")]
    pub values: Vec<Exception>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Exception {
    pub r#type: String,
    pub description: Description,
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_xml_rs::from_str;

    use super::*;

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
        assert_eq!(req.values.len(), 2);
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
        assert_eq!(
            exceptions.values[0].r#type,
            "APINGException".to_string()
        );
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
        assert!(matches!(params.values[0], parameters::Items::Request(_)));
        assert!(matches!(params.values[1], parameters::Items::SimpleResponse(_)));
        assert!(matches!(params.values[2], parameters::Items::Exceptions(_)));
    }

    #[rstest]
    fn test_parse_operation() {
        let xml = r#"
    <operation name="listEventTypes" since="1.0.0">
        <description>Returns a list of Event Types (i.e. Sports) associated with the markets selected by the
            MarketFilter.
        </description>
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
    </operation>
    "#;

        let op = from_str::<Operation>(xml).unwrap();
        assert_eq!(op.name, "listEventTypes");
        assert_eq!(op.since, "1.0.0");
        assert_eq!(op.values.len(), 2);
        assert!(matches!(op.values[0], operation::Items::Description(_)));
        assert!(matches!(op.values[1], operation::Items::Parameters(_)));
    }
}
