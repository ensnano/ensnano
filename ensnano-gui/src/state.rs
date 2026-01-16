use ensnano_utils::graphics::DrawArea;
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
    GuiAppState, left_panel::LeftPanel, requests::GuiRequests, status_bar::StatusBar,
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
