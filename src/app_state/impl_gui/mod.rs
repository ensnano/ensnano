mod curve_builders;

use crate::app_state::{AppState, NewHelixStrand};
use curve_builders::{BEZIER_CURVE_BUILDER, ELLIPSE_BUILDER, TWO_SPHERES_BUILDER};
use ensnano_design::{
    bezier_plane::BezierPathId,
    design_element::DesignElementKey,
    interaction_modes::{ActionMode, SelectionMode},
    organizer_tree::GroupId,
    parameters::HelixParameters,
    selection::{Selection, all_helices_no_grid},
};
use ensnano_gui::{
    design_reader::GuiDesignReaderExt,
    left_panel::tabs::revolution_tab::{CurveDescriptorBuilder, RevolutionScaling},
    state::GuiAppState,
};
use ensnano_utils::{
    PastingStatus, ScaffoldInfo, SimulationState, StrandBuildingStatus, WidgetBasis,
    app_state_parameters::{
        check_xovers_parameter::CheckXoversParameter, suggestion_parameters::SuggestionParameters,
    },
    clipboard::ClipboardContent,
    graphics::HBondDisplay,
    operation::CurrentOpState,
};

impl GuiAppState for AppState {
    const POSSIBLE_CURVES: &'static [CurveDescriptorBuilder<Self>] =
        &[ELLIPSE_BUILDER, TWO_SPHERES_BUILDER, BEZIER_CURVE_BUILDER];

    fn get_selection_mode(&self) -> SelectionMode {
        self.0.selection_mode
    }

    fn get_action_mode(&self) -> ActionMode {
        self.0.action_mode
    }

    fn get_widget_basis(&self) -> WidgetBasis {
        self.0.widget_basis
    }

    fn get_simulation_state(&self) -> SimulationState {
        self.0.design.get_simulation_state()
    }

    fn get_dna_parameters(&self) -> HelixParameters {
        self.0.design.get_dna_parameters()
    }

    fn get_selection(&self) -> &[Selection] {
        self.selection_content().as_ref()
    }

    fn get_selection_as_design_element(&self) -> Vec<DesignElementKey> {
        self.selection_content()
            .iter()
            .filter_map(|s| DesignElementKey::from_selection(s, 0))
            .collect()
    }

    fn is_building_hyperboloid(&self) -> bool {
        self.0.design.is_building_hyperboloid()
    }

    fn get_scaffold_info(&self) -> Option<ScaffoldInfo> {
        self.get_design_interactor().get_scaffold_info()
    }

    fn can_make_grid(&self) -> bool {
        self.selection_content().len() > 4
            && all_helices_no_grid(self.selection_content(), &self.get_design_interactor())
    }

    fn get_reader(&self) -> Box<dyn GuiDesignReaderExt> {
        Box::new(self.get_design_interactor())
    }

    fn design_was_modified(&self, other: &Self) -> bool {
        self.0.design.has_different_design_than(&other.0.design)
    }

    fn selection_was_updated(&self, other: &Self) -> bool {
        self.selection_content() != other.selection_content()
    }

    fn get_build_helix_mode(&self) -> ActionMode {
        if let Some(NewHelixStrand { length, start }) = self.0.strand_on_new_helix.as_ref() {
            ActionMode::BuildHelix {
                position: *start,
                length: *length,
            }
        } else {
            ActionMode::BuildHelix {
                position: 0,
                length: 0,
            }
        }
    }

    fn get_current_operation_state(&self) -> Option<CurrentOpState> {
        self.0.design.get_current_operation_state()
    }

    fn get_strand_building_state(&self) -> Option<StrandBuildingStatus> {
        self.get_strand_building_state()
    }

    fn get_selected_group(&self) -> Option<GroupId> {
        self.0.selection.selected_group
    }

    fn get_suggestion_parameters(&self) -> &SuggestionParameters {
        &self.0.parameters.suggestion_parameters
    }

    fn get_checked_xovers_parameters(&self) -> CheckXoversParameter {
        self.0.parameters.check_xover_parameters
    }

    fn follow_stereographic_camera(&self) -> bool {
        self.0.parameters.follow_stereography
    }

    fn show_stereographic_camera(&self) -> bool {
        self.0.parameters.show_stereography
    }

    fn get_h_bonds_display(&self) -> HBondDisplay {
        self.0.parameters.show_h_bonds
    }

    fn get_scroll_sensitivity(&self) -> f32 {
        self.0.parameters.scroll_sensitivity
    }

    fn get_invert_y_scroll(&self) -> bool {
        self.0.parameters.inverted_y_scroll
    }

    fn want_all_helices_on_axis(&self) -> bool {
        self.0.parameters.all_helices_on_axis
    }

    fn expand_insertions(&self) -> bool {
        !self.0.show_insertion_discriminants
    }

    fn get_show_bezier_paths(&self) -> bool {
        self.0.parameters.show_bezier_paths
    }

    fn get_selected_bezier_path(&self) -> Option<BezierPathId> {
        if let Some(Selection::BezierVertex(vertex)) = self.0.selection.selection.first() {
            Some(vertex.path_id)
        } else {
            None
        }
    }

    fn is_exporting(&self) -> bool {
        self.0.exporting
    }

    fn is_transitory(&self) -> bool {
        !self.is_in_stable_state()
    }

    fn get_current_revolution_radius(&self) -> Option<f64> {
        self.0
            .unrooted_surface
            .descriptor
            .as_ref()?
            .revolution_radius
            .to_signed_f64()
    }

    fn get_recommended_scaling_revolution_surface(
        &self,
        scaffold_len: usize,
    ) -> Option<RevolutionScaling> {
        let area_surface = self.0.unrooted_surface.area?;
        let perimeter_surface = self
            .0
            .unrooted_surface
            .descriptor
            .as_ref()?
            .curve
            .perimeter();
        let helix_parameters = self.get_dna_parameters();
        let area_one_nucl = helix_parameters.rise * helix_parameters.inter_helix_axis_gap();
        let scaling_factor = (scaffold_len as f64 * area_one_nucl as f64 / area_surface).sqrt();
        let scaled_perimeter = scaling_factor * perimeter_surface;

        // We use floor instead of round, because it works better to increase the revolution radius
        // to gain more nucleotide rather than diminishing it.
        let half_number_helix =
            (scaled_perimeter / 2. / HelixParameters::INTER_CENTER_GAP as f64).floor() as usize;

        Some(RevolutionScaling {
            nb_helix: half_number_helix * 2,
        })
    }

    fn get_clipboard_content(&self) -> ClipboardContent {
        self.0.design.get_clipboard_content()
    }

    fn get_pasting_status(&self) -> PastingStatus {
        self.get_pasting_status()
    }
}
