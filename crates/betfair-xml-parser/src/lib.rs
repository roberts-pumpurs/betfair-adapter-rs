#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![allow(clippy::self_named_module_files)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

//! Betfair XML file parser.
//! The intended use is to parse the XML files and generate Rust structs that
//! can be used to generate code for the Betfair API-NG in Rust (or other languages?)
//!
//! Input: XML files from Betfair API-NG
//! Output: Rust structs as representation of the XML files

use common::Description;
use log as _;
use serde::{Deserialize, Serialize};

pub mod common;
pub mod data_type;
pub mod exception_type;
pub mod operation;
pub mod simple_type;

/// Top level representation of the XML file
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Interface {
    /// The name of the interface
    pub name: String,
    /// The owner of the interface
    pub owner: String,
    /// The version of the interface
    pub version: String,
    /// The date of publication
    pub date: String,
    /// The namespace of the interface
    pub namespace: String,
    /// Vector of possible values enclosed within the interface
    #[serde(rename = "$value")]
    pub items: Vec<InterfaceItems>,
}

/// A child item of the <interface> tag
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum InterfaceItems {
    /// The description of the interface
    Description(Description),
    /// A simple type tag
    SimpleType(simple_type::SimpleType),
    /// A data type tag
    DataType(data_type::DataType),
    /// An exception type tag
    ExceptionType(exception_type::ExceptionType),
    /// An operation tag
    Operation(operation::Operation),
}

/// Parse the XML file into a Rust struct
///
/// # Arguments
///
/// * `xml` - The XML file as a string
///
/// # Returns
///
/// * `Result<Interface, serde_xml_rs::Error>` - The parsed interface or an error
///
/// # Example
///
/// ```
/// let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
/// <interface name="HeartbeatAPING" owner="BDP" version="1.0.0" date="now()" namespace="com.betfair.heartbeat.api"
///            xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
///     <description>Heartbeat</description>
///     <operation name="heartbeat" since="1.0.0">
///         <description>...</description>
///         <parameters>...</parameters>
///     </operation>
/// </interface>"#;
///
/// let interface: Interface = xml.into();
/// ```
///
/// # Errors
///
/// * `serde_xml_rs::Error` - If the XML file is not valid
/// ```
pub fn parse_interface(xml: &str) -> Result<Interface, serde_xml_rs::Error> {
    serde_xml_rs::from_str(xml)
}

impl From<&str> for Interface {
    fn from(val: &str) -> Self {
        parse_interface(val).unwrap_or_else(|_| Into::into("Failed to parse XML file"))
    }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn interface_test() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<interface name="HeartbeatAPING" owner="BDP" version="1.0.0" date="now()" namespace="com.betfair.heartbeat.api"
           xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <description>Heartbeat</description>

    <operation name="heartbeat" since="1.0.0">
        <description>
            This heartbeat operation is provided to help customers have their positions managed automatically in the
            event of their API clients losing connectivity with the Betfair API.

            If a heartbeat request is not received within a prescribed time period, then Betfair will attempt to cancel
            all 'LIMIT' type bets for the given customer on the given exchange.

            There is no guarantee that this service will result in all bets being cancelled as there are a number of
            circumstances where bets are unable to be cancelled. Manual intervention is strongly advised in the event of a loss of connectivity
            to ensure that positions are correctly managed.

            If this service becomes unavailable for any reason, then your heartbeat will be unregistered automatically to avoid bets being
            inadvertently cancelled upon resumption of service.
            you should manage your position manually until the service is resumed.

            Heartbeat data may also be lost in the unlikely event of  nodes failing within the cluster, which
            may result in your position not being managed until a subsequent heartbeat request is received.
        </description>

        <parameters>
            <request>
                <parameter mandatory="true" name="preferredTimeoutSeconds" type="i32">
                    <description>
                        Maximum period in seconds that may elapse (without a subsequent heartbeat request),
                        before a cancellation request is automatically submitted on your behalf.

                        Passing 0 will result in your heartbeat being unregistered (or ignored if you have no current
                        heartbeat registered).

                        You will still get an actionPerformed value returned when passing 0, so this may be used to
                        determine if any action was performed since your last heartbeat, without actually registering a new heartbeat.

                        Passing a negative value will result in an error being returned, INVALID_INPUT_DATA.

