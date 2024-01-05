use std::collections::HashMap;

use betfair_adapter::betfair_types::{types::sports_aping::{MarketId, SelectionId}, size::Size};
use betfair_adapter::rust_decimal;
use betfair_stream_types::response::market_change_message::{
    MarketChange, MarketChangeMessage, MarketDefinition, RunnerChange, RunnerDefinition, StreamMarketDefinitionStatus,
};
use betfair_stream_types::response::{UpdateSet2, UpdateSet3};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use super::available_cache::Available;
use super::runner_book_cache::RunnerBookCache;

pub struct MarketBookCache {
    market_id: MarketId,
    publish_time: DateTime<Utc>,
    active: bool,
    total_matched: Size,
    market_definition: Option<Box<MarketDefinition>>,
    runners: HashMap<(SelectionId, Option<Decimal>), RunnerBookCache>,
}

impl MarketBookCache {
    pub fn new(market_id: MarketId, publish_time: DateTime<Utc>) -> Self {
        Self {
            market_id,
            active: true,
            publish_time,
            market_definition: None,
            total_matched: Size::new(Decimal::ZERO),
            runners: HashMap::new(),
        }
    }

    pub fn is_closed(&self) -> bool {
        !self
            .market_definition
            .as_ref()
            .map(|x| x.status == StreamMarketDefinitionStatus::Open)
            .unwrap_or(false)
    }

    pub fn update_cache(
        &mut self,
        market_change: MarketChange,
        publish_time: DateTime<Utc>,
        active: bool,
    ) {
        self.active = active;
        self.publish_time = publish_time;

        if let Some(market_definition) = market_change.market_definition {
            self.market_definition = Some(market_definition);
        }

        if let Some(tv) = market_change.tv {
            self.total_matched = tv;
        }

        if let Some(rc) = market_change.rc {
            for runner_change in rc {
                let Some(selection_id) = runner_change.id.clone() else {
                    continue;
                };
                let runner = self.runners.get_mut(&(selection_id, runner_change.hc));
                let Some(runner) = runner else {
                    self.add_runner_from_change(runner_change);
                    continue;
                };

                if let Some(ltp) = runner_change.ltp {
                    runner.set_last_price_traded(ltp);
                }
                if let Some(tv) = runner_change.tv {
                    runner.set_total_matched(tv);
                }
                if let Some(spn) = runner_change.spn {
                    runner.set_starting_price_near(spn);
                }
                if let Some(spf) = runner_change.spf {
                    runner.set_starting_price_far(spf);
                }
                if let Some(trd) = runner_change.trd {
                    runner.update_traded(trd.as_slice());
                }
                if let Some(atb) = runner_change.atb {
                    runner.update_available_to_back(atb);
                }
                if let Some(atl) = runner_change.atl {
                    runner.update_available_to_lay(atl);
                }
                if let Some(batb) = runner_change.batb {
                    runner.update_best_available_to_back(batb);
                }
                if let Some(batl) = runner_change.batl {
                    runner.update_best_available_to_lay(batl);
                }
                if let Some(bdatb) = runner_change.bdatb {
                    runner.update_best_display_available_to_back(bdatb);
                }
                if let Some(bdatl) = runner_change.bdatl {
                    runner.update_best_display_available_to_lay(bdatl);
                }
                if let Some(spb) = runner_change.spb {
                    runner.update_starting_price_back(spb);
                }
                if let Some(spl) = runner_change.spl {
                    runner.update_starting_price_lay(spl);
                }
            }
        }

        self.total_matched = self
            .runners
            .values()
            .map(|x| x.total_matched().unwrap_or_else(|| Size::new(Decimal::ZERO)))
            .fold(Size::new(Decimal::ZERO), |acc, x| acc + x);
    }

    fn add_runner_from_change(&mut self, runner_change: RunnerChange) {
        let Some(selection_id) = runner_change.id.clone() else {
            return;
        };
        let key = (selection_id, runner_change.hc);
        let Ok(runner) = RunnerBookCache::new_from_runner_change(runner_change) else {
            return;
        };
        self.runners.insert(key, runner);
    }
    fn add_runner_from_definition(&mut self, runner_definition: RunnerDefinition) {
        let Some(selection_id) = runner_definition.id.clone() else {
            return;
        };
        let key = (selection_id, runner_definition.hc);
        let Ok(runner) = RunnerBookCache::new_from_runner_definition(runner_definition) else {
            return;
        };
        self.runners.insert(key, runner);
    }

    fn update_market_definition(&mut self, market_definition: MarketDefinition) {
        self.market_definition = Some(Box::new(market_definition.clone()));

        for runner_definition in market_definition.runners.into_iter() {
            let selection_id = runner_definition.id.clone();
            let Some(selection_id) = selection_id else {
                continue;
            };
            let hc = runner_definition.hc;
            let key = (selection_id, hc);
            let runner = self.runners.get_mut(&key);
            if let Some(runner) = runner {
                runner.set_definition(runner_definition.clone());
            } else {
                self.add_runner_from_definition(runner_definition);
            }
        }
    }
}
