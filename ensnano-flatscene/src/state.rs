use ensnano_design::interaction_modes::SelectionMode;
use ensnano_state::design::selection::{MainDesignReaderExt, Selection};
use ensnano_utils::{StrandBuildingStatus, strand_builder::StrandBuilder};

use crate::design_reader::FlatSceneDesignReaderExt;

pub trait FlatSceneAppState: Clone {
    type Reader: FlatSceneDesignReaderExt + MainDesignReaderExt;

    fn selection_was_updated(&self, other: &Self) -> bool;
    fn candidate_was_updated(&self, other: &Self) -> bool;
    fn get_selection(&self) -> &[Selection];
    fn get_candidates(&self) -> &[Selection];
    fn get_selection_mode(&self) -> SelectionMode;
    fn get_design_reader(&self) -> Self::Reader;
    fn get_strand_builders(&self) -> &[StrandBuilder];
    fn design_was_updated(&self, other: &Self) -> bool;
    fn is_changing_color(&self) -> bool;
    fn is_pasting(&self) -> bool;
    fn get_building_state(&self) -> Option<StrandBuildingStatus>;
}
