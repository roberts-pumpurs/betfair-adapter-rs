use std::collections::HashMap;

use betfair_adapter::betfair_types::types::sports_aping::MarketId;
use betfair_stream_types::response::order_change_message::OrderChangeMessage;

use super::HasFullImage;
use crate::cache::primitives::OrderBookCache;

#[derive(Debug)]
pub(crate) struct OrderStreamTracker {
    market_state: HashMap<MarketId, OrderBookCache>,
    updates_processed: u64,
}

impl OrderStreamTracker {
    pub(crate) fn new() -> Self {
        Self {
            market_state: HashMap::new(),
            updates_processed: 0,
        }
    }

    pub(crate) fn process(
        &mut self,
        msg: OrderChangeMessage,
    ) -> (Option<Vec<&OrderBookCache>>, HasFullImage) {
        let mut updated_caches = Vec::new();
        let mut img = HasFullImage(false);
        let Some(publish_time) = msg.publish_time else {
            tracing::warn!("No publish time in market change message");
            return (None, img);
        };

        if let Some(data) = msg.0.data {
            let mut market_ids = Vec::with_capacity(data.len());
            for market_change in data.into_iter() {
                let market_id = market_change.market_id.clone();
                let market = self
                    .market_state
                    .entry(market_id.clone())
                    .or_insert_with(|| {
                        img = HasFullImage(true);
                        OrderBookCache::new(market_id.clone(), publish_time)
                    });

                let full_image = market_change.full_image.unwrap_or(false);
                if full_image {
                    img = HasFullImage(true);
                    *market = OrderBookCache::new(market_id.clone(), publish_time);
                }
                market.update_cache(market_change, publish_time);
                market_ids.push(market_id);
            }

            for market_id in market_ids {
                let market = self.market_state.get(&market_id);
                let Some(market) = market else {
                    continue;
                };

                updated_caches.push(market);
                self.updates_processed += 1;
            }
        }

        if updated_caches.is_empty() {
            (None, img)
        } else {
            (Some(updated_caches), img)
        }
    }

    pub(crate) fn clear_stale_cache(&mut self, publish_time: chrono::DateTime<chrono::Utc>) {
        let max_cache_age = chrono::Duration::hours(8);
        self.market_state
            .retain(|_, v| !(v.is_closed() && (publish_time - v.publish_time()) > max_cache_age));
    }
}
