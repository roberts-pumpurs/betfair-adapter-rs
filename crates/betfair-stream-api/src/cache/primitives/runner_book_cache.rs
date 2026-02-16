//! Runner book cache (used for market book Stream API caching)

use std::sync::Arc;

use betfair_adapter::betfair_types::numeric::F64Ord;
use betfair_adapter::betfair_types::price::Price;
use betfair_adapter::betfair_types::size::Size;
use betfair_adapter::betfair_types::types::sports_aping::SelectionId;
use betfair_stream_types::response::market_change_message::{RunnerChange, RunnerDefinition};
use betfair_stream_types::response::{UpdateSet2, UpdateSet3};
use eyre::bail;

use super::available_cache::Available;

/// Runner book cache (used for market book Stream API caching)
#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
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
    handicap: Option<F64Ord>,
    definition: Option<Arc<RunnerDefinition>>,
}

impl RunnerBookCache {
    pub fn new_from_runner_change(runner_change: RunnerChange) -> eyre::Result<Self> {
        let Some(id) = runner_change.id else {
            bail!("Invalid selection id");
        };
        let selection_id = id;
        let handicap = runner_change.handicap;
        let definition = None;

        Ok(Self {
            selection_id,
            last_price_traded: runner_change.last_traded_price,
            total_matched: runner_change.total_value,
            traded: runner_change
                .traded
                .map_or_else(|| Available::new(&[]), Available::new),
            available_to_back: runner_change
                .available_to_back
                .map_or_else(|| Available::new(&[]), Available::new),
            best_available_to_back: runner_change
                .best_available_to_back
                .map_or_else(|| Available::new(&[]), Available::new),
            best_display_available_to_back: runner_change
                .best_display_available_to_back
                .map_or_else(|| Available::new(&[]), Available::new),
            available_to_lay: runner_change
                .available_to_lay
                .map_or_else(|| Available::new(&[]), Available::new),
            best_available_to_lay: runner_change
                .best_available_to_lay
                .map_or_else(|| Available::new(&[]), Available::new),
            best_display_available_to_lay: runner_change
                .best_display_available_to_lay
                .map_or_else(|| Available::new(&[]), Available::new),
            starting_price_back: runner_change
                .starting_price_back
                .map_or_else(|| Available::new(&[]), Available::new),
            starting_price_lay: runner_change
                .starting_price_lay
                .map_or_else(|| Available::new(&[]), Available::new),
            starting_price_near: runner_change.starting_price_near,
            starting_price_far: runner_change.starting_price_far,
            handicap,
            definition,
        })
    }

    pub fn new_from_runner_definition(
        runner_definition: Arc<RunnerDefinition>,
    ) -> eyre::Result<Self> {
        let Some(selection_id) = runner_definition.id else {
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
            self.total_matched = Some(Size::zero());
            return;
        }
        self.total_matched = Some(
            traded
                .iter()
                .map(|x| x.1)
                .fold(Size::zero(), |acc, x| acc.saturating_add(&x)),
        );
        self.traded.update(traded);
    }

    pub fn set_definition(&mut self, new_def: RunnerDefinition) {
        match &mut self.definition {
            Some(arc) => match Arc::get_mut(arc) {
                Some(inner) => *inner = new_def,
                None => *arc = Arc::new(new_def),
            },
            None => self.definition = Some(Arc::new(new_def)),
        }
    }

    #[must_use]
    pub const fn total_matched(&self) -> Option<Size> {
        self.total_matched
    }

    #[must_use]
    pub const fn selection_id(&self) -> &SelectionId {
        &self.selection_id
    }

    pub fn set_last_price_traded(&mut self, last_price_traded: Price) {
        self.last_price_traded = Some(last_price_traded);
    }

    pub fn set_total_matched(&mut self, total_matched: Size) {
        self.total_matched = Some(total_matched);
    }

    pub(crate) fn set_starting_price_near(&mut self, spn: Price) {
        self.starting_price_near = Some(spn);
    }

    pub(crate) fn set_starting_price_far(&mut self, spf: Price) {
        self.starting_price_far = Some(spf);
    }

    pub(crate) fn update_available_to_back(&mut self, atb: impl AsRef<[UpdateSet2]>) {
        self.available_to_back.update(atb);
    }

    pub(crate) fn update_available_to_lay(&mut self, atl: impl AsRef<[UpdateSet2]>) {
        self.available_to_lay.update(atl);
    }

    pub(crate) fn update_best_available_to_back(&mut self, batb: impl AsRef<[UpdateSet3]>) {
        self.best_available_to_back.update(batb);
    }

    pub(crate) fn update_best_available_to_lay(&mut self, batl: impl AsRef<[UpdateSet3]>) {
        self.best_available_to_lay.update(batl);
    }

    pub(crate) fn update_best_display_available_to_back(
        &mut self,
        bdatb: impl AsRef<[UpdateSet3]>,
    ) {
        self.best_display_available_to_back.update(bdatb);
    }

    pub(crate) fn update_best_display_available_to_lay(&mut self, bdatl: impl AsRef<[UpdateSet3]>) {
        self.best_display_available_to_lay.update(bdatl);
    }

    pub(crate) fn update_starting_price_back(&mut self, spb: impl AsRef<[UpdateSet2]>) {
        self.starting_price_back.update(spb);
    }

    pub(crate) fn update_starting_price_lay(&mut self, spl: impl AsRef<[UpdateSet2]>) {
        self.starting_price_lay.update(spl);
    }

    #[must_use]
    pub const fn last_price_traded(&self) -> Option<&Price> {
        self.last_price_traded.as_ref()
    }

    #[must_use]
    pub const fn traded(&self) -> &Available<UpdateSet2> {
        &self.traded
    }

    #[must_use]
    pub const fn available_to_back(&self) -> &Available<UpdateSet2> {
        &self.available_to_back
    }

    #[must_use]
    pub const fn best_available_to_back(&self) -> &Available<UpdateSet3> {
        &self.best_available_to_back
    }

    #[must_use]
    pub const fn best_display_available_to_back(&self) -> &Available<UpdateSet3> {
        &self.best_display_available_to_back
    }

    #[must_use]
    pub const fn available_to_lay(&self) -> &Available<UpdateSet2> {
        &self.available_to_lay
    }

    #[must_use]
    pub const fn best_available_to_lay(&self) -> &Available<UpdateSet3> {
        &self.best_available_to_lay
    }

    #[must_use]
    pub const fn best_display_available_to_lay(&self) -> &Available<UpdateSet3> {
        &self.best_display_available_to_lay
    }

    #[must_use]
    pub const fn starting_price_back(&self) -> &Available<UpdateSet2> {
        &self.starting_price_back
    }

    #[must_use]
    pub const fn starting_price_lay(&self) -> &Available<UpdateSet2> {
        &self.starting_price_lay
    }

    #[must_use]
    pub const fn starting_price_near(&self) -> Option<&Price> {
        self.starting_price_near.as_ref()
    }

    #[must_use]
    pub const fn starting_price_far(&self) -> Option<&Price> {
        self.starting_price_far.as_ref()
    }

    #[must_use]
    pub const fn handicap(&self) -> Option<F64Ord> {
        self.handicap
    }

    #[must_use]
    pub fn definition(&self) -> Option<&RunnerDefinition> {
        self.definition.as_deref()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    const fn test_update_traded() {}
}
