use crate::app_state::{AppState, design_interactor::DesignInteractor};
use crate::ensnano_flatscene::AppState as App2D;
use crate::ensnano_interactor::{
    selection::{Selection, SelectionMode},
    strand_builder::StrandBuilder,
};

impl App2D for AppState {
    type Reader = DesignInteractor;
    fn get_selection(&self) -> &[Selection] {
        self.selection_content().as_slice()
    }

    fn get_candidates(&self) -> &[Selection] {
        self.0.candidates.as_slice()
    }

    fn selection_was_updated(&self, other: &Self) -> bool {
        self.selection_content() != other.selection_content()
    }

    fn candidate_was_updated(&self, other: &Self) -> bool {
        self.0.candidates != other.0.candidates
    }

    fn get_selection_mode(&self) -> SelectionMode {
        self.0.selection_mode
    }

    fn get_design_reader(&self) -> Self::Reader {
        self.0.design.clone_inner()
    }

    fn get_strand_builders(&self) -> &[StrandBuilder] {
        self.0.design.get_strand_builders()
    }

    fn design_was_updated(&self, other: &Self) -> bool {
        self.0.design.has_different_design_than(&other.0.design)
    }

    fn is_changing_color(&self) -> bool {
        self.is_changing_color()
    }

    fn is_pasting(&self) -> bool {
        self.get_pasting_status().is_pasting()
    }

    fn get_building_state(&self) -> Option<crate::ensnano_interactor::StrandBuildingStatus> {
        self.get_strand_building_state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_update() {
        let mut state = AppState::default();
        let old_state = state.clone();

        // When a new state is created with this methods it should be considered to have a new
        // selection but the same selection
        state = state.with_selection(vec![Selection::Strand(0, 0)], None);
        assert!(state.selection_was_updated(&old_state));
    }

    #[test]
    fn selection_mode_update() {
        let mut state = AppState::default();
        let old_selection_mode = state.get_selection_mode();
        let old_state = state.clone();
        state = state.with_selection_mode(SelectionMode::Helix);
        assert_eq!(old_state.get_selection_mode(), old_selection_mode);
        assert_eq!(state.get_selection_mode(), SelectionMode::Helix);
        assert!(!state.selection_was_updated(&old_state));
    }
}
