//! The [GUI Manager](Gui) handles redraw request on textures that corresponds to regions
//! attributed to GUI components and events happening on these regions.
//!
//! When a message is emitted by a GUI component that have consequences that must be forwarded to
//! other components of the program it is forwarded to the `main` function via the
//! [Request](Requests) data structure.

mod color_picker;
mod consts;
pub mod fonts;
mod helpers;
pub mod left_panel;
pub mod messages;
pub mod requests;
pub mod state;
pub mod status_bar;
pub mod theme;
pub mod top_bar;
mod widgets;

use crate::requests::GuiRequests;
use crate::state::{GuiAppState, GuiState, TopBarState};
use crate::{
    fonts::{INTER_REGULAR_FONT, load_fonts},
    left_panel::LeftPanel,
    status_bar::StatusBar,
    top_bar::TopBar,
};
use ensnano_design::{
    CameraId,
    bezier_plane::{BezierPathId, BezierVertexId},
    design_element::DesignElement,
    grid::GridId,
    nucl::Nucl,
    operation::InsertionPoint,
    organizer_tree::OrganizerTree,
    selection::Selection,
};
use ensnano_utils::{
    TEXTURE_FORMAT, app_state_parameters::AppStateParameters, graphics::GuiComponentType,
    multiplexer_ext::MultiplexerExt, ui_size::UiSize,
};
use ensnano_utils::{convert_size_f32, convert_size_u32};
use iced::{
    advanced::{mouse, renderer},
    event::Event,
    mouse::Cursor,
};
use iced_runtime::{Debug, program};
use iced_wgpu::Backend;
use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
    sync::{Arc, Mutex},
};
use ultraviolet::{Rotor3, Vec2, Vec3};
use wgpu::{Device, Queue};
use winit::{event::Modifiers, window::Window};

/// A Gui component.
struct GuiComponent<R: GuiRequests, S: GuiAppState> {
    state: GuiState<R, S>,
    debug: Debug,
    redraw: bool,
    element_type: GuiComponentType,
    renderer: iced::Renderer,
}

impl<R: GuiRequests, S: GuiAppState> GuiComponent<R, S> {
    /// Initialize the top bar gui component
    fn top_bar(
        mut renderer: iced::Renderer,
        window: &Window,
        multiplexer: &dyn MultiplexerExt,
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
            convert_size_f32(top_bar_area.size),
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
        mut renderer: iced::Renderer,
        window: &Window,
        multiplexer: &dyn MultiplexerExt,
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
            convert_size_f32(left_panel_area.size),
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
        mut renderer: iced::Renderer,
        window: &Window,
        multiplexer: &dyn MultiplexerExt,
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
            convert_size_f32(status_bar_area.size),
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

    fn resize(&mut self, window: &Window, multiplexer: &dyn MultiplexerExt) {
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
        multiplexer: &dyn MultiplexerExt,
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
                convert_size_f32(area.size),
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

    pub(crate) fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut wgpu::CommandEncoder,
        clear_color: Option<iced::Color>,
        window: &Window,
        multiplexer: &dyn MultiplexerExt,
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
                TEXTURE_FORMAT,
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
pub struct Gui<R: GuiRequests, S: GuiAppState> {
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

impl<R: GuiRequests, State: GuiAppState> Gui<R, State> {
    pub fn new(
        device: Rc<Device>,
        queue: Rc<Queue>,
        window: &Window,
        multiplexer: &dyn MultiplexerExt,
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
        multiplexer: &dyn MultiplexerExt,
        state: &State,
        top_bar_state: TopBarState,
    ) {
        // NOTE: Wow…
        //       Argument 'state' is called 'global_state' when called above, and it is used
        //       for both left_panel and status_bar.
        //       Type of 'state' is a parameter implementing 'AppState', while top_bar_state
        //       is another type.
        //
        let mut top_bar_renderer = iced::Renderer::Wgpu(iced_wgpu::Renderer::new(
            Backend::new(
                self.device.as_ref(),
                self.queue.as_ref(),
                self.wgpu_settings,
                TEXTURE_FORMAT,
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

        let mut left_panel_renderer = iced::Renderer::Wgpu(iced_wgpu::Renderer::new(
            Backend::new(
                self.device.as_ref(),
                self.queue.as_ref(),
                self.wgpu_settings,
                TEXTURE_FORMAT,
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
                iced::Renderer::Wgpu(iced_wgpu::Renderer::new(
                    Backend::new(
                        self.device.as_ref(),
                        self.queue.as_ref(),
                        self.wgpu_settings,
                        TEXTURE_FORMAT,
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
    pub fn resize(&mut self, multiplexer: &dyn MultiplexerExt, window: &Window) {
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
        multiplexer: &dyn MultiplexerExt,
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
        multiplexer: &dyn MultiplexerExt,
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
        multiplexer: &dyn MultiplexerExt,
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
        multiplexer: &dyn MultiplexerExt,
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
        multiplexer: &dyn MultiplexerExt,
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

/// Message sent to the gui component
pub struct IcedMessages<S: GuiAppState> {
    left_panel: VecDeque<left_panel::Message<S>>,
    top_bar: VecDeque<top_bar::Message<S>>,
    status_bar: VecDeque<status_bar::Message<S>>,
    application_state: S,
    last_top_bar_state: TopBarState,
    redraw: bool,
}

impl<S: GuiAppState> IcedMessages<S> {
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

pub trait GuiDesignReaderExt: 'static {
    fn grid_has_persistent_phantom(&self, g_id: GridId) -> bool;
    fn grid_has_small_spheres(&self, g_id: GridId) -> bool;
    fn get_strand_length(&self, s_id: usize) -> Option<usize>;
    fn is_id_of_scaffold(&self, s_id: usize) -> bool;
    fn length_decomposition(&self, s_id: usize) -> String;
    fn nucl_is_anchor(&self, nucl: Nucl) -> bool;
    fn get_dna_elements(&self) -> &[DesignElement];
    fn get_organizer_tree(&self) -> Option<Arc<OrganizerTree>>;
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
