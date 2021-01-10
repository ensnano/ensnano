//! The [Gui Manager](gui::Gui) handles redraw request on textures that corresponds to regions
//! attributed to GUI components and events happening on these regions.
//!
//! When a message is emmitted by a Gui component that have consequences that must be forwarded to
//! other components of the program it is forwarded to the [main](main) function via the
//! [Request](Requests) data structure.

/// Draw the top bar of the GUI
pub mod top_bar;
pub use top_bar::TopBar;
/// Draw the left pannel of the GUI
pub mod left_panel;
pub use left_panel::{ColorOverlay, LeftPanel};
pub mod status_bar;
use status_bar::StatusBar;

use crate::mediator::{ActionMode, Operation, SelectionMode};
use crate::SplitMode;
use crate::{DrawArea, ElementType, IcedMessages, Multiplexer};
use iced_native::Event;
use iced_wgpu::{wgpu, Backend, Renderer, Settings, Viewport};
use iced_winit::{conversion, program, winit, Debug, Size};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use ultraviolet::Vec3;
use wgpu::Device;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    window::Window,
};

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
    pub operation_update: Option<Arc<dyn Operation>>,
    pub toggle_persistent_helices: Option<bool>,
    pub new_grid: bool,
    pub camera_rotation: Option<(f32, f32)>,
    pub camera_target: Option<(Vec3, Vec3)>,
    pub small_spheres: Option<bool>,
    pub set_scaffold_id: Option<Option<usize>>,
    pub scaffold_sequence: Option<String>,
    pub stapples_request: bool,
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
            toggle_scene: Some(SplitMode::Both),
            scroll_sensitivity: None,
            make_grids: false,
            overlay_closed: None,
            overlay_opened: None,
            operation_update: None,
            toggle_persistent_helices: None,
            new_grid: false,
            camera_target: None,
            camera_rotation: None,
            small_spheres: None,
            set_scaffold_id: None,
            scaffold_sequence: None,
            stapples_request: false,
        }
    }
}

#[derive(PartialEq)]
pub enum OverlayType {
    Color,
}

enum GuiState {
    TopBar(iced_winit::program::State<TopBar>),
    LeftPanel(iced_winit::program::State<LeftPanel>),
    StatusBar(iced_winit::program::State<StatusBar>),
}

impl GuiState {
    fn queue_event(&mut self, event: Event) {
        match self {
            GuiState::TopBar(state) => state.queue_event(event),
            GuiState::LeftPanel(state) => state.queue_event(event),
            GuiState::StatusBar(state) => state.queue_event(event),
        }
    }

    fn queue_top_bar_message(&mut self, message: top_bar::Message) {
        if let GuiState::TopBar(ref mut state) = self {
            state.queue_message(message)
        } else {
            panic!("wrong message type")
        }
    }

    fn queue_left_panel_message(&mut self, message: left_panel::Message) {
        if let GuiState::LeftPanel(ref mut state) = self {
            state.queue_message(message)
        } else {
            panic!("wrong message type")
        }
    }

    fn queue_status_bar_message(&mut self, message: status_bar::Message) {
        if let GuiState::StatusBar(ref mut state) = self {
            state.queue_message(message)
        } else {
            panic!("wrong message type")
        }
    }

    fn resize(&mut self, area: DrawArea, window: &Window) {
        match self {
            GuiState::TopBar(ref mut state) => state.queue_message(top_bar::Message::Resize(
                area.size.to_logical(window.scale_factor()),
            )),
            GuiState::LeftPanel(ref mut state) => {
                state.queue_message(left_panel::Message::Resized(
                    area.size.to_logical(window.scale_factor()),
                    area.position.to_logical(window.scale_factor()),
                ))
            }
            GuiState::StatusBar(_) => {}
        }
    }

    fn is_queue_empty(&self) -> bool {
        match self {
            GuiState::TopBar(state) => state.is_queue_empty(),
            GuiState::LeftPanel(state) => state.is_queue_empty(),
            GuiState::StatusBar(state) => state.is_queue_empty(),
        }
    }

    fn update(
        &mut self,
        size: iced::Size,
        cursor_position: iced::Point,
        renderer: &mut Renderer,
        debug: &mut Debug,
    ) {
        match self {
            GuiState::TopBar(state) => {
                state.update(size, cursor_position, None, renderer, debug);
            }
            GuiState::LeftPanel(state) => {
                state.update(size, cursor_position, None, renderer, debug);
            }
            GuiState::StatusBar(state) => {
                state.update(size, cursor_position, None, renderer, debug);
            }
        }
    }

