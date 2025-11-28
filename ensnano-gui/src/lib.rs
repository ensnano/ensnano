//! The [GUI Manager](Gui) handles redraw request on textures that corresponds to regions
//! attributed to GUI components and events happening on these regions.
//!
//! When a message is emitted by a GUI component that have consequences that must be forwarded to
//! other components of the program it is forwarded to the `main` function via the
//! [Request](Requests) data structure.

mod consts;
mod icon;
pub mod left_panel;
pub mod status_bar;
pub mod top_bar;

use crate::left_panel::tabs::revolution_tab::{CurveDescriptorBuilder, RevolutionScaling};
use ensnano_design::{
    CameraId, Nucl,
    bezier_plane::{BezierPathId, BezierVertexId},
    elements::{DesignElement, DesignElementKey, DnaAttribute},
    grid::{GridId, GridTypeDescr},
    parameters::HelixParameters,
};
use ensnano_iced::{
    fonts::{INTER_REGULAR_FONT, load_fonts},
    ui_size::UiSize,
};
use ensnano_interactor::{
    HyperboloidRequest, InsertionPoint, PastingStatus, RapierSimulationRequest, RollRequest,
    ScaffoldInfo, SimulationState, StrandBuildingStatus, WidgetBasis,
    app_state_parameters::{
        AppStateParameters, check_xovers_parameter::CheckXoversParameter,
        suggestion_parameters::SuggestionParameters,
    },
    graphics::{
        Background3D, DrawArea, FogParameters, GuiComponentType, HBondDisplay, RenderingMode,
        SplitMode,
    },
    multiplexer::Multiplexer,
    operation::Operation,
    selection::{ActionMode, Selection, SelectionMode},
    surfaces::{RevolutionSurfaceSystemDescriptor, UnrootedRevolutionSurfaceDescriptor},
};
use ensnano_organizer::tree::{GroupId, OrganizerTree};
use iced::{
    Renderer, Size,
    advanced::{clipboard, mouse, renderer},
    event::Event,
    keyboard,
    mouse::Cursor,
};
use iced_runtime::{Debug, program};
use iced_wgpu::Backend;
use left_panel::{LeftPanel, RigidBodyParametersRequest};
use status_bar::{ClipboardContent, CurrentOpState, StatusBar};
use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    rc::Rc,
    sync::{Arc, Mutex},
};
use top_bar::TopBar;
use ultraviolet::{Rotor3, Vec2, Vec3};
use wgpu::{Device, Queue};
use winit::{dpi::PhysicalSize, event::Modifiers, window::Window};

pub type EnsnTree = OrganizerTree<DesignElementKey>;

