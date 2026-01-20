use crate::{
    design::selection::Selection,
    gui::{
        curve::{CurveDescriptorBuilder, RevolutionScaling},
        design_reader::GuiDesignReaderExt,
    },
    utils::operation::CurrentOpState,
};
use ensnano_design::{
    bezier_plane::BezierPathId,
    design_element::DesignElementKey,
    interaction_modes::{ActionMode, SelectionMode},
    organizer_tree::GroupId,
    parameters::HelixParameters,
};
use ensnano_utils::{
    PastingStatus, ScaffoldInfo, SimulationState, StrandBuildingStatus, WidgetBasis,
    app_state_parameters::{
        check_xovers_parameter::CheckXoversParameter, suggestion_parameters::SuggestionParameters,
    },
    clipboard::ClipboardContent,
    graphics::HBondDisplay,
};

pub trait GuiAppState:
    Default + PartialEq + Clone + 'static + Send + std::fmt::Debug + std::fmt::Pointer
{
    const POSSIBLE_CURVES: &'static [CurveDescriptorBuilder<Self>];

    fn get_selection_mode(&self) -> SelectionMode;
    fn get_action_mode(&self) -> ActionMode;
    fn get_build_helix_mode(&self) -> ActionMode;
    fn get_widget_basis(&self) -> WidgetBasis;
    fn get_simulation_state(&self) -> SimulationState;
    fn get_dna_parameters(&self) -> HelixParameters;
    fn is_building_hyperboloid(&self) -> bool;
    fn get_scaffold_info(&self) -> Option<ScaffoldInfo>;
    fn get_selection(&self) -> &[Selection];
    fn get_selection_as_design_element(&self) -> Vec<DesignElementKey>;
    fn can_make_grid(&self) -> bool;
    fn get_reader(&self) -> Box<dyn GuiDesignReaderExt>;
    fn design_was_modified(&self, other: &Self) -> bool;
    fn selection_was_updated(&self, other: &Self) -> bool;
    fn get_current_operation_state(&self) -> Option<CurrentOpState>;
    fn get_strand_building_state(&self) -> Option<StrandBuildingStatus>;
    fn get_selected_group(&self) -> Option<GroupId>;
    fn get_suggestion_parameters(&self) -> &SuggestionParameters;
    fn get_checked_xovers_parameters(&self) -> CheckXoversParameter;
    fn follow_stereographic_camera(&self) -> bool;
    fn show_stereographic_camera(&self) -> bool;
    fn get_h_bonds_display(&self) -> HBondDisplay;
    fn get_scroll_sensitivity(&self) -> f32;
    fn get_invert_y_scroll(&self) -> bool;
    fn want_all_helices_on_axis(&self) -> bool;
    fn expand_insertions(&self) -> bool;
    fn get_show_bezier_paths(&self) -> bool;
    fn get_selected_bezier_path(&self) -> Option<BezierPathId>;
    fn is_exporting(&self) -> bool;
    fn is_transitory(&self) -> bool;
    fn get_current_revolution_radius(&self) -> Option<f64>;
    fn get_recommended_scaling_revolution_surface(
        &self,
        scaffold_len: usize,
    ) -> Option<RevolutionScaling>;
    fn get_clipboard_content(&self) -> ClipboardContent;
    fn get_pasting_status(&self) -> PastingStatus;
}

#[derive(Debug, Clone, Copy)]
pub enum RevolutionParameterId {
    SectionParameter(usize),
    HalfTurnCount,
    RevolutionRadius,
    NbSpiral,
    NbSectionPerSegment,
    ScaffoldLenTarget,
    SpringStiffness,
    TorsionStiffness,
    FluidFriction,
    BallMass,
    TimeSpan,
    SimulationStep,
}
