use crate::{app_state::AppState, design::selection::Selection};
use ensnano_utils::StrandBuildingStatus;

impl AppState {
    pub fn get_candidates(&self) -> &[Selection] {
        self.0.candidates.as_slice()
    }

    pub fn candidate_was_updated(&self, other: &Self) -> bool {
        self.0.candidates != other.0.candidates
    }

    pub fn design_was_updated(&self, other: &Self) -> bool {
        self.0.design.has_different_design_than(&other.0.design)
    }

    pub fn get_building_state(&self) -> Option<StrandBuildingStatus> {
        self.get_strand_building_state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ensnano_design::interaction_modes::SelectionMode;

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
