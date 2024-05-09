use std::collections::HashMap;

use betfair_adapter::betfair_types::size::Size;
use betfair_adapter::betfair_types::types::sports_aping::{MarketId, SelectionId};
use betfair_adapter::rust_decimal;
use betfair_stream_types::response::market_change_message::{
    MarketChange, MarketDefinition, RunnerChange, RunnerDefinition, StreamMarketDefinitionStatus,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use super::runner_book_cache::RunnerBookCache;

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
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

        if let Some(tv) = market_change.total_value {
            self.total_matched = tv;
        }

        let mut calculate_total_matched = false;
        if let Some(rc) = market_change.runner_change {
            for runner_change in rc {
                let Some(selection_id) = runner_change.id.clone() else {
                    continue;
                };
                let runner = self
                    .runners
                    .get_mut(&(selection_id, runner_change.handicap));
                let Some(runner) = runner else {
                    self.add_runner_from_change(runner_change);
                    continue;
                };

                if let Some(ltp) = runner_change.last_traded_price {
                    runner.set_last_price_traded(ltp);
                }
                if let Some(tv) = runner_change.total_value {
                    runner.set_total_matched(tv);
                }
                if let Some(spn) = runner_change.starting_price_near {
                    runner.set_starting_price_near(spn);
                }
                if let Some(spf) = runner_change.starting_price_far {
                    runner.set_starting_price_far(spf);
                }
                if let Some(trd) = runner_change.traded {
                    runner.update_traded(trd.as_slice());
                    calculate_total_matched = true;
                }
                if let Some(atb) = runner_change.available_to_back {
                    runner.update_available_to_back(atb);
                }
                if let Some(atl) = runner_change.available_to_lay {
                    runner.update_available_to_lay(atl);
                }
                if let Some(batb) = runner_change.best_available_to_back {
                    runner.update_best_available_to_back(batb);
                }
                if let Some(batl) = runner_change.best_available_to_lay {
                    runner.update_best_available_to_lay(batl);
                }
                if let Some(bdatb) = runner_change.best_display_available_to_back {
                    runner.update_best_display_available_to_back(bdatb);
                }
                if let Some(bdatl) = runner_change.best_display_available_to_lay {
                    runner.update_best_display_available_to_lay(bdatl);
                }
                if let Some(spb) = runner_change.starting_price_back {
                    runner.update_starting_price_back(spb);
                }
                if let Some(spl) = runner_change.starting_price_lay {
                    runner.update_starting_price_lay(spl);
                }
            }
        }

        if calculate_total_matched {
            self.total_matched = self
                .runners
                .values()
                .map(|x| {
                    x.total_matched()
                        .unwrap_or_else(|| Size::new(Decimal::ZERO))
                })
                .fold(Size::new(Decimal::ZERO), |acc, x| acc + x);
        }
    }

    pub fn update_market_definition(&mut self, market_definition: MarketDefinition) {
        self.market_definition = Some(Box::new(market_definition.clone()));

        for runner_definition in market_definition.runners.into_iter() {
            let selection_id = runner_definition.id.clone();
            let Some(selection_id) = selection_id else {
                continue;
            };
            let hc = runner_definition.handicap;
            let key = (selection_id, hc);
            let runner = self.runners.get_mut(&key);
            if let Some(runner) = runner {
                runner.set_definition(runner_definition.clone());
            } else {
                self.add_runner_from_definition(runner_definition);
            }
        }
    }

    fn add_runner_from_change(&mut self, runner_change: RunnerChange) {
        let Some(selection_id) = runner_change.id.clone() else {
            return;
        };
        let key = (selection_id, runner_change.handicap);
        let Ok(runner) = RunnerBookCache::new_from_runner_change(runner_change) else {
            return;
        };
        self.runners.insert(key, runner);
    }
    fn add_runner_from_definition(&mut self, runner_definition: RunnerDefinition) {
        let Some(selection_id) = runner_definition.id.clone() else {
            return;
        };
        let key = (selection_id, runner_definition.handicap);
        let Ok(runner) = RunnerBookCache::new_from_runner_definition(runner_definition) else {
            return;
        };
        self.runners.insert(key, runner);
    }

    pub fn publish_time(&self) -> DateTime<Utc> {
        self.publish_time
    }

    pub fn runners(&self) -> &HashMap<(SelectionId, Option<Decimal>), RunnerBookCache> {
        &self.runners
    }

    pub fn market_definition(&self) -> Option<&Box<MarketDefinition>> {
        self.market_definition.as_ref()
    }

    pub fn market_id(&self) -> &MarketId {
        &self.market_id
    }
}

