use ensnano_design::{
    bezier_plane::BezierPathId,
    design_element::DesignElementKey,
    interaction_modes::{ActionMode, SelectionMode},
    organizer_tree::GroupId,
    parameters::HelixParameters,
    selection::Selection,
};
use ensnano_utils::{
    PastingStatus, ScaffoldInfo, SimulationState, StrandBuildingStatus, WidgetBasis,
    app_state_parameters::{
        check_xovers_parameter::CheckXoversParameter, suggestion_parameters::SuggestionParameters,
    },
    clipboard::ClipboardContent,
    graphics::{DrawArea, HBondDisplay},
    operation::CurrentOpState,
};
use iced::{
    Event, Size,
    advanced::{clipboard, mouse, renderer},
    keyboard,
    mouse::Cursor,
};
use iced_runtime::{Debug, program};
use wgpu::{Device, Queue};
use winit::window::Window;

use crate::{
    design_reader::GuiDesignReaderExt,
    left_panel::{
        LeftPanel,
        tabs::revolution_tab::{CurveDescriptorBuilder, RevolutionScaling},
    },
    requests::GuiRequests,
    status_bar::StatusBar,
    top_bar::TopBar,
};

#[expect(clippy::large_enum_variant)]
pub enum GuiState<R: GuiRequests, S: GuiAppState> {
    TopBar(program::State<TopBar<R, S>>),
    LeftPanel(program::State<LeftPanel<R, S>>),
    StatusBar(program::State<StatusBar<R, S>>),
}

impl<R: GuiRequests, S: GuiAppState> GuiState<R, S> {
    pub fn queue_event(&mut self, event: Event) {
        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Tab),
            ..
        }) = event
        {
            match self {
                Self::StatusBar(_) => {
                    self.queue_status_bar_message(crate::status_bar::Message::TabPressed);
                }
                Self::TopBar(_) | Self::LeftPanel(_) => (),
            }
        } else {
            match self {
                Self::TopBar(state) => state.queue_event(event),
                Self::LeftPanel(state) => state.queue_event(event),
                Self::StatusBar(state) => state.queue_event(event),
            }
        }
    }

    pub fn queue_top_bar_message(&mut self, message: crate::top_bar::Message<S>) {
        log::trace!("Queue top bar {message:?}");
        if let Self::TopBar(state) = self {
            state.queue_message(message);
        } else {
            panic!("wrong message type")
        }
    }

    pub fn queue_left_panel_message(&mut self, message: crate::left_panel::Message<S>) {
        log::trace!("Queue left panel {message:?}");
        if let Self::LeftPanel(state) = self {
            state.queue_message(message);
        } else {
            panic!("wrong message type")
        }
    }

    pub fn queue_status_bar_message(&mut self, message: crate::status_bar::Message<S>) {
        log::trace!("Queue status_bar {message:?}");
        if let Self::StatusBar(state) = self {
            state.queue_message(message);
        } else {
            panic!("wrong message type")
        }
    }

    pub fn resize(&mut self, area: DrawArea, window: &Window) {
        match self {
            Self::TopBar(state) => state.queue_message(crate::top_bar::Message::Resize(
                area.size.to_logical(window.scale_factor()),
            )),
            Self::LeftPanel(state) => state.queue_message(crate::left_panel::Message::Resized(
                area.size.to_logical(window.scale_factor()),
                area.position.to_logical(window.scale_factor()),
            )),
            Self::StatusBar(state) => state.queue_message(crate::status_bar::Message::Resize(
                area.size.to_logical(window.scale_factor()),
            )),
        }
    }

    pub fn is_queue_empty(&self) -> bool {
        match self {
            Self::TopBar(state) => state.is_queue_empty(),
            Self::LeftPanel(state) => state.is_queue_empty(),
            Self::StatusBar(state) => state.is_queue_empty(),
        }
    }

    pub fn update(
        &mut self,
        size: Size,
        cursor: Cursor,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        debug: &mut Debug,
    ) {
        let mut clipboard = clipboard::Null;
        match self {
            Self::TopBar(state) => {
                let _ = state.update(size, cursor, renderer, theme, style, &mut clipboard, debug);
            }
            Self::LeftPanel(state) => {
                let _ = state.update(size, cursor, renderer, theme, style, &mut clipboard, debug);
            }
            Self::StatusBar(state) => {
                let _ = state.update(size, cursor, renderer, theme, style, &mut clipboard, debug);
            }
        }
    }

    pub fn render(
        &mut self,
        renderer: &mut iced::Renderer,
        device: &Device,
        queue: &Queue,
        encoder: &mut wgpu::CommandEncoder,
        clear_color: Option<iced::Color>,
        format: wgpu::TextureFormat,
        frame: &wgpu::TextureView,
        viewport: &iced_graphics::Viewport,
        debug: &Debug,
        mouse_interaction: &mut mouse::Interaction,
    ) {
        match renderer {
            iced::Renderer::Wgpu(wgpu_renderer) => {
                wgpu_renderer.with_primitives(|backend, primitives| {
                    backend.present(
                        device,
                        queue,
                        encoder,
                        clear_color,
                        format,
                        frame,
                        primitives,
                        viewport,
                        &debug.overlay(),
                    );
                });
            }
            iced::Renderer::TinySkia(_) => panic!("Unhandled renderer"),
        }

        match self {
            Self::TopBar(state) => *mouse_interaction = state.mouse_interaction(),
            Self::LeftPanel(state) => {
                let icon = state.mouse_interaction();
                if icon > *mouse_interaction {
                    *mouse_interaction = icon;
                }
            }
            Self::StatusBar(state) => {
                let icon = state.mouse_interaction();
                if icon > *mouse_interaction {
                    *mouse_interaction = icon;
                }
            }
        }
    }
}

/// Some main application state, mostly related with top bar buttons.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TopBarState {
    /// Whether the Undo operation is possible.
    pub can_undo: bool,
    /// Whether the Redo operation is possible.
    pub can_redo: bool,
    pub need_save: bool,
    pub can_reload: bool,
    pub can_split_2d: bool,
    pub can_toggle_2d: bool,
    pub is_split_2d: bool,
}

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
