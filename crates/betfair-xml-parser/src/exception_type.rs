//! Betfair XML file <exceptionType> tag parser

use serde::{Deserialize, Serialize};

use crate::common::{Description, Parameter};

/// Representation of the <exceptionType> tag
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExceptionType {
    /// The name of the exception
    pub name: String,
    /// The prefix of the exception
    pub prefix: String,
    /// Vector of possible values
    #[serde(rename = "$value")]
    pub values: Vec<ExceptionTypeItems>,
}

/// A child item of the <exceptionType> tag
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ExceptionTypeItems {
    /// The description of the exception
    Description(Description),
    /// A parameter tag of the exception
    Parameter(Parameter),
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_xml_rs::from_str;

    use super::*;

    #[rstest]
    fn test_parse_aping_exception() {
        let xml = r#"
    <exceptionType name="APINGException" prefix="ANGX">
        <description>This exception is thrown when an operation fails</description>
        <parameter name="errorCode" type="string">
            <description>the unique code for this error</description>
            <validValues>
                <value id="1" name="TOO_MUCH_DATA">
                    <description>The operation requested too much data</description>
                </value>
                <value id="2" name="INVALID_INPUT_DATA">
                    <description>Invalid input data</description>
                </value>
                <value id="3" name="INVALID_SESSION_INFORMATION">
                    <description>The session token passed is invalid</description>
                </value>
                <value id="4" name="NO_APP_KEY">
                    <description>An application key is required for this operation</description>
                </value>
                <value id="5" name="NO_SESSION">
                    <description>A session token is required for this operation</description>
                </value>
                <value id="6" name="UNEXPECTED_ERROR">
                    <description>An unexpected internal error occurred that prevented successful request processing.
                    </description>
                </value>
                <value id="7" name="INVALID_APP_KEY">
                    <description>The application key passed is invalid</description>
                </value>
                <value id="8" name="TOO_MANY_REQUESTS">
                    <description>There are too many pending requests</description>
                </value>
                <value id="9" name="SERVICE_BUSY">
                    <description>The service is currently too busy to service this request</description>
                </value>
                <value id="10" name="TIMEOUT_ERROR">
                    <description>Internal call to downstream service timed out</description>
                </value>
                <value id="11" name="APP_KEY_CREATION_FAILED">
                    <description>The application key creation has failed</description>
                </value>
                <value id="12" name="DUPLICATE_APP_NAME">
                    <description>The application name specified already exists</description>
                </value>
                <value id="13" name="APP_CREATION_FAILED">
                    <description>The application name specified is too long</description>
                </value>
                <value id="14" name="REQUEST_SIZE_EXCEEDS_LIMIT">
                    <description>The request has exceeded the maximum allowed size</description>
                </value>
                <value id="15" name="ACCESS_DENIED">
                    <description>The access to this functionality is not allowed</description>
                </value>
                <value id="16" name="INVALID_MARKET_GROUP">
                    <description>
                        Provided market group id does not identify a known market group
                    </description>
                </value>
                <value id="17" name="EXPOSURE_LIMIT_NOT_EXIST">
                    <description>
                        Unable to delete/update limit as it doesn't exist
                    </description>
                </value>
                <value id="18" name="MARKET_GROUP_NOT_BLOCKED">
                    <description>
                        Unable to unblock market group after exposure limit breach, market group is not blocked
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
    </exceptionType>
    "#;

        let exception = from_str::<ExceptionType>(xml).unwrap();
        assert_eq!(exception.name, "APINGException");
        assert_eq!(exception.prefix, "ANGX");
        assert_eq!(exception.values.len(), 4);
        assert_eq!(
            exception.values[0],
            ExceptionTypeItems::Description(Description {
                value: Some("This exception is thrown when an operation fails".to_string()),
            })
        );
        assert!(matches!(
            exception.values[1],
            ExceptionTypeItems::Parameter(_)
        ));
    }
}
