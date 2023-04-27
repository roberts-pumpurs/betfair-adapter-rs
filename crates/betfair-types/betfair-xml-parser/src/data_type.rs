use serde::{Deserialize, Serialize};

use crate::common::{Description, Parameter};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DataType {
    pub name: String,
    #[serde(rename = "$value")]
    pub values: Vec<data_type::Items>,
}
pub mod data_type {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub enum Items {
        Description(Description),
        Parameter(Parameter),
    }
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
        assert!(matches!(req.values[0], data_type::Items::Description(_)));
        assert!(matches!(req.values[1], data_type::Items::Parameter(_)));
        assert!(matches!(req.values[2], data_type::Items::Parameter(_)));
        assert!(matches!(req.values[3], data_type::Items::Parameter(_)));
        assert!(matches!(req.values[4], data_type::Items::Parameter(_)));
        assert!(matches!(req.values[5], data_type::Items::Parameter(_)));
    }
}