    fn render(
        &mut self,
        renderer: &mut Renderer,
        device: &Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        viewport: &iced_graphics::Viewport,
        debug: &Debug,
        mouse_interaction: &mut iced::mouse::Interaction,
    ) {
        match self {
            GuiState::TopBar(ref state) => {
                *mouse_interaction = renderer.backend_mut().draw(
                    device,
                    staging_belt,
                    encoder,
                    target,
                    viewport,
                    state.primitive(),
                    &debug.overlay(),
                );
            }
            GuiState::LeftPanel(ref state) => {
                renderer.backend_mut().draw(
                    device,
                    staging_belt,
                    encoder,
                    target,
                    viewport,
                    state.primitive(),
                    &debug.overlay(),
                );
            }
            GuiState::StatusBar(ref state) => {
                renderer.backend_mut().draw(
                    device,
                    staging_belt,
                    encoder,
                    target,
                    viewport,
                    state.primitive(),
                    &debug.overlay(),
                );
            }
        }
    }
}

/// A Gui component.
struct GuiElement {
    state: GuiState,
    debug: Debug,
    redraw: bool,
    element_type: ElementType,
}

impl GuiElement {
    /// Initialize the top bar gui component
    fn top_bar(
        renderer: &mut Renderer,
        window: &Window,
        multiplexer: &Multiplexer,
        requests: Arc<Mutex<Requests>>,
    ) -> Self {
        let cursor_position = PhysicalPosition::new(-1., -1.);
        let top_bar_area = multiplexer.get_element_area(ElementType::TopBar).unwrap();
        let top_bar = TopBar::new(
            requests.clone(),
            top_bar_area.size.to_logical(window.scale_factor()),
        );
        let mut top_bar_debug = Debug::new();
        let top_bar_state = program::State::new(
            top_bar,
            convert_size(top_bar_area.size),
            conversion::cursor_position(cursor_position, window.scale_factor()),
            renderer,
            &mut top_bar_debug,
        );
        Self {
            state: GuiState::TopBar(top_bar_state),
            debug: top_bar_debug,
            redraw: true,
            element_type: ElementType::TopBar,
        }
    }

    /// Initialize the left panel gui component
    fn left_panel(
        renderer: &mut Renderer,
        window: &Window,
        multiplexer: &Multiplexer,
        requests: Arc<Mutex<Requests>>,
    ) -> Self {
        let cursor_position = PhysicalPosition::new(-1., -1.);
        let left_panel_area = multiplexer
            .get_element_area(ElementType::LeftPanel)
            .unwrap();
        let left_panel = LeftPanel::new(
            requests.clone(),
            left_panel_area.size.to_logical(window.scale_factor()),
            left_panel_area.position.to_logical(window.scale_factor()),
        );
        let mut left_panel_debug = Debug::new();
        let left_panel_state = program::State::new(
            left_panel,
            convert_size(left_panel_area.size),
            conversion::cursor_position(cursor_position, window.scale_factor()),
            renderer,
            &mut left_panel_debug,
        );
        Self {
            state: GuiState::LeftPanel(left_panel_state),
            debug: left_panel_debug,
            redraw: true,
            element_type: ElementType::LeftPanel,
        }
    }

    fn status_bar(
        renderer: &mut Renderer,
        window: &Window,
        multiplexer: &Multiplexer,
        requests: Arc<Mutex<Requests>>,
    ) -> Self {
        let cursor_position = PhysicalPosition::new(-1., -1.);
        let status_bar_area = multiplexer
            .get_element_area(ElementType::StatusBar)
            .unwrap();
        let status_bar = StatusBar::new(requests);
        let mut status_bar_debug = Debug::new();
        let status_bar_state = program::State::new(
            status_bar,
            convert_size(status_bar_area.size),
            conversion::cursor_position(cursor_position, window.scale_factor()),
            renderer,
            &mut status_bar_debug,
        );
        Self {
            state: GuiState::StatusBar(status_bar_state),
            debug: status_bar_debug,
            redraw: true,
            element_type: ElementType::StatusBar,
        }
    }

    fn forward_event(&mut self, event: Event) {
        self.state.queue_event(event)
    }

    fn get_state(&mut self) -> &mut GuiState {
        &mut self.state
    }

    fn resize(&mut self, window: &Window, multiplexer: &Multiplexer) {
        let area = multiplexer.get_draw_area(self.element_type).unwrap();
        self.state.resize(area, window);
        self.redraw = true;
    }

    fn fetch_change(
        &mut self,
        window: &Window,
        multiplexer: &Multiplexer,
        renderer: &mut Renderer,
        resized: bool,
    ) -> bool {
        let area = multiplexer.get_element_area(self.element_type).unwrap();
        let cursor = if multiplexer.foccused_element() == Some(self.element_type) {
            multiplexer.get_cursor_position()
        } else {
            PhysicalPosition::new(-1., -1.)
        };
        if !self.state.is_queue_empty() || resized {
            // We update iced
            self.redraw = true;
            let _ = self.state.update(
                convert_size(area.size),
                conversion::cursor_position(cursor, window.scale_factor()),
                renderer,
                &mut self.debug,
            );
            true
        } else {
            false
        }
    }