#[cfg(test)]
mod tests {
    use betfair_adapter::betfair_types::price::Price;
    use betfair_stream_types::response::market_change_message::MarketChangeMessage;
    use betfair_stream_types::response::UpdateSet2;
    use rust_decimal_macros::dec;

    use super::*;
    use crate::cache::primitives::available_cache::Available;

    fn init() -> (MarketId, DateTime<Utc>, MarketBookCache) {
        let market_id = MarketId("1.23456789".to_string());
        let publish_time = Utc::now();
        let market_book_cache = MarketBookCache::new(market_id.clone(), publish_time);
        (market_id, publish_time, market_book_cache)
    }

    #[test]
    fn test_init() {
        let (market_id, publish_time, market_book_cache) = init();

        assert!(market_book_cache.active);
        assert_eq!(market_book_cache.market_id, market_id);
        assert_eq!(market_book_cache.publish_time, publish_time);
        assert_eq!(market_book_cache.total_matched, Size::new(Decimal::ZERO));
        assert_eq!(market_book_cache.market_definition, None);
        assert!(market_book_cache.runners.is_empty());
    }

    #[test]
    fn test_update_mc() {
        let data = r#"{"op":"mcm","id":12345,"clk":"AKEIANcNANkP","pt":1478717720756,"mc":[{"id":"1.128149474","marketDefinition":{"bspMarket":false,"turnInPlayEnabled":true,"persistenceEnabled":true,"marketBaseRate":5,"eventId":"28009395","eventTypeId":"2","numberOfWinners":1,"bettingType":"ODDS","marketType":"GAME_BY_GAME_01_07","marketTime":"2016-11-09T18:15:00.000Z","suspendTime":"2016-11-09T18:15:00.000Z","bspReconciled":false,"complete":true,"inPlay":true,"crossMatching":true,"runnersVoidable":false,"numberOfActiveRunners":2,"betDelay":5,"status":"SUSPENDED","runners":[{"status":"ACTIVE","sortPriority":1,"id":4520808},{"status":"ACTIVE","sortPriority":2,"id":7431682}],"regulators":["MR_INT"],"countryCode":"CO","discountAllowed":true,"timezone":"UTC","openDate":"2016-11-09T18:15:00.000Z","version":1488624717}}]}"#;
        let market_change_message: MarketChangeMessage = serde_json::from_str(data).unwrap();
        let market_change = market_change_message.data.as_ref().unwrap();
        let mut init = init().2;

        for change in market_change {
            init.update_cache(change.clone(), Utc::now(), true);
            assert!(init.active);
            assert_eq!(init.total_matched, change.total_value.unwrap_or_default());
        }
    }

    #[test]
    fn test_update_tv() {
        let data = r#"{"op":"mcm","id":2,"clk":"AHMAcArtjjje","pt":1471370160471,"mc":[{"id":"1.126235656","tv":69.69}]}"#;
        let market_change_message: MarketChangeMessage = serde_json::from_str(data).unwrap();
        let market_change = market_change_message.data.as_ref().unwrap();
        let mut init = init().2;

        for change in market_change {
            init.update_cache(change.clone(), Utc::now(), true);
            assert!(init.active);
            assert_eq!(init.total_matched, change.total_value.unwrap_or_default());
        }
    }

