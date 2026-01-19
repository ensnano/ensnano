use std::path::PathBuf;

use ensnano_design::{
    bezier_plane::BezierVertexId,
    group_attributes::GroupPivot,
    interaction_modes::{ActionMode, SelectionMode},
    organizer_tree::GroupId,
    selection::{CenterOfSelection, Selection},
};
use ensnano_utils::{
    WidgetBasis, app_state_parameters::check_xovers_parameter::CheckXoversParameter,
    strand_builder::StrandBuilder, surfaces::UnrootedRevolutionSurfaceDescriptor,
};

use crate::{design_reader::SceneDesignReaderExt, view::DrawOptions};

pub trait SceneAppState: Clone + 'static {
    type AppStateDesignReader: SceneDesignReaderExt;
    fn get_selection(&self) -> &[Selection];
    fn get_candidates(&self) -> &[Selection];
    fn selection_was_updated(&self, other: &Self) -> bool;
    fn candidates_set_was_updated(&self, other: &Self) -> bool;
    fn design_was_modified(&self, other: &Self) -> bool;
    fn design_model_matrix_was_updated(&self, other: &Self) -> bool;
    fn get_selection_mode(&self) -> SelectionMode;
    fn get_action_mode(&self) -> (ActionMode, WidgetBasis);
    fn get_design_reader(&self) -> Self::AppStateDesignReader;
    fn get_strand_builders(&self) -> &[StrandBuilder];
    fn get_widget_basis(&self) -> WidgetBasis;
    fn is_changing_color(&self) -> bool;
    fn is_pasting(&self) -> bool;
    fn get_selected_element(&self) -> Option<CenterOfSelection>;
    fn get_current_group_pivot(&self) -> Option<GroupPivot>;
    fn get_current_group_id(&self) -> Option<GroupId>;
    fn suggestion_parameters_were_updated(&self, other: &Self) -> bool;
    fn get_check_xover_parameters(&self) -> CheckXoversParameter;
    fn follow_stereographic_camera(&self) -> bool;
    fn get_draw_options(&self) -> DrawOptions;
    fn draw_options_were_updated(&self, other: &Self) -> bool;
    fn get_scroll_sensitivity(&self) -> f32;
    fn show_insertion_discriminants(&self) -> bool;

    fn insertion_bond_display_was_modified(&self, other: &Self) -> bool {
        self.show_insertion_discriminants() != other.show_insertion_discriminants()
    }

    fn show_bezier_paths(&self) -> bool;

    fn get_design_path(&self) -> Option<PathBuf>;

    fn get_selected_bezier_vertex(&self) -> Option<BezierVertexId>;

    fn has_selected_a_bezier_grid(&self) -> bool;

    fn get_revolution_axis_position(&self) -> Option<f64>;
    fn revolution_bezier_updated(&self, other: &Self) -> bool;
    fn get_current_unrooted_surface(&self) -> Option<UnrootedRevolutionSurfaceDescriptor>;
}
