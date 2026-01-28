//! The [GUI Manager](Gui) handles redraw request on textures that corresponds to regions
//! attributed to GUI components and events happening on these regions.
//!
//! When a message is emitted by a GUI component that have consequences that must be forwarded to
//! other components of the program it is forwarded to the `main` function via the
//! [Request](Requests) data structure.

mod color_picker;
mod consts;
pub mod fog;
pub mod fonts;
mod helpers;
pub mod left_panel;
pub mod status_bar;
pub mod theme;
pub mod top_bar;
mod widgets;

use crate::{
    fonts::{INTER_REGULAR_FONT, load_fonts},
    left_panel::LeftPanelState,
    status_bar::StatusBarState,
    top_bar::TopBarState,
};
use ensnano_state::{
    app_state::AppState,
    gui::messages::{
        GuiMessages, LeftPanelMessage, StatusBarMessage, TopBarMessage, TopBarStateFlags,
    },
    multiplexer::Multiplexer,
    requests::Requests,
};
use ensnano_utils::{
    TEXTURE_FORMAT,
    app_state_parameters::AppStateParameters,
    convert_size_f32, convert_size_u32,
    graphics::{DrawArea, GuiComponentType},
    multiplexer_ext::MultiplexerExt,
    ui_size::UiSize,
};
use iced::{
    Event, Size,
    advanced::{clipboard, mouse, renderer},
    keyboard,
    mouse::Cursor,
};
use iced_runtime::{Debug, program};
use iced_wgpu::Backend;
use std::{
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};
use wgpu::{Device, Queue};
use winit::window::Window;

#[expect(clippy::large_enum_variant)]
pub enum GuiState {
    TopBar(program::State<TopBarState>),
    LeftPanel(program::State<LeftPanelState>),
    StatusBar(program::State<StatusBarState>),
}

impl GuiState {
    pub fn queue_event(&mut self, event: Event) {
        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Tab),
            ..
        }) = event
        {
            match self {
                Self::StatusBar(_) => {
                    self.queue_status_bar_message(StatusBarMessage::TabPressed);
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

    pub fn queue_top_bar_message(&mut self, message: TopBarMessage) {
        log::trace!("Queue top bar {message:?}");
        if let Self::TopBar(state) = self {
            state.queue_message(message);
        } else {
            panic!("wrong message type")
        }
    }

    pub fn queue_left_panel_message(&mut self, message: LeftPanelMessage) {
        log::trace!("Queue left panel {message:?}");
        if let Self::LeftPanel(state) = self {
            state.queue_message(message);
        } else {
            panic!("wrong message type")
        }
    }

    pub fn queue_status_bar_message(&mut self, message: StatusBarMessage) {
        log::trace!("Queue status_bar {message:?}");
        if let Self::StatusBar(state) = self {
            state.queue_message(message);
        } else {
            panic!("wrong message type")
        }
    }

    pub fn resize(&mut self, area: DrawArea, window: &Window) {
        match self {
            Self::TopBar(state) => state.queue_message(TopBarMessage::Resize(
                area.size.to_logical(window.scale_factor()),
            )),
            Self::LeftPanel(state) => state.queue_message(LeftPanelMessage::Resized(
                area.size.to_logical(window.scale_factor()),
                area.position.to_logical(window.scale_factor()),
            )),
            Self::StatusBar(state) => state.queue_message(StatusBarMessage::Resize(
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

/// A Gui component.
struct GuiComponent {
    state: GuiState,
    debug: Debug,
    redraw: bool,
    element_type: GuiComponentType,
    renderer: iced::Renderer,
}

impl GuiComponent {
    /// Initialize the top bar gui component
    fn top_bar(
        mut renderer: iced::Renderer,
        window: &Window,
        multiplexer: &dyn MultiplexerExt,
        requests: Arc<Mutex<Requests>>,
        app_state: AppState,
        top_bar_state: TopBarStateFlags,
        ui_size: UiSize,
    ) -> Self {
        let top_bar_area = multiplexer.get_draw_area(GuiComponentType::TopBar).unwrap();
        let top_bar = TopBarState::new(
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
        requests: Arc<Mutex<Requests>>,
        first_time: bool,
        state: &AppState,
        parameters: &AppStateParameters,
    ) -> Self {
        let left_panel_area = multiplexer
            .get_draw_area(GuiComponentType::LeftPanel)
            .unwrap();
        let left_panel = LeftPanelState::new(
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
        requests: Arc<Mutex<Requests>>,
        state: &AppState,
        ui_size: UiSize,
    ) -> Self {
        let status_bar_area = multiplexer
            .get_draw_area(GuiComponentType::StatusBar)
            .unwrap();
        let status_bar = StatusBarState::new(
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

    fn get_state(&mut self) -> &mut GuiState {
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
pub struct GuiManager {
    /// WGPU Settings
    wgpu_settings: iced_wgpu::Settings,
    /// WGPU device
    pub device: Rc<Device>,
    /// WGPU queue
    pub queue: Rc<Queue>,
    resized: bool,
    requests: Arc<Mutex<Requests>>,
    parameters: AppStateParameters,
    /// [`GuiComponent`] mapped by [`GuiComponentType`]
    components: HashMap<GuiComponentType, GuiComponent>,
}

impl GuiManager {
    pub fn new(
        device: Rc<Device>,
        queue: Rc<Queue>,
        window: &Window,
        multiplexer: &dyn MultiplexerExt,
        requests: Arc<Mutex<Requests>>,
        parameters: AppStateParameters,
        global_state: &AppState,
        top_bar_state: TopBarStateFlags,
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
        state: &AppState,
        top_bar_state: TopBarStateFlags,
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
    pub fn forward_messages(&mut self, messages: &mut GuiMessages) {
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
    pub fn resize(&mut self, multiplexer: &Multiplexer, window: &Window) {
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
        multiplexer: &Multiplexer,
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
        app_state: &AppState,
        top_bar_state: TopBarStateFlags,
    ) {
        self.set_text_size(ui_size.main_text());
        self.parameters.ui_size = ui_size;

        self.rebuild_gui(window, multiplexer, app_state, top_bar_state);
    }

    pub fn notify_scale_factor_change(
        &mut self,
        window: &Window,
        multiplexer: &Multiplexer,
        app_state: &AppState,
        top_bar_state: TopBarStateFlags,
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