                        Any errors while registering your heartbeat will result in a error being returned, UNEXPECTED_ERROR.

                        Passing a value that is less than the minimum timeout will result in your heartbeat adopting the
                        minimum timeout.

                        Passing a value that is greater than the maximum timeout will result in your heartbeat adopting
                        the maximum timeout.

                        The minimum and maximum timeouts are subject to change, so your client should utilise the
                        returned actualTimeoutSeconds to set an appropriate frequency for your subsequent heartbeat requests.
                    </description>
                </parameter>
            </request>
            <simpleResponse type="HeartbeatReport">
                <description>Response from heartbeat operation</description>
            </simpleResponse>
            <exceptions>
                <exception type="APINGException">
                    <description>Thrown if the operation fails</description>
                </exception>
            </exceptions>
        </parameters>
    </operation>


    <dataType name="HeartbeatReport">
        <description>Response from heartbeat operation</description>
        <parameter mandatory="true" name="actionPerformed" type="ActionPerformed">
            <description>The action performed since your last heartbeat request.</description>
        </parameter>
        <parameter mandatory="true" name="actualTimeoutSeconds" type="i32">
            <description>The actual timeout applied to your heartbeat request, see timeout request parameter description
                for details.
            </description>
        </parameter>
    </dataType>


      <exceptionType name="APINGException" prefix="HBT">
        <description>This exception is thrown when an operation fails</description>
        <parameter name="errorCode" type="string">
            <description>the unique code for this error</description>
            <validValues>
                <value id="1" name="INVALID_INPUT_DATA">
                    <description>Invalid input data</description>
                </value>
                <value id="2" name="INVALID_SESSION_INFORMATION">
                    <description>The session token passed is invalid</description>
                </value>
                <value id="3" name="NO_APP_KEY">
                    <description>An application key is required for this operation</description>
                </value>
                <value id="4" name="NO_SESSION">
                    <description>A session token is required for this operation</description>
                </value>
                <value id="5" name="INVALID_APP_KEY">
                    <description>The application key passed is invalid</description>
                </value>
                <value id="6" name="UNEXPECTED_ERROR">
                    <description>An unexpected internal error occurred that prevented successful request processing.</description>
                </value>
            </validValues>
        </parameter>
        <parameter name="errorDetails" type="string">
            <description>Specific error details</description>
        </parameter>
        <parameter name="requestUUID" type="string">
            <description/>
        </parameter>
    </exceptionType>

    <simpleType name="ActionPerformed" type="string">
        <validValues>
            <value name="NONE">
                <description>No action was performed since last heartbeat, or this is the first heartbeat</description>
            </value>
            <value name="CANCELLATION_REQUEST_SUBMITTED">
                <description>A request to cancel all unmatched bets was submitted since last heartbeat</description>
            </value>
            <value name="ALL_BETS_CANCELLED">
                <description>All unmatched bets were cancelled since last heartbeat</description>
            </value>
            <value name="SOME_BETS_NOT_CANCELLED">
                <description>Not all unmatched bets were cancelled since last heartbeat</description>
            </value>
            <value name="CANCELLATION_REQUEST_ERROR">
                <description>There was an error requesting cancellation, no bets have been cancelled</description>
            </value>
            <value name="CANCELLATION_STATUS_UNKNOWN">
                <description>There was no response from requesting cancellation, cancellation status unknown</description>
            </value>
        </validValues>
    </simpleType>

</interface>
        "#;

        let interface: Interface = xml.into();
        assert_eq!(interface.name, "HeartbeatAPING");
        assert_eq!(interface.owner, "BDP");
        assert_eq!(interface.version, "1.0.0");
        assert_eq!(interface.date, "now()");
        assert_eq!(interface.namespace, "com.betfair.heartbeat.api");
        assert_eq!(interface.items.len(), 5);
        assert!(matches!(interface.items[0], InterfaceItems::Description(_)));
        assert!(matches!(interface.items[1], InterfaceItems::Operation(_)));
        assert!(matches!(interface.items[2], InterfaceItems::DataType(_)));
        assert!(matches!(
            interface.items[3],
            InterfaceItems::ExceptionType(_)
        ));
        assert!(matches!(interface.items[4], InterfaceItems::SimpleType(_)));
    }
}
