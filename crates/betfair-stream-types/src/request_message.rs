use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op")]
#[serde(rename_all = "camelCase")]
pub enum RequestMessage {
    #[serde(rename = "authentication")]
    Authentication {
        /// Client generated unique id to link request with response (like json rpc)
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        session: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "appKey")]
        app_key: Option<String>,
    },
    #[serde(rename = "heartbeat")]
    Heartbeat {
        /// Client generated unique id to link request with response (like json rpc)
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<i32>,
    },
    #[serde(rename = "marketSubscription")]
    MarketSubscription {
        /// Client generated unique id to link request with response (like json rpc)
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<i32>,
        /// Segmentation Enabled - allow the server to send large sets of data in segments, instead
        /// of a single block
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "segmentationEnabled")]
        segmentation_enabled: Option<bool>,
        /// Token value delta (received in MarketChangeMessage) that should be passed to resume a
        /// subscription
        #[serde(skip_serializing_if = "Option::is_none")]
        clk: Option<String>,
        /// Heartbeat Milliseconds - the heartbeat rate (looped back on initial image after
        /// validation: bounds are 500 to 5000)
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "heartbeatMs")]
        heartbeat_ms: Option<i64>,
        /// Token value (received in initial MarketChangeMessage) that should be passed to resume a
        /// subscription
        #[serde(skip_serializing_if = "Option::is_none")]
        initial_clk: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "marketFilter")]
        market_filter: Option<Box<super::MarketFilter>>,
        /// Conflate Milliseconds - the conflation rate (looped back on initial image after
        /// validation: bounds are 0 to 120000)
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "conflateMs")]
        conflate_ms: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "marketDataFilter")]
        market_data_filter: Option<Box<super::MarketDataFilter>>,
    },
    #[serde(rename = "orderSubscription")]
    OrderSubscription {
        /// Client generated unique id to link request with response (like json rpc)
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<i32>,
        /// Segmentation Enabled - allow the server to send large sets of data in segments, instead
        /// of a single block
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "segmentationEnabled")]
        segmentation_enabled: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "orderFilter")]
        order_filter: Option<Box<super::OrderFilter>>,
        /// Token value delta (received in MarketChangeMessage) that should be passed to resume a
        /// subscription
        #[serde(skip_serializing_if = "Option::is_none")]
        clk: Option<String>,
        /// Heartbeat Milliseconds - the heartbeat rate (looped back on initial image after
        /// validation: bounds are 500 to 5000)
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "heartbeatMs")]
        heartbeat_ms: Option<i64>,
        /// Token value (received in initial MarketChangeMessage) that should be passed to resume a
        /// subscription
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "initialClk")]
        initial_clk: Option<String>,
        /// Conflate Milliseconds - the conflation rate (looped back on initial image after
        /// validation: bounds are 0 to 120000)
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "conflateMs")]
        conflate_ms: Option<i64>,
    },
}
