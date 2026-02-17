use std::collections::HashMap;
use std::sync::Arc;

use betfair_adapter::betfair_types::types::sports_aping::MarketId;
use betfair_stream_types::response::market_change_message::MarketChangeMessage;

use super::HasFullImage;
use crate::cache::primitives::MarketBookCache;

#[derive(Debug, Clone)]
pub struct MarketStreamTracker {
    market_state: HashMap<MarketId, Arc<MarketBookCache>>,
    updates_processed: u64,
}

impl MarketStreamTracker {
    pub(crate) fn new() -> Self {
        Self {
            market_state: HashMap::with_capacity(64),
            updates_processed: 0,
        }
    }

    pub(crate) fn process(
        &mut self,
        msg: MarketChangeMessage,
    ) -> (Option<Vec<Arc<MarketBookCache>>>, HasFullImage) {
        let mut img = HasFullImage(false);
        let Some(publish_time) = msg.publish_time else {
            tracing::warn!("No publish time in market change message");
            return (None, img);
        };

        if let Some(data) = msg.0.data {
            let mut updated_caches: Vec<Arc<MarketBookCache>> = Vec::with_capacity(data.len());
            let mut market_ids = Vec::with_capacity(data.len());
            for mut market_change in data {
                // Take ownership instead of cloning from the Option
                let Some(market_id) = market_change.market_id.take() else {
                    continue;
                };

                let full_image = market_change.full_image.unwrap_or(false);

                // Single hash lookup via entry API instead of contains_key + insert + get_mut
                let market = match self.market_state.entry(market_id.clone()) {
                    std::collections::hash_map::Entry::Vacant(e) => {
                        img = HasFullImage(true);
                        let id_clone = e.key().clone();
                        e.insert(Arc::new(MarketBookCache::new(id_clone, publish_time)))
                    }
                    std::collections::hash_map::Entry::Occupied(e) => {
                        let m = e.into_mut();
                        if full_image {
                            img = HasFullImage(true);
                            *m = Arc::new(MarketBookCache::new(market_id.clone(), publish_time));
                        }
                        m
                    }
                };

                Arc::make_mut(market).update_cache(market_change, publish_time, true);
                market_ids.push(market_id); // Move, no clone
            }

            for market_id in market_ids {
                let market = self.market_state.get(&market_id);
                let Some(market) = market else {
                    continue;
                };

                updated_caches.push(Arc::clone(market));
                self.updates_processed = self.updates_processed.saturating_add(1);
            }
            return (Some(updated_caches), img);
        };
        (None, img)
    }

    pub(crate) fn clear_stale_cache(&mut self, publish_time: chrono::DateTime<chrono::Utc>) {
        let max_cache_age = chrono::Duration::hours(8);
        self.market_state.retain(|_, cache| {
            !(cache.is_closed()
                && (publish_time.signed_duration_since(cache.publish_time())) > max_cache_age)
        });
    }

    pub fn states(&self) -> Vec<&MarketBookCache> {
        self.market_state.values().map(|arc| arc.as_ref()).collect()
    }
}
