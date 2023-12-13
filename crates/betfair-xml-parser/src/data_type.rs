//! Betfair XML file <dataType> tag parser

use serde::{Deserialize, Serialize};

use crate::common::{Description, Parameter};

/// Representation of the <dataType> tag
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DataType {
    /// The name of the data type
    pub name: String,
    /// Vector of possible values
    #[serde(rename = "$value")]
    pub values: Vec<DataTypeItems>,
}

/// A child item of the <dataType> tag
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DataTypeItems {
    /// The description of the data type
    Description(Description),
    /// A parameter tag of the data type
    Parameter(Parameter),
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_xml_rs::from_str;

    use super::*;

    #[rstest]
    fn test_parse_data_type() {
        let xml = r#"
    <dataType name="RunnerCatalog">
        <description>Information about the Runners (selections) in a market</description>
        <parameter mandatory="true" name="selectionId" type="SelectionId">
            <description>The unique id for the selection.</description>
        </parameter>
        <parameter name="runnerName" type="string" mandatory="true">
            <description>The name of the runner</description>
        </parameter>
        <parameter mandatory="true" name="handicap" type="double">
            <description>The handicap</description>
        </parameter>
        <parameter mandatory="true" name="sortPriority" type="i32">
            <description>The sort priority of this runner</description>
        </parameter>
        <parameter name="metadata" type="map(string,string)">
            <description>Metadata associated with the runner</description>
        </parameter>
    </dataType>

    "#;

        let req = from_str::<DataType>(xml).unwrap();
        assert_eq!(req.values.len(), 6);
        assert_eq!(req.name, "RunnerCatalog");
        assert!(matches!(req.values[0], DataTypeItems::Description(_)));
        assert!(matches!(req.values[1], DataTypeItems::Parameter(_)));
        assert!(matches!(req.values[2], DataTypeItems::Parameter(_)));
        assert!(matches!(req.values[3], DataTypeItems::Parameter(_)));
        assert!(matches!(req.values[4], DataTypeItems::Parameter(_)));
        assert!(matches!(req.values[5], DataTypeItems::Parameter(_)));
    }
}
