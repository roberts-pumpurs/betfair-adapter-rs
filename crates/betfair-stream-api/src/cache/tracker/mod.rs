mod market_stream_tracker;
mod order_stream_tracker;

use betfair_stream_types::response::market_change_message::MarketChangeMessage;
use betfair_stream_types::response::order_change_message::OrderChangeMessage;
use betfair_stream_types::response::{
    ChangeType, Clock, DataChange, DatasetChangeMessage, InitialClock,
};
use serde::de::DeserializeOwned;

use self::market_stream_tracker::MarketStreamTracker;
use self::order_stream_tracker::OrderStreamTracker;
use super::primitives::{MarketBookCache, OrderBookCache};

/// Separate stream struct to hold market/order caches
#[derive(Debug, Clone)]
pub struct StreamState {
    pub stream_id: Option<u64>,
    pub update_clk: Option<Clock>,
    pub max_latency_ms: Option<u64>,
    pub unique_id: Option<i32>,
    pub initial_clock: Option<InitialClock>,
    pub time_created: chrono::DateTime<chrono::Utc>,
    pub time_updated: chrono::DateTime<chrono::Utc>,
    pub market_stream_tracker: MarketStreamTracker,
    pub order_stream_tracker: OrderStreamTracker,
}

pub enum Updates<'a> {
    Market(Vec<&'a MarketBookCache>),
    Order(Vec<&'a OrderBookCache>),
}

pub struct HasFullImage(pub bool);

impl Default for StreamState {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            stream_id: None,
            update_clk: None,
            max_latency_ms: None,
            unique_id: None,
            initial_clock: None,
            time_created: chrono::Utc::now(),
            time_updated: chrono::Utc::now(),
            market_stream_tracker: MarketStreamTracker::new(),
            order_stream_tracker: OrderStreamTracker::new(),
        }
    }

    pub fn order_change_update(&mut self, msg: OrderChangeMessage) -> Option<Vec<&OrderBookCache>> {
        match msg.change_type {
            Some(ChangeType::SubImage) => {
                self.update_clk(&msg);
                self.order_stream_tracker.process(msg).0
            }
            Some(ChangeType::Heartbeat) => {
                self.update_clk(&msg);
                self.on_heartbeat(&msg);
                None
            }
            None | Some(ChangeType::ResubDelta) => {
                self.on_update(&msg);
                self.order_stream_tracker.process(msg).0
            }
        }
    }

    pub fn market_change_update(
        &mut self,
        msg: MarketChangeMessage,
    ) -> Option<Vec<&MarketBookCache>> {
        match msg.change_type {
            Some(ChangeType::SubImage) => {
                self.update_clk(&msg);
                self.market_stream_tracker.process(msg).0
            }
            Some(ChangeType::Heartbeat) => {
                self.update_clk(&msg);
                self.on_heartbeat(&msg);
                None
            }
            None | Some(ChangeType::ResubDelta) => {
                self.on_update(&msg);
                self.market_stream_tracker.process(msg).0
            }
        }
    }

    fn on_update<T: DeserializeOwned + DataChange<T>>(&mut self, msg: &DatasetChangeMessage<T>) {
        if self.update_clk.is_some() {
            self.update_clk(msg);
        }
        if let (Some(publish_time), Some(max_latency_ms)) = (msg.publish_time, self.max_latency_ms)
        {
            let latency = chrono::Utc::now().signed_duration_since(publish_time);
            if latency.num_milliseconds() > max_latency_ms.try_into().unwrap_or(0) {
                tracing::warn!(
                    "High Latency! {:?}ms is greater than max_latency_ms of {:?}ms",
                    latency.num_milliseconds(),
                    max_latency_ms
                );
            }
        }
    }

    fn on_heartbeat<T: DeserializeOwned + DataChange<T>>(&mut self, msg: &DatasetChangeMessage<T>) {
        self.update_clk(msg);
    }

    pub(crate) fn clear_stale_cache(&mut self, publish_time: chrono::DateTime<chrono::Utc>) {
        self.market_stream_tracker.clear_stale_cache(publish_time);
        self.order_stream_tracker.clear_stale_cache(publish_time);
    }

    fn update_clk<T: DeserializeOwned + DataChange<T>>(&mut self, data: &DatasetChangeMessage<T>) {
        if let Some(ref initial_clock) = data.initial_clock {
            self.initial_clock = Some(initial_clock.clone());
        }

        if let Some(ref update_clk) = data.clock {
            self.update_clk = Some(update_clk.clone());
        }

        self.time_updated = chrono::Utc::now();
    }
}
