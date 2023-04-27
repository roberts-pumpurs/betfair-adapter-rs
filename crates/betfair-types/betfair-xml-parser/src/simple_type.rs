use serde::{Deserialize, Serialize};

use crate::common::ValidValues;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SimpleType {
    name: String,
    r#type: String,
    valid_values: Option<ValidValues>,
}


#[cfg(test)]
mod tests {

    use serde_xml_rs::from_str;

    use crate::common::{Value, Description};

    use super::*;

    #[rstest::rstest]
    fn test_simple_raw_type() {
        let xml = r#"
    <simpleType name="MarketType" type="string"/>
        "#;

        let simple_type: SimpleType = from_str(xml).unwrap();
        assert_eq!(simple_type.name, "MarketType");
        assert_eq!(simple_type.r#type, "string");
        assert_eq!(simple_type.valid_values, None);
    }
    #[rstest::rstest]
    fn test_simple_raw_type_2() {
        let xml = r#"
    <simpleType name="Handicap" type="double"/>
        "#;

        let simple_type: SimpleType = from_str(xml).unwrap();
        assert_eq!(simple_type.name, "Handicap");
        assert_eq!(simple_type.r#type, "double");
        assert_eq!(simple_type.valid_values, None);
    }

    #[rstest::rstest]
    fn test_simple_enum_type() {
        let xml = r#"
    <simpleType name="MarketProjection" type="string">
        <validValues>
            <value name="COMPETITION">
                <description>If not selected then the competition will not be returned with marketCatalogue
                </description>
            </value>
            <value name="EVENT">
                <description>If not selected then the event will not be returned with marketCatalogue</description>
            </value>
            <value name="EVENT_TYPE">
                <description>If not selected then the eventType will not be returned with marketCatalogue</description>
            </value>
        </validValues>
    </simpleType>
        "#;

        let simple_type: SimpleType = from_str(xml).unwrap();
        assert_eq!(simple_type.name, "MarketProjection");
        assert_eq!(simple_type.r#type, "string");
        assert_eq!(simple_type.valid_values.as_ref().unwrap().items.len(), 3);
        assert_eq!(simple_type.valid_values.unwrap().items, vec![
            Value {
                id: None,
                name: "COMPETITION".to_string(),
                description: Description {value: Some("If not selected then the competition will not be returned with marketCatalogue".to_string() )},
            },
            Value {
                id: None,
                name: "EVENT".to_string(),
                description: Description {value: Some("If not selected then the event will not be returned with marketCatalogue".to_string() )},
            },
            Value {
                id: None,
                name: "EVENT_TYPE".to_string(),
                description: Description {value: Some("If not selected then the eventType will not be returned with marketCatalogue".to_string() )},
            },
        ]);
    }

    #[rstest::rstest]
    fn test_simple_enum_type_2() {
        let xml = r#"
    <simpleType name="InstructionReportStatus" type="string">
        <validValues>
            <value name="SUCCESS">
                <description/>
            </value>
            <value name="FAILURE">
                <description/>
            </value>
            <value name="TIMEOUT">
                <description/>
            </value>
        </validValues>
    </simpleType>
        "#;

        let simple_type: SimpleType = from_str(xml).unwrap();
        assert_eq!(simple_type.name, "InstructionReportStatus");
        assert_eq!(simple_type.r#type, "string");
        assert_eq!(simple_type.valid_values.as_ref().unwrap().items.len(), 3);
        assert_eq!(
            simple_type.valid_values.unwrap().items,
            vec![
                Value {
                    id: None,
                    name: "SUCCESS".to_string(),
                    description: Description { value: None },
                },
                Value {
                    id: None,
                    name: "FAILURE".to_string(),
                    description: Description { value: None },
                },
                Value {
                    id: None,
                    name: "TIMEOUT".to_string(),
                    description: Description { value: None },
                },
            ]
        );
    }
}
