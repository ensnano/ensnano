use crate::app_state::{AppState, design_interactor::DesignInteractor};
use crate::design::selection::Selection;
use ensnano_design::interaction_modes::SelectionMode;
use ensnano_utils::{StrandBuildingStatus, strand_builder::StrandBuilder};

impl AppState {
    pub fn get_candidates(&self) -> &[Selection] {
        self.0.candidates.as_slice()
    }

    pub fn selection_was_updated(&self, other: &Self) -> bool {
        self.selection_content() != other.selection_content()
    }

    pub fn candidate_was_updated(&self, other: &Self) -> bool {
        self.0.candidates != other.0.candidates
    }

    pub fn get_selection_mode(&self) -> SelectionMode {
        self.0.selection_mode
    }

    pub fn get_design_reader(&self) -> DesignInteractor {
        self.0.design.clone_inner()
    }

    pub fn get_strand_builders(&self) -> &[StrandBuilder] {
        self.0.design.get_strand_builders()
    }

    pub fn design_was_updated(&self, other: &Self) -> bool {
        self.0.design.has_different_design_than(&other.0.design)
    }

    pub fn is_pasting(&self) -> bool {
        self.get_pasting_status().is_pasting()
    }

    pub fn get_building_state(&self) -> Option<StrandBuildingStatus> {
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