pub trait Requests: 'static + Send {
    fn close_overlay(&mut self, overlay_type: OverlayType);
    /// Change the color of the selected strands
    fn change_strand_color(&mut self, color: u32);
    /// Change the background of the 3D scene
    fn change_3d_background(&mut self, bg: Background3D);
    /// Change the rendering mode
    fn change_3d_rendering_mode(&mut self, rendering_mode: RenderingMode);
    /// Set the selected strand as the scaffold
    fn set_scaffold_from_selection(&mut self);
    /// Cancel the current hyperboloid construction
    fn cancel_hyperboloid(&mut self);
    /// Change the scrolling direction
    fn invert_scroll(&mut self, invert: bool);
    /// Resize all the 2D helices, or only the selected ones
    fn resize_2d_helices(&mut self, all: bool);
    /// Make all elements of the design visible
    fn make_all_elements_visible(&mut self);
    /// Toggle the visibility of the selected elements
    fn toggle_visibility(&mut self, visible: bool);
    fn change_action_mode(&mut self, action_mode: ActionMode);
    fn change_selection_mode(&mut self, selection_mode: SelectionMode);
    /// Switch widget basis between world and object
    fn toggle_widget_basis(&mut self);
    /// Show/hide the DNA sequences
    fn set_dna_sequences_visibility(&mut self, visible: bool);
    /// Download the staples as an xlsx file
    fn download_staples(&mut self);
    fn set_scaffold_sequence(&mut self, shift: usize);
    fn set_scaffold_shift(&mut self, shift: usize);
    /// Change the size of the UI components
    fn set_ui_size(&mut self, size: UiSize);
    /// Finalize the currently edited hyperboloid grid
    fn finalize_hyperboloid(&mut self);
    fn stop_roll_simulation(&mut self);
    fn start_roll_simulation(&mut self, roll_request: RollRequest);
    /// Request a Rapier simulation of the current design
    fn request_rapier_simulation(&mut self, request: RapierSimulationRequest);
    /// Make a grid from the set of selected helices
    fn make_grid_from_selection(&mut self);
    /// Start of Update the rigid helices simulation
    fn update_rigid_helices_simulation(&mut self, parameters: RigidBodyParametersRequest);
    /// Start of Update the rigid grids simulation
    fn update_rigid_grids_simulation(&mut self, parameters: RigidBodyParametersRequest);
    fn start_twist_simulation(&mut self, grid_id: GridId);
    /// Update the parameters of the current simulation (rigid grids or helices)
    fn update_rigid_body_simulation_parameters(&mut self, parameters: RigidBodyParametersRequest);
    fn create_new_hyperboloid(&mut self, parameters: HyperboloidRequest);
    /// Update the parameters of the currently edited hyperboloid grid
    fn update_current_hyperboloid(&mut self, parameters: HyperboloidRequest);
    fn update_roll_of_selected_helices(&mut self, roll: f32);
    fn update_scroll_sensitivity(&mut self, sensitivity: f32);
    fn set_fog_parameters(&mut self, parameters: FogParameters);
    /// Set the direction and up vector of the 3D camera
    fn set_camera_dir_up_vec(&mut self, direction: Vec3, up: Vec3);
    fn perform_camera_rotation(&mut self, xz: f32, yz: f32, xy: f32);
    /// Create a new grid in front of the 3D camera
    fn create_grid(&mut self, grid_type_descriptor: GridTypeDescr);
    fn set_candidates_keys(&mut self, candidates: Vec<DesignElementKey>);
    fn set_selected_keys(
        &mut self,
        selection: Vec<DesignElementKey>,
        group_id: Option<GroupId>,
        new_group: bool,
    );
    fn update_organizer_tree(&mut self, tree: OrganizerTree<DesignElementKey>);
    /// Update one attribute of several Dna Elements
    fn update_attribute_of_elements(
        &mut self,
        attribute: DnaAttribute,
        keys: BTreeSet<DesignElementKey>,
    );
    fn change_split_mode(&mut self, split_mode: SplitMode);
    fn export(&mut self, export_type: ensnano_exports::ExportType);
    /// Split/Unsplit the 2D view
    fn toggle_2d_view_split(&mut self);
    fn undo(&mut self);
    fn redo(&mut self);
    /// Display the help message in the contextual panel, regardless of the selection
    fn force_help(&mut self);
    /// Show tutorial in the contextual panel
    fn show_tutorial(&mut self);
    fn new_design(&mut self);
    fn save_as(&mut self);
    fn save(&mut self);
    fn open_file(&mut self);
    /// Adjust the 2D and 3D cameras so that the design fit in screen
    fn fit_design_in_scenes(&mut self);
    /// Update the parameters of the current operation
    fn update_current_operation(&mut self, operation: Arc<dyn Operation>);
    /// Set the scaffold to be the some strand with id `s_id`, or none
    fn set_scaffold_id(&mut self, s_id: Option<usize>);
    /// make the spheres of the currently selected grid large/small
    fn toggle_helices_persistence_of_grid(&mut self, persistent: bool);
    /// make the spheres of the currently selected grid large/small
    fn set_small_sphere(&mut self, small: bool);
    fn finish_changing_color(&mut self);
    fn stop_simulations(&mut self);
    fn reset_simulations(&mut self);
    fn reload_file(&mut self);
    fn add_double_strand_on_new_helix(&mut self, parameters: Option<(isize, usize)>);
    fn set_strand_name(&mut self, s_id: usize, name: String);
    fn create_new_camera(&mut self);
    fn delete_camera(&mut self, camera_id: CameraId);
    fn select_camera(&mut self, camera_id: CameraId);
    fn set_camera_name(&mut self, camera_id: CameraId, name: String);
    fn set_suggestion_parameters(&mut self, param: SuggestionParameters);
    fn set_grid_position(&mut self, grid_id: GridId, position: Vec3);
    fn set_grid_orientation(&mut self, grid_id: GridId, orientation: Rotor3);
    fn toggle_2d(&mut self);
    fn set_nb_turn(&mut self, grid_id: GridId, nb_turn: f32);
    fn set_check_xover_parameters(&mut self, parameters: CheckXoversParameter);
    fn follow_stereographic_camera(&mut self, follow: bool);
    fn set_show_stereographic_camera(&mut self, show: bool);
    fn set_show_h_bonds(&mut self, show: HBondDisplay);
    fn flip_split_views(&mut self);
    fn set_rainbow_scaffold(&mut self, rainbow: bool);
    fn set_all_helices_on_axis(&mut self, off_axis: bool);
    fn align_horizon(&mut self);
    fn download_origamis(&mut self);
    fn set_dna_parameters(&mut self, param: HelixParameters);
    fn set_expand_insertions(&mut self, expand: bool);
    fn set_insertion_length(&mut self, insertion_point: InsertionPoint, length: usize);
    fn create_bezier_plane(&mut self);
    fn turn_path_into_grid(&mut self, path_id: BezierPathId, grid_type: GridTypeDescr);
    fn set_show_bezier_paths(&mut self, show: bool);
    fn make_bezier_path_cyclic(&mut self, path_id: BezierPathId, cyclic: bool);
    fn set_exporting(&mut self, exporting: bool);
    fn import_3d_object(&mut self);
    fn set_position_of_bezier_vertex(&mut self, vertex_id: BezierVertexId, position: Vec2);
    fn optimize_scaffold_shift(&mut self);
    fn start_revolution_relaxation(&mut self, desc: RevolutionSurfaceSystemDescriptor);
    fn finish_revolution_relaxation(&mut self);
    fn load_svg(&mut self);
    fn set_bezier_revolution_radius(&mut self, radius: f64);
    fn set_bezier_revolution_id(&mut self, id: Option<usize>);
    fn set_unrooted_surface(&mut self, surface: Option<UnrootedRevolutionSurfaceDescriptor>);
    /// Make a screenshot of the 2D flatscene.
    fn request_screenshot_2d(&mut self);
    /// Make a screenshot of the 3D scene.
    fn request_screenshot_3d(&mut self);
    fn request_save_nucleotides_positions(&mut self);
    fn notify_revolution_tab(&mut self);
    fn request_stl_export(&mut self);
    /// Set keyboard priority, i.e. whether activate keyboard shortcuts.
    fn set_keyboard_priority(&mut self, priority: bool);
}

