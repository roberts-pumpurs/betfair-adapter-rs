use std::collections::HashMap;

use betfair_adapter::betfair_types::{size::Size, price::Price};
use betfair_adapter::betfair_types::types::sports_aping::SelectionId;
use betfair_adapter::rust_decimal;
use betfair_stream_types::response::market_change_message::{RunnerChange, RunnerDefinition};
use betfair_stream_types::response::{UpdateSet2, UpdateSet3};
use eyre::bail;
use rust_decimal::Decimal;

use super::available_cache::Available;

pub struct RunnerBookCache {
    selection_id: SelectionId,
    last_price_traded: Option<Price>,
    total_matched: Option<Size>,
    traded: Available<UpdateSet2>,
    available_to_back: Available<UpdateSet2>,
    best_available_to_back: Available<UpdateSet3>,
    best_display_available_to_back: Available<UpdateSet3>,
    available_to_lay: Available<UpdateSet2>,
    best_available_to_lay: Available<UpdateSet3>,
    best_display_available_to_lay: Available<UpdateSet3>,
    starting_price_back: Available<UpdateSet2>,
    starting_price_lay: Available<UpdateSet2>,
    starting_price_near: Option<Price>,
    starting_price_far: Option<Price>,
    handicap: Option<Decimal>,
    definition: Option<RunnerDefinition>,
}

impl RunnerBookCache {
    pub fn new_from_runner_change(runner_change: RunnerChange) -> eyre::Result<Self> {
        let Some(id) = runner_change.id else {
            bail!("Invalid selection id");
        };
        let selection_id = id;
        let handicap = runner_change.hc;
        let definition = None;

        Ok(Self {
            selection_id,
            last_price_traded: runner_change.ltp,
            total_matched: runner_change.tv,
            traded: runner_change
                .trd
                .map(|x| Available::new(x))
                .unwrap_or_else(|| Available::new(&[])),
            available_to_back: runner_change
                .atb
                .map(|x| Available::new(x))
                .unwrap_or_else(|| Available::new(&[])),
            best_available_to_back: runner_change
                .batb
                .map(|x| Available::new(x))
                .unwrap_or_else(|| Available::new(&[])),
            best_display_available_to_back: runner_change
                .bdatb
                .map(|x| Available::new(x))
                .unwrap_or_else(|| Available::new(&[])),
            available_to_lay: runner_change
                .atl
                .map(|x| Available::new(x))
                .unwrap_or_else(|| Available::new(&[])),
            best_available_to_lay: runner_change
                .batl
                .map(|x| Available::new(x))
                .unwrap_or_else(|| Available::new(&[])),
            best_display_available_to_lay: runner_change
                .bdatl
                .map(|x| Available::new(x))
                .unwrap_or_else(|| Available::new(&[])),
            starting_price_back: runner_change
                .spb
                .map(|x| Available::new(x))
                .unwrap_or_else(|| Available::new(&[])),
            starting_price_lay: runner_change
                .spl
                .map(|x| Available::new(x))
                .unwrap_or_else(|| Available::new(&[])),
            starting_price_near: runner_change.spn,
            starting_price_far: runner_change.spf,
            handicap,
            definition,
        })
    }

    pub fn new_from_runner_definition(runner_definition: RunnerDefinition) -> eyre::Result<Self> {
        let Some(selection_id) = runner_definition.id.clone() else {
            bail!("Invalid selection id");
        };
        let definition = Some(runner_definition);

        Ok(Self {
            selection_id,
            last_price_traded: None,
            total_matched: None,
            traded: Available::new(&[]),
            available_to_back: Available::new(&[]),
            best_available_to_back: Available::new(&[]),
            best_display_available_to_back: Available::new(&[]),
            available_to_lay: Available::new(&[]),
            best_available_to_lay: Available::new(&[]),
            best_display_available_to_lay: Available::new(&[]),
            starting_price_back: Available::new(&[]),
            starting_price_lay: Available::new(&[]),
            starting_price_near: None,
            starting_price_far: None,
            handicap: None,
            definition,
        })
    }

    pub fn update_traded(&mut self, traded: &[UpdateSet2]) {
        if traded.is_empty() {
            self.traded.clear();
            return;
        }
        self.total_matched = Some(
            traded
                .iter()
                .map(|x| x.1)
                .fold(Size::new(Decimal::ZERO), |acc, x| acc + x),
        );
        self.traded.update(traded);
    }

    pub fn set_definition(&mut self, definition: RunnerDefinition) {
        self.definition = Some(definition);
    }

    pub fn total_matched(&self) -> Option<Size> {
        self.total_matched
    }

    pub fn selection_id(&self) -> &SelectionId {
        &self.selection_id
    }

    pub fn set_last_price_traded(&mut self, last_price_traded: Price) {
        self.last_price_traded = Some(last_price_traded);
    }

    pub fn set_total_matched(&mut self, total_matched: Size) {
        self.total_matched = Some(total_matched);
    }

    pub(crate) fn set_starting_price_near(&mut self, spn: Price)  {
        self.starting_price_near = Some(spn);
    }

    pub(crate) fn set_starting_price_far(&mut self, spf: Price)  {
        self.starting_price_far = Some(spf);
    }

    pub(crate) fn update_available_to_back(&mut self, atb: impl AsRef<[UpdateSet2]>)  {
        self.available_to_back.update(atb);
    }

    pub(crate) fn update_available_to_lay(&mut self, atl: impl AsRef<[UpdateSet2]>)  {
        self.available_to_lay.update(atl);
    }

    pub(crate) fn update_best_available_to_back(&mut self, batb: impl AsRef<[UpdateSet3]>)  {
        self.best_available_to_back.update(batb);
    }

    pub(crate) fn update_best_available_to_lay(&mut self, batl: impl AsRef<[UpdateSet3]>)  {
        self.best_available_to_lay.update(batl);
    }

    pub(crate) fn update_best_display_available_to_back(&mut self, bdatb: impl AsRef<[UpdateSet3]>)  {
        self.best_display_available_to_back.update(bdatb);
    }

    pub(crate) fn update_best_display_available_to_lay(&mut self, bdatl: impl AsRef<[UpdateSet3]>)  {
        self.best_display_available_to_lay.update(bdatl);
    }

    pub(crate) fn update_starting_price_back(&mut self, spb: impl AsRef<[UpdateSet2]>)  {
        self.starting_price_back.update(spb);
    }

    pub(crate) fn update_starting_price_lay(&mut self, spl: impl AsRef<[UpdateSet2]>)  {
        self.starting_price_lay.update(spl);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_traded() {}
}
