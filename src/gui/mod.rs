/// Draw the top bar of the GUI
pub mod top_bar;
pub use top_bar::TopBar;
/// Draw the left pannel of the GUI
pub mod left_panel;
pub use left_panel::{ColorOverlay, LeftPanel};

use crate::mediator::{ActionMode, SelectionMode};
use crate::SplitMode;
use std::path::PathBuf;

/// A structure that contains all the requests that can be made through the GUI.
pub struct Requests {
    /// A change of the rotation mode
    pub action_mode: Option<ActionMode>,
    /// A change of the selection mode
    pub selection_mode: Option<SelectionMode>,
    /// A request to move the camera so that the frustrum fits the desgin
    pub fitting: bool,
    /// A request to load a design into the scene
    pub file_add: Option<PathBuf>,
    /// A request to remove all designs
    pub file_clear: bool,
    /// A request to save the selected design
    pub file_save: Option<PathBuf>,
    /// A request to change the color of the selcted strand
    pub strand_color_change: Option<u32>,
    /// A request to change the sequence of the selected strand
    pub sequence_change: Option<String>,
    /// A request to show/hide the sequences
    pub toggle_text: Option<bool>,
    /// A request to change the view
    pub toggle_scene: Option<SplitMode>,
    /// A request to change the sensitivity of scrolling
    pub scroll_sensitivity: Option<f32>,
    pub make_grids: bool,
    pub overlay_closed: Option<OverlayType>,
    pub overlay_opened: Option<OverlayType>,
}

impl Requests {
    /// Initialise the request structures with no requests
    pub fn new() -> Self {
        Self {
            action_mode: None,
            selection_mode: None,
            fitting: false,
            file_add: None,
            file_clear: false,
            file_save: None,
            strand_color_change: None,
            sequence_change: None,
            toggle_text: None,
            toggle_scene: None,
            scroll_sensitivity: None,
            make_grids: false,
            overlay_closed: None,
            overlay_opened: None,
        }
    }
}

#[derive(PartialEq)]
pub enum OverlayType {
    Color,
}
