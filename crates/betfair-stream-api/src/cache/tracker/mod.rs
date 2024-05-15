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
#[derive(Debug)]
pub(crate) struct StreamStateTracker {
    pub(crate) stream_id: Option<u64>,
    pub(crate) update_clk: Option<Clock>,
    pub(crate) max_latency_ms: Option<u64>,
    pub(crate) unique_id: Option<i32>,
    pub(crate) initial_clock: Option<InitialClock>,
    pub(crate) time_created: chrono::DateTime<chrono::Utc>,
    pub(crate) time_updated: chrono::DateTime<chrono::Utc>,
    pub(crate) market_stream_tracker: MarketStreamTracker,
    pub(crate) order_stream_tracker: OrderStreamTracker,
}

pub(crate) enum Updates<'a> {
    Market(Vec<&'a MarketBookCache>),
    Order(Vec<&'a OrderBookCache>),
}

pub(crate) enum IncomingMessage {
    Market(MarketChangeMessage),
    Order(OrderChangeMessage),
}

pub(crate) struct HasFullImage(pub(crate) bool);

impl StreamStateTracker {
    pub(crate) fn new() -> Self {
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

    pub(crate) fn update_unique_id(&mut self, unique_id: i32) {
        self.unique_id = Some(unique_id);
    }

    pub(crate) fn calculate_updates(&mut self, msg: IncomingMessage) -> Option<Updates<'_>> {
        let change_type = match &msg {
            IncomingMessage::Market(msg) => msg.change_type,
            IncomingMessage::Order(msg) => msg.change_type,
        };

        match change_type {
            Some(ChangeType::SubImage) => self.on_subscribe(msg).0,
            Some(ChangeType::Heartbeat) => self.on_heartbeat(&msg).0,
            Some(ChangeType::ResubDelta) => self.on_resubscribe(msg).0,
            None => self.on_update(msg).0,
        }
    }

    fn on_subscribe(&mut self, msg: IncomingMessage) -> (Option<Updates<'_>>, HasFullImage) {
        self.update_clock_global(&msg);

        let res = match msg {
            IncomingMessage::Market(msg) => self.process_market_change(msg),
            IncomingMessage::Order(msg) => self.process_order_change(msg),
        };

        res
    }

    fn on_resubscribe(&mut self, msg: IncomingMessage) -> (Option<Updates<'_>>, HasFullImage) {
        self.on_update(msg)
    }

    fn on_update(&mut self, msg: IncomingMessage) -> (Option<Updates<'_>>, HasFullImage) {
        if self.update_clk.is_some() {
            self.update_clock_global(&msg);
        }

        let publish_time = match &msg {
            IncomingMessage::Market(msg) => msg.publish_time,
            IncomingMessage::Order(msg) => msg.publish_time,
        };

        if let (Some(publish_time), Some(max_latency_ms)) = (publish_time, self.max_latency_ms) {
            let latency = chrono::Utc::now().signed_duration_since(publish_time);
            if latency.num_milliseconds() > max_latency_ms as i64 {
                tracing::warn!(
                    "High Latency! {:?}ms is greater than max_latency_ms of {:?}ms",
                    latency.num_milliseconds(),
                    max_latency_ms
                );
            }
        }
        let res = match msg {
            IncomingMessage::Market(msg) => self.process_market_change(msg),
            IncomingMessage::Order(msg) => self.process_order_change(msg),
        };

        res
    }

    fn on_heartbeat(&mut self, msg: &IncomingMessage) -> (Option<Updates<'_>>, HasFullImage) {
        self.update_clock_global(msg);
        (None, HasFullImage(false))
    }

    pub(crate) fn clear_stale_cache(&mut self, publish_time: chrono::DateTime<chrono::Utc>) {
        self.market_stream_tracker.clear_stale_cache(publish_time);
        self.order_stream_tracker.clear_stale_cache(publish_time);
    }

    fn update_clock_global(&mut self, msg: &IncomingMessage) {
        match msg {
            IncomingMessage::Market(ref msg) => {
                self.update_clk(msg);
            }
            IncomingMessage::Order(ref msg) => {
                self.update_clk(msg);
            }
        };
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

    fn process_market_change(
        &mut self,
        msg: MarketChangeMessage,
    ) -> (Option<Updates<'_>>, HasFullImage) {
        let res = self.market_stream_tracker.process(msg);
        let updates = res.0.map(Updates::Market);
        (updates, res.1)
    }

    fn process_order_change(
        &mut self,
        msg: OrderChangeMessage,
    ) -> (Option<Updates<'_>>, HasFullImage) {
        let res = self.order_stream_tracker.process(msg);
        let updates = res.0.map(Updates::Order);
        (updates, res.1)
    }
}
