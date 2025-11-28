pub mod check_xovers_parameter;
pub mod suggestion_parameters;

use crate::ensnano_interactor::graphics::{Background3D, HBondDisplay, RenderingMode};
use check_xovers_parameter::CheckXoversParameter;
use ensnano_iced::ui_size::UiSize;
use serde::{Deserialize, Serialize};
use suggestion_parameters::SuggestionParameters;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)] // workaround for https://github.com/rust-cli/confy/issues/34
pub struct AppStateParameters {
    pub suggestion_parameters: SuggestionParameters,
    pub check_xover_parameters: CheckXoversParameter,
    pub follow_stereography: bool,
    pub show_stereography: bool,
    pub rendering_mode: RenderingMode,
    pub background3d: Background3D,
    pub all_helices_on_axis: bool,
    pub scroll_sensitivity: f32,
    pub inverted_y_scroll: bool,
    pub show_h_bonds: HBondDisplay,
    pub show_bezier_paths: bool,
    pub ui_size: UiSize,
}

impl Default for AppStateParameters {
    fn default() -> Self {
        Self {
            suggestion_parameters: Default::default(),
            check_xover_parameters: Default::default(),
            follow_stereography: Default::default(),
            show_stereography: Default::default(),
            rendering_mode: Default::default(),
            background3d: Default::default(),
            all_helices_on_axis: false,
            scroll_sensitivity: 0.0,
            inverted_y_scroll: false,
            show_h_bonds: HBondDisplay::No,
            show_bezier_paths: false,
            ui_size: Default::default(),
        }
    }
}