    pub fn render(
        &mut self,
        renderer: &mut Renderer,
        encoder: &mut wgpu::CommandEncoder,
        device: &Device,
        window: &Window,
        multiplexer: &Multiplexer,
        staging_belt: &mut wgpu::util::StagingBelt,
        mouse_interaction: &mut iced::mouse::Interaction,
    ) {
        if self.redraw {
            let viewport = Viewport::with_physical_size(
                convert_size_u32(
                    multiplexer
                        .get_element_area(self.element_type)
                        .unwrap()
                        .size,
                ),
                window.scale_factor(),
            );
            let target = multiplexer.get_texture_view(self.element_type).unwrap();
            self.state.render(
                renderer,
                device,
                staging_belt,
                encoder,
                target,
                &viewport,
                &self.debug,
                mouse_interaction,
            );
            self.redraw = false;
        }
    }
}

/// The Gui manager.
pub struct Gui {
    /// HashMap mapping [ElementType](ElementType) to a GuiElement
    elements: HashMap<ElementType, GuiElement>,
    renderer: iced_wgpu::Renderer,
    device: Rc<Device>,
    resized: bool,
}

impl Gui {
    pub fn new(
        device: Rc<Device>,
        window: &Window,
        multiplexer: &Multiplexer,
        requests: Arc<Mutex<Requests>>,
    ) -> Self {
        let mut renderer = Renderer::new(Backend::new(device.as_ref(), Settings::default()));
        let mut elements = HashMap::new();
        elements.insert(
            ElementType::TopBar,
            GuiElement::top_bar(&mut renderer, window, multiplexer, requests.clone()),
        );
        elements.insert(
            ElementType::LeftPanel,
            GuiElement::left_panel(&mut renderer, window, multiplexer, requests.clone()),
        );
        elements.insert(
            ElementType::StatusBar,
            GuiElement::status_bar(&mut renderer, window, multiplexer, requests.clone()),
        );

        Self {
            elements,
            renderer,
            device,
            resized: true,
        }
    }

    /// Forward an event to the appropriate gui component
    pub fn forward_event(&mut self, area: ElementType, event: iced_native::Event) {
        self.elements.get_mut(&area).unwrap().forward_event(event);
    }

    /// Forward a message to the appropriate gui component
    pub fn forward_messages(&mut self, messages: &mut IcedMessages) {
        for m in messages.top_bar.drain(..) {
            self.elements
                .get_mut(&ElementType::TopBar)
                .unwrap()
                .get_state()
                .queue_top_bar_message(m);
        }
        for m in messages.left_panel.drain(..) {
            self.elements
                .get_mut(&ElementType::LeftPanel)
                .unwrap()
                .get_state()
                .queue_left_panel_message(m);
        }
        for m in messages.status_bar.drain(..) {
            self.elements
                .get_mut(&ElementType::StatusBar)
                .unwrap()
                .get_state()
                .queue_status_bar_message(m);
        }
    }

    /// Get the new size of each gui component from the multiplexer and forwards them.
    pub fn resize(&mut self, multiplexer: &Multiplexer, window: &Window) {
        for element in self.elements.values_mut() {
            element.resize(window, multiplexer)
        }
        self.resized = true;
    }

    /// Ask the gui component to process the event that they have recieved
    pub fn fetch_change(&mut self, window: &Window, multiplexer: &Multiplexer) -> bool{
        let mut ret = false;
        for elements in self.elements.values_mut() {
            ret |= elements.fetch_change(window, multiplexer, &mut self.renderer, false);
        }
        ret
    }

    /// Ask the gui component to process the event and messages that they that they have recieved.
    pub fn update(&mut self, multiplexer: &Multiplexer, window: &Window) {
        for elements in self.elements.values_mut() {
            elements.fetch_change(window, multiplexer, &mut self.renderer, self.resized);
        }
        self.resized = false;
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        window: &Window,
        multiplexer: &Multiplexer,
        staging_belt: &mut wgpu::util::StagingBelt,
        mouse_interaction: &mut iced::mouse::Interaction,
    ) {
        for element in self.elements.values_mut() {
            element.render(
                &mut self.renderer,
                encoder,
                self.device.as_ref(),
                window,
                multiplexer,
                staging_belt,
                mouse_interaction,
            )
        }
    }
}

fn convert_size(size: PhysicalSize<u32>) -> Size<f32> {
    Size::new(size.width as f32, size.height as f32)
}

fn convert_size_u32(size: PhysicalSize<u32>) -> Size<u32> {
    Size::new(size.width, size.height)
}