#[derive(Clone, Debug, PartialEq)]
pub enum OverlayType {
    Color,
}

#[expect(clippy::large_enum_variant)]
enum GuiState<R: Requests, S: AppState> {
    TopBar(program::State<TopBar<R, S>>),
    LeftPanel(program::State<LeftPanel<R, S>>),
    StatusBar(program::State<StatusBar<R, S>>),
}

impl<R: Requests, S: AppState> GuiState<R, S> {
    fn queue_event(&mut self, event: Event) {
        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Tab),
            ..
        }) = event
        {
            match self {
                Self::StatusBar(_) => {
                    self.queue_status_bar_message(status_bar::Message::TabPressed);
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

    fn queue_top_bar_message(&mut self, message: top_bar::Message<S>) {
        log::trace!("Queue top bar {message:?}");
        if let Self::TopBar(state) = self {
            state.queue_message(message);
        } else {
            panic!("wrong message type")
        }
    }

    fn queue_left_panel_message(&mut self, message: left_panel::Message<S>) {
        log::trace!("Queue left panel {message:?}");
        if let Self::LeftPanel(state) = self {
            state.queue_message(message);
        } else {
            panic!("wrong message type")
        }
    }

    fn queue_status_bar_message(&mut self, message: status_bar::Message<S>) {
        log::trace!("Queue status_bar {message:?}");
        if let Self::StatusBar(state) = self {
            state.queue_message(message);
        } else {
            panic!("wrong message type")
        }
    }

    fn resize(&mut self, area: DrawArea, window: &Window) {
        match self {
            Self::TopBar(state) => state.queue_message(top_bar::Message::Resize(
                area.size.to_logical(window.scale_factor()),
            )),
            Self::LeftPanel(state) => state.queue_message(left_panel::Message::Resized(
                area.size.to_logical(window.scale_factor()),
                area.position.to_logical(window.scale_factor()),
            )),
            Self::StatusBar(state) => state.queue_message(status_bar::Message::Resize(
                area.size.to_logical(window.scale_factor()),
            )),
        }
    }

    fn is_queue_empty(&self) -> bool {
        match self {
            Self::TopBar(state) => state.is_queue_empty(),
            Self::LeftPanel(state) => state.is_queue_empty(),
            Self::StatusBar(state) => state.is_queue_empty(),
        }
    }

    fn update(
        &mut self,
        size: Size,
        cursor: Cursor,
        renderer: &mut Renderer,
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

    fn render(
        &mut self,
        renderer: &mut Renderer,
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
            Renderer::Wgpu(wgpu_renderer) => {
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
            Renderer::TinySkia(_) => panic!("Unhandled renderer"),
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

/// A Gui component.
struct GuiComponent<R: Requests, S: AppState> {
    state: GuiState<R, S>,
    debug: Debug,
    redraw: bool,
    element_type: GuiComponentType,
    renderer: Renderer,
}

impl<R: Requests, S: AppState> GuiComponent<R, S> {
    /// Initialize the top bar gui component
    fn top_bar(
        mut renderer: Renderer,
        window: &Window,
        multiplexer: &dyn Multiplexer,
        requests: Arc<Mutex<R>>,
        app_state: S,
        top_bar_state: TopBarState,
        ui_size: UiSize,
    ) -> Self {
        let top_bar_area = multiplexer.get_draw_area(GuiComponentType::TopBar).unwrap();
        let top_bar = TopBar::new(
            requests,
            top_bar_area.size.to_logical(window.scale_factor()),
            app_state,
            top_bar_state,
            ui_size,
        );
        let mut top_bar_debug = Debug::new();
        let top_bar_state = program::State::new(
            top_bar,
            convert_size(top_bar_area.size),
            &mut renderer,
            &mut top_bar_debug,
        );
        Self {
            state: GuiState::TopBar(top_bar_state),
            debug: top_bar_debug,
            redraw: true,
            element_type: GuiComponentType::TopBar,
            renderer,
        }
    }

    /// Initialize the left panel gui component
    fn left_panel(
        mut renderer: Renderer,
        window: &Window,
        multiplexer: &dyn Multiplexer,
        requests: Arc<Mutex<R>>,
        first_time: bool,
        state: &S,
        parameters: &AppStateParameters,
    ) -> Self {
        let left_panel_area = multiplexer
            .get_draw_area(GuiComponentType::LeftPanel)
            .unwrap();
        let left_panel = LeftPanel::new(
            requests,
            left_panel_area.size.to_logical(window.scale_factor()),
            left_panel_area.position.to_logical(window.scale_factor()),
            first_time,
            state,
            parameters,
        );
        let mut left_panel_debug = Debug::new();
        let left_panel_state = program::State::new(
            left_panel,
            convert_size(left_panel_area.size),
            &mut renderer,
            &mut left_panel_debug,
        );
        Self {
            state: GuiState::LeftPanel(left_panel_state),
            debug: left_panel_debug,
            redraw: true,
            element_type: GuiComponentType::LeftPanel,
            renderer,
        }
    }

    fn status_bar(
        mut renderer: Renderer,
        window: &Window,
        multiplexer: &dyn Multiplexer,
        requests: Arc<Mutex<R>>,
        state: &S,
        ui_size: UiSize,
    ) -> Self {
        let status_bar_area = multiplexer
            .get_draw_area(GuiComponentType::StatusBar)
            .unwrap();
        let status_bar = StatusBar::new(
            requests,
            state,
            status_bar_area.size.to_logical(window.scale_factor()),
            ui_size,
        );
        let mut status_bar_debug = Debug::new();
        let status_bar_state = program::State::new(
            status_bar,
            convert_size(status_bar_area.size),
            &mut renderer,
            &mut status_bar_debug,
        );
        Self {
            state: GuiState::StatusBar(status_bar_state),
            debug: status_bar_debug,
            redraw: true,
            element_type: GuiComponentType::StatusBar,
            renderer,
        }
    }

    fn forward_event(&mut self, event: Event) {
        self.state.queue_event(event);
    }

    fn get_state(&mut self) -> &mut GuiState<R, S> {
        &mut self.state
    }

    fn resize(&mut self, window: &Window, multiplexer: &dyn Multiplexer) {
        let area = multiplexer.get_draw_area(self.element_type).unwrap();
        self.state.resize(area, window);
        log::debug!("resizing {area:?}");
        self.redraw = true;
    }

    fn fetch_change(
        &mut self,
        window: &Window,
        theme: &iced::Theme,
        style: &renderer::Style,
        multiplexer: &dyn Multiplexer,
        resized: bool,
    ) -> bool {
        let area = multiplexer.get_draw_area(self.element_type).unwrap();
        let cursor = if multiplexer.focused_element() == Some(self.element_type) {
            let point = iced_winit::conversion::cursor_position(
                multiplexer.get_cursor_position(),
                window.scale_factor(),
            );
            Cursor::Available(point)
        } else {
            Cursor::Unavailable
        };
        if !self.state.is_queue_empty() || resized {
            // We update iced
            self.redraw = true;
            self.state.update(
                convert_size(area.size),
                cursor,
                &mut self.renderer,
                theme,
                style,
                &mut self.debug,
            );
            true
        } else {
            false
        }
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut wgpu::CommandEncoder,
        clear_color: Option<iced::Color>,
        window: &Window,
        multiplexer: &dyn Multiplexer,
        mouse_interaction: &mut iced::mouse::Interaction,
    ) {
        if self.redraw {
            let viewport = iced_graphics::Viewport::with_physical_size(
                convert_size_u32(multiplexer.get_draw_area(self.element_type).unwrap().size),
                window.scale_factor(),
            );
            let frame = multiplexer.get_texture_view(self.element_type).unwrap();
            self.state.render(
                &mut self.renderer,
                device,
                queue,
                encoder,
                clear_color,
                ensnano_utils::TEXTURE_FORMAT,
                // NOTE: Maybe we should put the format above.
                frame,
                &viewport,
                &self.debug,
                mouse_interaction,
            );
            self.redraw = false;
        }
    }
}

/// The manager of the graphical user interface.
///
/// The manager contains a [`GuiComponent`] for each [`GuiComponentType`] (top_bar, left_panel, etc…)
pub struct Gui<R: Requests, S: AppState> {
    /// WGPU Settings
    wgpu_settings: iced_wgpu::Settings,
    /// WGPU device
    device: Rc<Device>,
    /// WGPU queue
    queue: Rc<Queue>,
    resized: bool,
    requests: Arc<Mutex<R>>,
    parameters: AppStateParameters,
    /// [`GuiComponent`] mapped by [`GuiComponentType`]
    components: HashMap<GuiComponentType, GuiComponent<R, S>>,
}

impl<R: Requests, State: AppState> Gui<R, State> {
    pub fn new(
        device: Rc<Device>,
        queue: Rc<Queue>,
        window: &Window,
        multiplexer: &dyn Multiplexer,
        requests: Arc<Mutex<R>>,
        parameters: AppStateParameters,
        global_state: &State,
        top_bar_state: TopBarState,
    ) -> Self {
        let wgpu_settings = iced_wgpu::Settings {
            antialiasing: Some(iced_graphics::Antialiasing::MSAAx4),
            default_font: INTER_REGULAR_FONT,
            default_text_size: iced::Pixels(parameters.ui_size.main_text()),
            ..Default::default()
        };

        let mut gui = Self {
            wgpu_settings,
            device,
            queue,
            resized: true,
            requests,
            parameters,
            components: HashMap::new(),
        };

        gui.rebuild_gui(window, multiplexer, global_state, top_bar_state);

        gui
    }

    /// Rebuild GUI components.
    ///
    /// Recreate renderers.
    ///
    /// WARN: Attributes device, queue, requests, ui_size, and wgpu_settings must be set
    ///       beforehand.
    ///
    fn rebuild_gui(
        &mut self,
        window: &Window,
        multiplexer: &dyn Multiplexer,
        state: &State,
        top_bar_state: TopBarState,
    ) {
        // NOTE: Wow…
        //       Argument 'state' is called 'global_state' when called above, and it is used
        //       for both left_panel and status_bar.
        //       Type of 'state' is a parameter implementing 'AppState', while top_bar_state
        //       is another type.
        //
        let mut top_bar_renderer = Renderer::Wgpu(iced_wgpu::Renderer::new(
            Backend::new(
                self.device.as_ref(),
                self.queue.as_ref(),
                self.wgpu_settings,
                ensnano_utils::TEXTURE_FORMAT,
            ),
            self.wgpu_settings.default_font,
            self.wgpu_settings.default_text_size,
        ));
        load_fonts(&mut top_bar_renderer);
        self.components.insert(
            GuiComponentType::TopBar,
            GuiComponent::top_bar(
                top_bar_renderer,
                window,
                multiplexer,
                Arc::clone(&self.requests),
                state.clone(),
                top_bar_state,
                self.parameters.ui_size,
            ),
        );

        let mut left_panel_renderer = Renderer::Wgpu(iced_wgpu::Renderer::new(
            Backend::new(
                self.device.as_ref(),
                self.queue.as_ref(),
                self.wgpu_settings,
                ensnano_utils::TEXTURE_FORMAT,
            ),
            self.wgpu_settings.default_font,
            self.wgpu_settings.default_text_size,
        ));
        load_fonts(&mut left_panel_renderer);
        self.components.insert(
            GuiComponentType::LeftPanel,
            GuiComponent::left_panel(
                left_panel_renderer,
                window,
                multiplexer,
                Arc::clone(&self.requests),
                self.components.contains_key(&GuiComponentType::LeftPanel),
                state,
                &self.parameters,
            ),
        );
        self.components.insert(
            GuiComponentType::StatusBar,
            GuiComponent::status_bar(
                Renderer::Wgpu(iced_wgpu::Renderer::new(
                    Backend::new(
                        self.device.as_ref(),
                        self.queue.as_ref(),
                        self.wgpu_settings,
                        ensnano_utils::TEXTURE_FORMAT,
                    ),
                    self.wgpu_settings.default_font,
                    self.wgpu_settings.default_text_size,
                )),
                window,
                multiplexer,
                Arc::clone(&self.requests),
                state,
                self.parameters.ui_size,
            ),
        );
    }

    /// Forward an event to the appropriate gui component
    pub fn forward_event(&mut self, area: GuiComponentType, event: Event) {
        self.components.get_mut(&area).unwrap().forward_event(event);
    }

    /// Clear the focus of all components of the GUI
    pub fn clear_focus(&mut self) {
        for elt in self.components.values_mut() {
            elt.forward_event(Event::Mouse(mouse::Event::CursorMoved {
                position: [-1., -1.].into(),
            }));
            elt.forward_event(Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            )));
        }
    }

    pub fn forward_event_all(&mut self, event: Event) {
        for e in self.components.values_mut() {
            e.forward_event(event.clone());
        }
    }

    /// Forward a message to the appropriate gui component
    pub fn forward_messages(&mut self, messages: &mut IcedMessages<State>) {
        for m in messages.top_bar.drain(..) {
            self.components
                .get_mut(&GuiComponentType::TopBar)
                .unwrap()
                .get_state()
                .queue_top_bar_message(m);
        }
        for m in messages.left_panel.drain(..) {
            self.components
                .get_mut(&GuiComponentType::LeftPanel)
                .unwrap()
                .get_state()
                .queue_left_panel_message(m);
        }
        for m in messages.status_bar.drain(..) {
            self.components
                .get_mut(&GuiComponentType::StatusBar)
                .unwrap()
                .get_state()
                .queue_status_bar_message(m);
        }
    }

    /// Get the new size of each gui component from the multiplexer and forwards them.
    pub fn resize(&mut self, multiplexer: &dyn Multiplexer, window: &Window) {
        for element in self.components.values_mut() {
            element.resize(window, multiplexer);
        }
        self.resized = true;
    }

    /// Ask the gui component to process the event that they have received
    pub fn fetch_change(
        &mut self,
        window: &Window,
        theme: &iced::Theme,
        style: &renderer::Style,
        multiplexer: &dyn Multiplexer,
    ) -> bool {
        let mut ret = false;
        for elements in self.components.values_mut() {
            ret |= elements.fetch_change(window, theme, style, multiplexer, false);
        }
        ret
    }

    /// Ask the gui component to process the event and messages that they have received.
    pub fn update(
        &mut self,
        multiplexer: &dyn Multiplexer,
        theme: &iced::Theme,
        style: &renderer::Style,
        window: &Window,
    ) {
        for elements in self.components.values_mut() {
            elements.fetch_change(window, theme, style, multiplexer, self.resized);
        }
        self.resized = false;
    }

    pub fn new_ui_size(
        &mut self,
        ui_size: UiSize,
        window: &Window,
        multiplexer: &dyn Multiplexer,
        app_state: &State,
        top_bar_state: TopBarState,
    ) {
        self.set_text_size(ui_size.main_text());
        self.parameters.ui_size = ui_size;

        self.rebuild_gui(window, multiplexer, app_state, top_bar_state);
    }

    pub fn notify_scale_factor_change(
        &mut self,
        window: &Window,
        multiplexer: &dyn Multiplexer,
        app_state: &State,
        top_bar_state: TopBarState,
    ) {
        self.set_text_size(self.parameters.ui_size.main_text());
        self.rebuild_gui(window, multiplexer, app_state, top_bar_state);
    }

    fn set_text_size(&mut self, text_size: f32) {
        let settings = iced_wgpu::Settings {
            default_text_size: iced::Pixels(text_size),
            ..self.wgpu_settings
        };
        self.wgpu_settings = settings;
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        clear_color: Option<iced::Color>,
        window: &Window,
        multiplexer: &dyn Multiplexer,
        mouse_interaction: &mut iced::mouse::Interaction,
    ) {
        *mouse_interaction = Default::default();
        for (element_key, element) in &mut self.components {
            log::trace!("render {element_key:?}");
            element.render(
                self.device.as_ref(),
                self.queue.as_ref(),
                encoder,
                clear_color,
                window,
                multiplexer,
                mouse_interaction,
            );
        }
    }
}

// NOTE: It would be nice to implement `From<PhysicalSize>`,
//       but Rust wouldn't allow it for types defined outside
//       the crate.

fn convert_size(size: PhysicalSize<u32>) -> Size<f32> {
    Size::new(size.width as f32, size.height as f32)
}

fn convert_size_u32(size: PhysicalSize<u32>) -> Size<u32> {
    Size::new(size.width, size.height)
}

/// Message sent to the gui component
pub struct IcedMessages<S: AppState> {
    left_panel: VecDeque<left_panel::Message<S>>,
    top_bar: VecDeque<top_bar::Message<S>>,
    status_bar: VecDeque<status_bar::Message<S>>,
    application_state: S,
    last_top_bar_state: TopBarState,
    redraw: bool,
}

impl<S: AppState> IcedMessages<S> {
    pub fn new() -> Self {
        Self {
            left_panel: VecDeque::new(),
            top_bar: VecDeque::new(),
            status_bar: VecDeque::new(),
            application_state: Default::default(),
            last_top_bar_state: Default::default(),
            redraw: false,
        }
    }

    pub fn push_message(&mut self, message: String) {
        self.status_bar
            .push_back(status_bar::Message::Message(Some(message)));
    }

    pub fn push_progress(&mut self, progress_name: String, progress: f32) {
        self.status_bar
            .push_back(status_bar::Message::Progress(Some((
                progress_name,
                progress,
            ))));
    }

    pub fn finish_progress(&mut self) {
        self.status_bar
            .push_back(status_bar::Message::Progress(None));
    }

    pub fn update_modifiers(&mut self, modifiers: Modifiers) {
        self.left_panel
            .push_back(left_panel::Message::ModifiersChanged(modifiers));
    }

    pub fn new_ui_size(&mut self, ui_size: UiSize) {
        self.left_panel
            .push_back(left_panel::Message::UiSizeChanged(ui_size));
        self.top_bar
            .push_back(top_bar::Message::UiSizeChanged(ui_size));
        self.status_bar
            .push_back(status_bar::Message::UiSizeChanged(ui_size));
    }

    pub fn push_show_tutorial(&mut self) {
        self.left_panel.push_back(left_panel::Message::ShowTutorial);
    }

    pub fn show_help(&mut self) {
        self.left_panel.push_back(left_panel::Message::ForceHelp);
    }

    pub fn push_application_state(&mut self, state: S, top_bar_state: TopBarState) {
        log::trace!("Old ptr {:p}, new ptr {:p}", state, self.application_state);
        self.application_state = state.clone();
        self.redraw |= top_bar_state != self.last_top_bar_state;
        self.last_top_bar_state = top_bar_state.clone();
        let must_update = self.application_state != state || self.redraw;
        if must_update {
            self.left_panel
                .push_back(left_panel::Message::NewApplicationState(state.clone()));
            self.top_bar
                .push_back(top_bar::Message::NewApplicationState((
                    state.clone(),
                    top_bar_state,
                )));
            self.status_bar
                .push_back(status_bar::Message::NewApplicationState(state));
        }
    }
}

pub trait AppState:
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

pub trait GuiDesignReaderExt: 'static {
    fn grid_has_persistent_phantom(&self, g_id: GridId) -> bool;
    fn grid_has_small_spheres(&self, g_id: GridId) -> bool;
    fn get_strand_length(&self, s_id: usize) -> Option<usize>;
    fn is_id_of_scaffold(&self, s_id: usize) -> bool;
    fn length_decomposition(&self, s_id: usize) -> String;
    fn nucl_is_anchor(&self, nucl: Nucl) -> bool;
    fn get_dna_elements(&self) -> &[DesignElement];
    fn get_organizer_tree(&self) -> Option<Arc<EnsnTree>>;
    fn strand_name(&self, s_id: usize) -> String;
    fn get_all_cameras(&self) -> Vec<(CameraId, &str)>;
    fn get_grid_position_and_orientation(&self, g_id: GridId) -> Option<(Vec3, Rotor3)>;
    fn get_grid_nb_turn(&self, g_id: GridId) -> Option<f32>;
    fn xover_length(&self, xover_id: usize) -> Option<(f32, Option<f32>)>;
    fn get_id_of_xover_involving_nucl(&self, nucl: Nucl) -> Option<usize>;
    fn rainbow_scaffold(&self) -> bool;
    fn get_insertion_length(&self, selection: &Selection) -> Option<usize>;
    fn get_insertion_point(&self, selection: &Selection) -> Option<InsertionPoint>;
    fn is_bezier_path_cyclic(&self, path_id: BezierPathId) -> Option<bool>;
    fn get_bezier_vertex_position(&self, vertex_id: BezierVertexId) -> Option<Vec2>;
    fn get_scaffold_sequence(&self) -> Option<&str>;
    fn get_current_length_of_relaxed_shape(&self) -> Option<usize>;
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
// NOTE: This was called “MainState”. I am not sure that “TopBarState” is the best name for this.
//       Maybe this would be more like a “GuiState”.
