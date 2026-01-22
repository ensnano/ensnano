use crate::multiplexer::Multiplexer;
use ensnano_gui::left_panel::ColorOverlay;
use ensnano_state::{gui::messages::GuiMessages, requests::Requests};
use ensnano_utils::{
    convert_size_f32, convert_size_u32, graphics::GuiComponentType, overlay::OverlayType,
};
use iced::{
    advanced::{clipboard, renderer},
    mouse::Cursor,
};
use iced_graphics::Viewport;
use iced_runtime::{Debug, program};
use std::sync::{Arc, Mutex};
use winit::{dpi::PhysicalSize, window::Window};

pub(crate) struct OverlayManager {
    color_state: program::State<ColorOverlay>,
    color_debug: Debug,
    overlay_types: Vec<OverlayType>,
}

impl OverlayManager {
    pub(crate) fn new(
        requests: Arc<Mutex<Requests>>,
        window: &Window,
        renderer: &mut iced::Renderer,
    ) -> Self {
        let color = ColorOverlay::new(
            requests,
            PhysicalSize::new(250., 250.).to_logical(window.scale_factor()),
        );
        let mut color_debug = Debug::new();
        let color_state = program::State::new(
            color,
            convert_size_f32(PhysicalSize::new(250, 250)),
            renderer,
            &mut color_debug,
        );

        Self {
            color_state,
            color_debug,
            overlay_types: Vec::new(),
        }
    }

    pub(crate) fn forward_event(&mut self, event: iced::Event, n: usize) {
        match self.overlay_types.get(n) {
            None => {
                log::error!("receive event from non existing overlay");
                unreachable!();
            }
            Some(OverlayType::Color) => self.color_state.queue_event(event),
        }
    }

    pub(crate) fn process_event(
        &mut self,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        resized: bool,
        multiplexer: &Multiplexer,
        window: &Window,
    ) {
        for (n, overlay) in self.overlay_types.iter().enumerate() {
            let cursor = if multiplexer.focused_element() == Some(GuiComponentType::Overlay(n)) {
                let point = iced_winit::conversion::cursor_position(
                    multiplexer.get_cursor_position(),
                    window.scale_factor(),
                );
                Cursor::Available(point)
            } else {
                Cursor::Unavailable
            };
            let mut clipboard = clipboard::Null;
            match overlay {
                OverlayType::Color => {
                    if !self.color_state.is_queue_empty() || resized {
                        let _ = self.color_state.update(
                            convert_size_f32(PhysicalSize::new(250, 250)),
                            cursor,
                            renderer,
                            theme,
                            style,
                            &mut clipboard,
                            &mut self.color_debug,
                        );
                    }
                }
            }
        }
    }

    pub(crate) fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        format: wgpu::TextureFormat,
        target: &wgpu::TextureView,
        multiplexer: &Multiplexer,
        window: &Window,
        renderer: &mut iced::Renderer,
    ) {
        for overlay_type in &self.overlay_types {
            match overlay_type {
                OverlayType::Color => {
                    let color_viewport = Viewport::with_physical_size(
                        convert_size_u32(multiplexer.window_size),
                        window.scale_factor(),
                    );
                    match renderer {
                        iced::Renderer::Wgpu(wgpu_renderer) => {
                            wgpu_renderer.with_primitives(|backend, primitives| {
                                backend.present(
                                    device,
                                    queue,
                                    encoder,
                                    None, // TODO: Examine what clear_color is.
                                    format,
                                    target,
                                    primitives,
                                    &color_viewport,
                                    &self.color_debug.overlay(),
                                );
                            });
                        }
                        iced::Renderer::TinySkia(_) => unreachable!(),
                    }
                }
            }
        }
    }

    pub(crate) fn forward_messages(&self, _messages: &mut GuiMessages) {}

    pub(crate) fn fetch_change(
        &mut self,
        multiplexer: &Multiplexer,
        window: &Window,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
    ) -> bool {
        let mut ret = false;
        for (n, overlay) in self.overlay_types.iter().enumerate() {
            let cursor = if multiplexer.focused_element() == Some(GuiComponentType::Overlay(n)) {
                let point = iced_winit::conversion::cursor_position(
                    multiplexer.get_cursor_position(),
                    window.scale_factor(),
                );
                Cursor::Available(point)
            } else {
                Cursor::Unavailable
            };
            let mut clipboard = clipboard::Null;
            match overlay {
                OverlayType::Color => {
                    if !self.color_state.is_queue_empty() {
                        ret = true;
                        let _ = self.color_state.update(
                            convert_size_f32(PhysicalSize::new(250, 250)),
                            cursor,
                            renderer,
                            theme,
                            style,
                            &mut clipboard,
                            &mut self.color_debug,
                        );
                    }
                }
            }
        }
        ret
    }
}