    #[test]
    fn test_update_multiple_rc() {
        // update data with multiple rc entries for the same selection
        let (market_id, _, mut init) = init();
        let data = vec![
            RunnerChange {
                available_to_back: Some(vec![UpdateSet2(
                    Price::new(dec!(1.01)).unwrap(),
                    Size::new(dec!(200)),
                )]),
                id: Some(SelectionId(13536143)),
                ..Default::default()
            },
            RunnerChange {
                available_to_lay: Some(vec![UpdateSet2(
                    Price::new(dec!(1.02)).unwrap(),
                    Size::new(dec!(200)),
                )]),
                id: Some(SelectionId(13536143)),
                ..Default::default()
            },
        ];
        let market_change = MarketChange {
            market_id: Some(market_id),
            runner_change: Some(data),
            ..Default::default()
        };

        init.update_cache(market_change, Utc::now(), true);
        assert!(init.active);
        assert_eq!(init.runners.len(), 1);
        // assert tv not changed
        assert_eq!(init.total_matched, Size::new(Decimal::ZERO));
        // assert atb updated
        assert_eq!(
            init.runners
                .get(&(SelectionId(13536143), None))
                .unwrap()
                .available_to_back(),
            &Available::new([UpdateSet2(
                Price::new(dec!(1.01)).unwrap(),
                Size::new(dec!(200))
            )])
        );
        // assert atl updated
        assert_eq!(
            init.runners
                .get(&(SelectionId(13536143), None))
                .unwrap()
                .available_to_lay(),
            &Available::new([UpdateSet2(
                Price::new(dec!(1.02)).unwrap(),
                Size::new(dec!(200))
            )])
        );
    }

    #[test]
    fn test_update_market_definition() {
        let mock_market_definition = MarketDefinition {
            bet_delay: 1,
            version: 234,
            complete: true,
            runners_voidable: false,
            status: StreamMarketDefinitionStatus::Open,
            bsp_reconciled: true,
            cross_matching: false,
            in_play: true,
            number_of_winners: 5,
            number_of_active_runners: 6,
            ..Default::default()
        };
        let (_, _, mut init) = init();

        init.update_market_definition(mock_market_definition.clone());

        assert_eq!(
            init.market_definition,
            Some(Box::new(mock_market_definition))
        );
    }

    #[test]
    fn test_update_runner_cache_tv() {
        let (market_id, _, mut init) = init();

        {
            let market_change = MarketChange {
                market_id: Some(market_id.clone()),
                runner_change: Some(vec![RunnerChange {
                    total_value: Some(Size::new(dec!(123.0))),
                    id: Some(SelectionId(13536143)),
                    ..Default::default()
                }]),
                ..Default::default()
            };
            init.update_cache(market_change.clone(), Utc::now(), true);
            assert_eq!(
                init.runners.iter().next().unwrap().1.total_matched(),
                Some(Size::new(dec!(123.0)))
            );
        }
        {
            let market_change = MarketChange {
                market_id: Some(market_id.clone()),
                runner_change: Some(vec![RunnerChange {
                    traded: Some(vec![]),
                    id: Some(SelectionId(13536143)),
                    ..Default::default()
                }]),
                ..Default::default()
            };
            init.update_cache(market_change.clone(), Utc::now(), true);
            assert_eq!(
                init.runners.iter().next().unwrap().1.total_matched(),
                Some(Size::new(Decimal::ZERO))
            );
        }
        {
            let market_change = MarketChange {
                market_id: Some(market_id.clone()),
                runner_change: Some(vec![RunnerChange {
                    traded: Some(vec![UpdateSet2(
                        Price::new(dec!(12.0)).unwrap(),
                        Size::new(dec!(2.0)),
                    )]),
                    id: Some(SelectionId(13536143)),
                    ..Default::default()
                }]),
                ..Default::default()
            };
            init.update_cache(market_change, Utc::now(), true);
            assert_eq!(
                init.runners.iter().next().unwrap().1.total_matched(),
                Some(Size::new(dec!(2.0)))
            );
        }
    }

    #[test]
    fn test_update_market_cache_tv() {
        let (market_id, _, mut init) = init();

        {
            let market_change = MarketChange {
                market_id: Some(market_id.clone()),
                runner_change: Some(vec![RunnerChange {
                    total_value: Some(Size::new(dec!(123.0))),
                    id: Some(SelectionId(13536143)),
                    ..Default::default()
                }]),
                ..Default::default()
            };
            init.update_cache(market_change.clone(), Utc::now(), true);
            assert_eq!(init.total_matched, Size::new(Decimal::ZERO));
        }
        {
            let market_change = MarketChange {
                market_id: Some(market_id.clone()),
                runner_change: Some(vec![RunnerChange {
                    traded: Some(vec![UpdateSet2(
                        Price::new(dec!(12.0)).unwrap(),
                        Size::new(dec!(2.0)),
                    )]),
                    id: Some(SelectionId(13536143)),
                    ..Default::default()
                }]),
                ..Default::default()
            };
            init.update_cache(market_change, Utc::now(), true);
            assert_eq!(init.total_matched, Size::new(dec!(2.0)));
        }
    }
}
