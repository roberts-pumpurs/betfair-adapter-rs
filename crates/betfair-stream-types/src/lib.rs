use serde::{Deserialize, Serialize};

pub mod all_request_types_example;
pub mod all_response_types_example;
pub mod authentication_message;
pub mod authentication_message_all_of;
pub mod connection_message;
pub mod connection_message_all_of;
pub mod heartbeat_message;
pub mod key_line_definition;
pub mod key_line_selection;
pub mod market_change;
pub mod market_change_message;
pub mod market_change_message_all_of;
pub mod market_data_filter;
pub mod market_definition;
pub mod market_filter;
pub mod market_subscription_message;
pub mod market_subscription_message_all_of;
pub mod order;
pub mod order_change_message;
pub mod order_change_message_all_of;
pub mod order_filter;
pub mod order_market_change;
pub mod order_runner_change;
pub mod order_subscription_message;
pub mod order_subscription_message_all_of;
pub mod price_ladder_definition;
pub mod request_message;
pub mod response_message;
pub mod runner_change;
pub mod runner_definition;
pub mod status_message;
pub mod status_message_all_of;
pub mod strategy_match_change;

pub use all_request_types_example::*;
pub use all_response_types_example::*;
pub use authentication_message::*;
pub use authentication_message_all_of::*;
pub use connection_message::*;
pub use connection_message_all_of::*;
pub use heartbeat_message::*;
pub use key_line_definition::*;
pub use key_line_selection::*;
pub use market_change::*;
pub use market_change_message::*;
pub use market_change_message_all_of::*;
pub use market_data_filter::*;
pub use market_definition::*;
pub use market_filter::*;
pub use market_subscription_message::*;
pub use market_subscription_message_all_of::*;
pub use order::*;
pub use order_change_message::*;
pub use order_change_message_all_of::*;
pub use order_filter::*;
pub use order_market_change::*;
pub use order_runner_change::*;
pub use order_subscription_message::*;
pub use order_subscription_message_all_of::*;
pub use price_ladder_definition::*;
pub use request_message::*;
pub use response_message::*;
pub use runner_change::*;
pub use runner_definition::*;
pub use status_message::*;
pub use status_message_all_of::*;
pub use strategy_match_change::*;

/// The type of error in case of a failure
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    NoAppKey,
    InvalidAppKey,
    NoSession,
    InvalidSessionInformation,
    NotAuthorized,
    InvalidInput,
    InvalidClock,
    UnexpectedError,
    Timeout,
    SubscriptionLimitExceeded,
    InvalidRequest,
    ConnectionFailed,
    MaxConnectionLimitExceeded,
    TooManyRequests,
}

impl Default for ErrorCode {
    fn default() -> ErrorCode {
        Self::NoAppKey
    }
}

/// The status of the last request
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatusCode {
    Success,
    Failure,
}

impl Default for StatusCode {
    fn default() -> StatusCode {
        Self::Success
    }
}

/// Change Type - set to indicate the type of change - if null this is a delta)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Ct {
    SubImage,
    ResubDelta,
    Heartbeat,
}

impl Default for Ct {
    fn default() -> Ct {
        Self::SubImage
    }
}
/// Segment Type - if the change is split into multiple segments, this denotes the beginning and end
/// of a change, and segments in between. Will be null if data is not segmented
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SegmentType {
    SegStart,
    Seg,
    SegEnd,
}

impl Default for SegmentType {
    fn default() -> SegmentType {
        Self::SegStart
    }
}
