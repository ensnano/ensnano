/*
ENSnano, a 3d graphical application for DNA nanostructures.
    Copyright (C) 2021  Nicolas Levy <nicolaspierrelevy@gmail.com> and Nicolas Schabanel <nicolas.schabanel@ens-lyon.fr>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
//! This module handles the different regions of the window.
//!
//! The [layout_manager] splits the window into different regions and attribute each region to an
//! an application or a GUI component.
//!
//! In addition, the multiplexer holds a [Vec] of [overlays](Overlay), which are floating regions.
//!
//! When an event is received by the window, the multiplexer is in charge of forwarding it to the
//! appropriate application, GUI component, or overlay. The multiplexer also handles some events
//! directly, like resizing events or keyboard input that should be handled independently of the
//! focused region.
//!
//!
//!
//! The multiplexer is also in charge of drawing to the frame.

pub mod layout_manager;

use crate::ensnano_interactor::{
    ActionMode, SelectionMode,
    graphics::{DrawArea, GuiComponentType, SplitMode},
};
use crate::ensnano_utils::texture::SampledTexture;
use crate::{PhySize, controller::normal_state::Action, requests::Requests};
use ensnano_iced::{
    UiSize,
    iced_wgpu::wgpu,
    iced_wgpu::wgpu::Device,
    iced_winit::winit::{
        dpi::{PhysicalPosition, PhysicalSize},
        event::{ElementState, KeyEvent, Modifiers, WindowEvent},
        keyboard::{Key, KeyLocation, ModifiersState, NamedKey},
        window::{CursorIcon, Window},
    },
};
use layout_manager::{LayoutTree, PixelRegion};
use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

/// A structure that handles the division of the window into different `DrawArea`.
///
///
pub struct Multiplexer {
    /// The *physical* size of the window.
    pub window_size: PhySize,
    /// The scale factor of the window.
    pub scale_factor: f64,
    /// The object mapping pixels to drawing areas.
    layout: LayoutTree,
    /// The element on which the mouse cursor is currently on.
    focus: Option<GuiComponentType>,
    /// The *physical* position of the cursor on the focus area.
    cursor_position: PhysicalPosition<f64>,
    /// The area that are drawn on top of the application.
    overlays: Vec<Overlay>,
    /// The texture on which the scene is rendered.
    scene_texture: Option<MultiplexerTexture>,
    /// The texture on which the top bar gui is rendered.
    top_bar_texture: Option<MultiplexerTexture>,
    /// The texture on which the left panel is rendered.
    left_panel_texture: Option<MultiplexerTexture>,
    /// The textures on which the overlays are rendered.
    overlays_textures: Vec<MultiplexerTexture>,
    /// The texture on which the grid is rendered.
    grid_panel_texture: Option<MultiplexerTexture>,
    /// The texture on which the stereographic scene is rendered.
    stereographic_scene_texture: Option<MultiplexerTexture>,
    /// The texture on which the status bar gui is rendered.
    status_bar_texture: Option<MultiplexerTexture>,
    /// The texture on which the flat scene is rendered.
    flat_scene_texture: Option<MultiplexerTexture>,
    /// The pointer to the node that separate the top bar from the scene.
    top_bar_split: usize,
    /// The pointer to the node that separate the status bar from the scene.
    status_bar_split: usize,
    /// The WGPU device.
    device: Rc<Device>,
    /// The WGPU pipeline.
    pipeline: Option<wgpu::RenderPipeline>,
    //. 3D/Flat scene split mode.
    split_mode: SplitMode,
    requests: Arc<Mutex<Requests>>,
    state: State,
    modifiers_state: ModifiersState,
    ui_size: UiSize,
    pub icon: Option<CursorIcon>,
    element_3d: GuiComponentType,
    element_2d: GuiComponentType,
}

/// Maximum width of the left panel.
const MAX_LEFT_PANEL_WIDTH: f64 = 200.;
/// Maximum height of the status bar.
const MAX_STATUS_BAR_HEIGHT: f64 = 56.;

impl Multiplexer {
    /// Create a new multiplexer for a window with size `window_size`.
    ///
    /// Immediately creates a _top bar_, then a _left panel_, then a _status bar_. The remaining
    /// area is called the _scene._ It looks like this:
    ///
    /// ```text
    ///     ┌───────────────────────────┐
    ///     │          top bar          │
    ///     ├───────────────────────────┤
    ///     │┌────────┬────────────────┐│
    ///     ││        │┌──────────────┐││
    ///     ││  left  ││              │││
    ///     ││  panel ││     scene    │││
    ///     ││        ││              │││
    ///     ││        │├──────────────┤││
    ///     ││        ││  status bar  │││
    ///     ││        │└──────────────┘││
    ///     │└────────┴────────────────┘│
    ///     └───────────────────────────┘
    /// ```
    ///
    pub fn new(
        window_size: PhySize,
        scale_factor: f64,
        device: Rc<Device>,
        requests: Arc<Mutex<Requests>>,
        ui_size: UiSize,
    ) -> Self {
        let mut layout = LayoutTree::new();
        let (width, height) = (window_size.width as f64, window_size.height as f64);

        // The top bar are.
        let top_bar_proportion = ui_size.top_bar_height() * scale_factor / height;
        let top_bar_split = 0;
        let (top_bar, scene) = layout.hsplit(0, top_bar_proportion, false);

        // The left panel area.
        let left_panel_proportion = (MAX_LEFT_PANEL_WIDTH * scale_factor / width).max(0.2);
        let (left_panel, scene) = layout.vsplit(scene, left_panel_proportion, true);

        // The status bar area.
        let scene_height = (1. - top_bar_proportion) * height;
        let status_bar_proportion = MAX_STATUS_BAR_HEIGHT * scale_factor / scene_height;
        let status_bar_split = scene;
        let (scene, status_bar) = layout.hsplit(scene, 1. - status_bar_proportion, false);

        // Add GUI component types to areas.
        layout.attribute_element(top_bar, GuiComponentType::TopBar);
        layout.attribute_element(scene, GuiComponentType::Scene);
        layout.attribute_element(status_bar, GuiComponentType::StatusBar);
        layout.attribute_element(left_panel, GuiComponentType::LeftPanel);

        let mut ret = Self {
            window_size,
            scale_factor,
            layout,
            focus: None,
            cursor_position: PhysicalPosition::new(-1., -1.),
            scene_texture: None,
            flat_scene_texture: None,
            top_bar_texture: None,
            left_panel_texture: None,
            grid_panel_texture: None,
            status_bar_texture: None,
            stereographic_scene_texture: None,
            overlays: Vec::new(),
            overlays_textures: Vec::new(),
            device,
            pipeline: None,
            split_mode: SplitMode::Scene3D,
            requests,
            status_bar_split,
            top_bar_split,
            state: State::Normal {
                mouse_position: PhysicalPosition::new(-1., -1.),
            },
            modifiers_state: ModifiersState::empty(),
            ui_size,
            icon: None,
            element_2d: GuiComponentType::FlatScene,
            element_3d: GuiComponentType::Scene,
        };
        ret.generate_textures();
        ret
    }

    /// Return a view of the texture on which the element must be rendered
    pub fn get_texture_view(&self, element_type: GuiComponentType) -> Option<&wgpu::TextureView> {
        match element_type {
            GuiComponentType::StereographicScene => self
                .stereographic_scene_texture
                .as_ref()
                .map(|t| &t.texture.view),
            GuiComponentType::Scene => self.scene_texture.as_ref().map(|t| &t.texture.view),
            GuiComponentType::LeftPanel => {
                self.left_panel_texture.as_ref().map(|t| &t.texture.view)
            }
            GuiComponentType::TopBar => self.top_bar_texture.as_ref().map(|t| &t.texture.view),
            GuiComponentType::Overlay(n) => Some(&self.overlays_textures[n].texture.view),
            GuiComponentType::GridPanel => {
                self.grid_panel_texture.as_ref().map(|t| &t.texture.view)
            }
            GuiComponentType::FlatScene => {
                self.flat_scene_texture.as_ref().map(|t| &t.texture.view)
            }
            GuiComponentType::StatusBar => {
                self.status_bar_texture.as_ref().map(|t| &t.texture.view)
            }
            GuiComponentType::Unattributed => unreachable!(),
        }
    }

    fn get_texture_size(&self, element_type: GuiComponentType) -> Option<DrawArea> {
        match element_type {
            GuiComponentType::Scene => self.scene_texture.as_ref().map(|t| t.area),
            GuiComponentType::LeftPanel => self.left_panel_texture.as_ref().map(|t| t.area),
            GuiComponentType::TopBar => self.top_bar_texture.as_ref().map(|t| t.area),
            GuiComponentType::Overlay(n) => Some(self.overlays_textures[n].area),
            GuiComponentType::GridPanel => self.grid_panel_texture.as_ref().map(|t| t.area),
            GuiComponentType::FlatScene => self.flat_scene_texture.as_ref().map(|t| t.area),
            GuiComponentType::StatusBar => self.status_bar_texture.as_ref().map(|t| t.area),
            GuiComponentType::StereographicScene => {
                self.stereographic_scene_texture.as_ref().map(|t| t.area)
            }
            GuiComponentType::Unattributed => unreachable!(),
        }
    }

    pub fn update_modifiers(&mut self, modifiers: Modifiers) {
        self.modifiers_state = modifiers.state()
    }

    pub fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        window: &crate::Window,
    ) {
        if self.pipeline.is_none() {
            let bg_layout = &self.top_bar_texture.as_ref().unwrap().texture.bg_layout;
            self.pipeline = Some(create_pipeline(self.device.as_ref(), bg_layout));
        }

        let msaa_texture = None;

        let attachment = if msaa_texture.is_some() {
            msaa_texture.as_ref().unwrap()
        } else {
            target
        };

        let resolve_target = if msaa_texture.is_some() {
            Some(target)
        } else {
            None
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Multiplexer render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: attachment,
                resolve_target,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        if self.window_size.width > 0 && self.window_size.height > 0 {
            for element in [
                GuiComponentType::TopBar,
                GuiComponentType::LeftPanel,
                GuiComponentType::GridPanel,
                GuiComponentType::Scene,
                GuiComponentType::FlatScene,
                GuiComponentType::StereographicScene,
                GuiComponentType::StatusBar,
            ]
            .iter()
            {
                log::debug!("Draw {:?}", element);
                if let Some(area) = self.get_texture_size(*element) {
                    render_pass.set_bind_group(0, self.get_bind_group(element), &[]);

                    render_pass.set_viewport(
                        area.position.x as f32,
                        area.position.y as f32,
                        area.size.width as f32,
                        area.size.height as f32,
                        0.0,
                        1.0,
                    );
                    let width = area
                        .size
                        .width
                        .min(window.inner_size().width - area.position.x);
                    let height = area
                        .size
                        .height
                        .min(window.inner_size().height - area.position.y);
                    render_pass.set_scissor_rect(area.position.x, area.position.y, width, height);
                    render_pass.set_pipeline(self.pipeline.as_ref().unwrap());
                    render_pass.draw(0..4, 0..1);
                }
            }
        }
    }

    fn get_bind_group(&self, element_type: &GuiComponentType) -> &wgpu::BindGroup {
        match element_type {
            GuiComponentType::TopBar => &self.top_bar_texture.as_ref().unwrap().texture.bind_group,
            GuiComponentType::LeftPanel => {
                &self.left_panel_texture.as_ref().unwrap().texture.bind_group
            }
            GuiComponentType::Scene => &self.scene_texture.as_ref().unwrap().texture.bind_group,
            GuiComponentType::FlatScene => {
                &self.flat_scene_texture.as_ref().unwrap().texture.bind_group
            }
            GuiComponentType::GridPanel => {
                &self.grid_panel_texture.as_ref().unwrap().texture.bind_group
            }
            GuiComponentType::Overlay(n) => &self.overlays_textures[*n].texture.bind_group,
            GuiComponentType::StatusBar => {
                &self.status_bar_texture.as_ref().unwrap().texture.bind_group
            }
            GuiComponentType::StereographicScene => {
                &self
                    .stereographic_scene_texture
                    .as_ref()
                    .unwrap()
                    .texture
                    .bind_group
            }
            GuiComponentType::Unattributed => unreachable!(),
        }
    }

    /// Return the drawing area attributed to an element.
    pub fn get_draw_area(&self, element_type: GuiComponentType) -> Option<DrawArea> {
        let (position, size) = if let GuiComponentType::Overlay(n) = element_type {
            (self.overlays[n].position, self.overlays[n].size)
        } else {
            let (left, top, right, bottom) = self.layout.get_area(element_type)?;
            let top = (top * self.window_size.height as f64).round();
            let left = (left * self.window_size.width as f64).round();
            let bottom = (bottom * self.window_size.height as f64).round();
            let right = (right * self.window_size.width as f64).round();

            // WARN: There can be floating point issue here: `top`, `left`, `bottom`, and `right`
            //       are proportions, e.g., values between 0 and 1, stored as f64; they are
            //       multiplied by the window size and casted to the u32 type. If the rounding is
            //       not well handled, there can be few missing pixels, of few pixels too much —
            //       which make the soft crash with a “Viewport has invalid rect” message.
            //
            // NOTE: I tried to naively solve the problem by adding `.round()`, but the ideal
            //       solution would be to distribute the pixels.

            (
                PhysicalPosition::new(left, top).cast::<u32>(),
                PhysicalSize::new(right - left, bottom - top).cast::<u32>(),
            )
        };
        Some(DrawArea { position, size })
    }

    pub fn check_scale_factor(&mut self, window: &crate::Window) -> bool {
        if self.scale_factor != window.scale_factor() {
            self.scale_factor = window.scale_factor();
            self.window_size = window.inner_size();
            self.resize(self.window_size, self.scale_factor);

            if self.window_size.width > 0 && self.window_size.height > 0 {
                self.generate_textures();
            }
            true
        } else {
            false
        }
    }

    /// Forwards event to the element on which they happen.
    pub fn event(
        &mut self,
        mut event: WindowEvent,
        resized: &mut bool,
        scale_factor_changed: &mut bool,
    ) -> Option<(WindowEvent, GuiComponentType)> {
        let mut captured = false;
        match &mut event {
            WindowEvent::CursorMoved { position, .. } => match &mut self.state {
                State::Resizing {
                    region,
                    mouse_position,
                    clicked_position,
                    old_proportion,
                } => {
                    *mouse_position = *position;
                    let mut position = position.clone();
                    position.x /= self.window_size.width as f64;
                    position.y /= self.window_size.height as f64;
                    *resized = true;
                    self.layout.resize_click(
                        *region,
                        &position,
                        &clicked_position,
                        *old_proportion,
                    );
                    self.icon = Some(CursorIcon::EwResize);
                    captured = true;
                }

                State::Normal { mouse_position, .. } => {
                    *mouse_position = *position;
                    let &mut PhysicalPosition { x, y } = position;
                    if x > 0.0 || y > 0.0 {
                        let element = self.pixel_to_element(*position);
                        let area = match element {
                            PixelRegion::Resize(_) => {
                                self.icon = Some(CursorIcon::EwResize);
                                None
                            }
                            PixelRegion::Element(element) => {
                                self.icon = None;
                                self.focus = Some(element);
                                self.get_draw_area(element)
                            }
                            PixelRegion::Area(_) => unreachable!(),
                        }
                        .or(self.focus.and_then(|e| self.get_draw_area(e)));

                        if let Some(area) = area {
                            self.cursor_position.x = position.x - area.position.cast::<f64>().x;
                            self.cursor_position.y = position.y - area.position.cast::<f64>().y;
                        }
                    }
                }
                State::Interacting {
                    mouse_position,
                    element,
                } => {
                    *mouse_position = *position;
                    let element = element.clone();
                    let area = self.get_draw_area(element);
                    if let Some(area) = area {
                        self.cursor_position.x = position.x - area.position.cast::<f64>().x;
                        self.cursor_position.y = position.y - area.position.cast::<f64>().y;
                    }
                }
            },
            WindowEvent::Resized(new_size) => {
                self.window_size = *new_size;
                self.resize(*new_size, self.scale_factor);
                *resized = true;
                if self.window_size.width > 0 && self.window_size.height > 0 {
                    self.generate_textures();
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = *scale_factor;
                //self.window_size = **new_inner_size;
                //TODO: The WindowEvent used to provide [new_inner_size], that we use to
                //      update self.windows_size. This is now longer possible, and I
                //      don't know where to get the new size.
                //      Please, check and fix the self.window_size value.
                self.resize(self.window_size, self.scale_factor);
                *resized = true;
                *scale_factor_changed = true;
                if self.window_size.width > 0 && self.window_size.height > 0 {
                    self.generate_textures();
                }
            }
            WindowEvent::MouseInput { state, .. } => {
                let element = self.pixel_to_element(self.state.mouse_position());
                let mouse_position = self.state.mouse_position();
                match element {
                    PixelRegion::Resize(n) if *state == ElementState::Pressed => {
                        let mut clicked_position = mouse_position.clone();
                        clicked_position.x /= self.window_size.width as f64;
                        clicked_position.y /= self.window_size.height as f64;
                        let old_proportion = self.layout.get_proportion(n).unwrap();
                        self.state = State::Resizing {
                            mouse_position,
                            clicked_position,
                            region: n,
                            old_proportion,
                        };
                    }
                    PixelRegion::Resize(_) => {
                        self.state = State::Normal { mouse_position };
                        if log::log_enabled!(log::Level::Info) {
                            log::info!("Tree after resize");
                            self.layout.log_tree();
                        }
                    }
                    PixelRegion::Element(element) => match state {
                        ElementState::Pressed => {
                            self.state = State::Interacting {
                                mouse_position,
                                element,
                            };
                        }
                        ElementState::Released => {
                            if matches!(self.state, State::Resizing { .. })
                                && log::log_enabled!(log::Level::Info)
                            {
                                log::info!("Tree after resize");
                                self.layout.log_tree();
                            }
                            self.state = State::Normal { mouse_position };
                        }
                    },
                    _ => unreachable!(),
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        location,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                //
                // NOTE: Gui keyboard shortcuts are defined and handled here.
                //
                captured = true;
                match logical_key.as_ref() {
                    Key::Named(NamedKey::Escape) => {
                        self.requests.lock().unwrap().action_mode = Some(ActionMode::Normal)
                    }
                    Key::Character("X") | Key::Character("x") if self.modifiers_state.alt_key() => {
                        self.requests.lock().unwrap().keep_proceed.push_back(
                            Action::MakeAllSuggestedXover {
                                doubled: self.modifiers_state.shift_key(),
                            },
                        )
                    }
                    Key::Character("X") | Key::Character("x") => {
                        self.requests.lock().unwrap().toggle_all_helices_on_axis = Some(());
                    }
                    Key::Character("Z") | Key::Character("z")
                        if control_key(&self.modifiers_state) =>
                    {
                        if self.modifiers_state.shift_key() {
                            self.requests.lock().unwrap().redo = Some(())
                        } else {
                            self.requests.lock().unwrap().undo = Some(());
                        }
                    }
                    Key::Character("R") | Key::Character("r")
                        if control_key(&self.modifiers_state) =>
                    {
                        self.requests.lock().unwrap().redo = Some(());
                    }
                    Key::Character("C") | Key::Character("c")
                        if control_key(&self.modifiers_state) =>
                    {
                        self.requests.lock().unwrap().copy = Some(());
                    }
                    Key::Character("V") | Key::Character("v")
                        if control_key(&self.modifiers_state) =>
                    {
                        self.requests.lock().unwrap().paste = Some(());
                    }
                    Key::Character("J") | Key::Character("j")
                        if control_key(&self.modifiers_state) =>
                    {
                        self.requests.lock().unwrap().duplication = Some(());
                    }
                    Key::Character("L") | Key::Character("l")
                        if control_key(&self.modifiers_state) =>
                    {
                        self.requests.lock().unwrap().anchor = Some(());
                    }
                    Key::Character("R") | Key::Character("r")
                        if !control_key(&self.modifiers_state) =>
                    {
                        self.requests.lock().unwrap().action_mode = Some(ActionMode::Rotate)
                    }
                    Key::Character("T") | Key::Character("t") => {
                        self.requests.lock().unwrap().action_mode = Some(ActionMode::Translate)
                    }
                    Key::Character("N") | Key::Character("n") => {
                        self.requests.lock().unwrap().selection_mode =
                            Some(SelectionMode::Nucleotide)
                    }
                    Key::Character("H") | Key::Character("h") => {
                        self.requests.lock().unwrap().selection_mode = Some(SelectionMode::Helix)
                    }
                    Key::Character("S") | Key::Character("s")
                        if control_key(&self.modifiers_state) =>
                    {
                        self.requests.lock().unwrap().save_shortcut = Some(());
                    }
                    Key::Character("O") | Key::Character("o")
                        if control_key(&self.modifiers_state) =>
                    {
                        self.requests
                            .lock()
                            .unwrap()
                            .keep_proceed
                            .push_back(Action::LoadDesign(None));
                    }
                    Key::Character("Q") | Key::Character("q")
                        if control_key(&self.modifiers_state) && cfg!(target_os = "macos") =>
                    {
                        self.requests
                            .lock()
                            .unwrap()
                            .keep_proceed
                            .push_back(Action::Exit);
                    }
                    Key::Character("S") | Key::Character("s") => {
                        self.requests.lock().unwrap().selection_mode = Some(SelectionMode::Strand)
                    }
                    Key::Character("K") | Key::Character("k") => {
                        self.requests.lock().unwrap().recolor_staples = Some(());
                    }
                    Key::Named(NamedKey::Delete) | Key::Named(NamedKey::Backspace) => {
                        self.requests.lock().unwrap().delete_selection = Some(());
                    }
                    Key::Character("0")
                    | Key::Character("1")
                    | Key::Character("2")
                    | Key::Character("3")
                    | Key::Character("4")
                    | Key::Character("5")
                    | Key::Character("6")
                    | Key::Character("7")
                    | Key::Character("8")
                    | Key::Character("9")
                        if location == &KeyLocation::Standard =>
                    {
                        if let Some(num) = keycode_to_num(logical_key, location) {
                            self.requests
                                .lock()
                                .unwrap()
                                .keep_proceed
                                .push_back(Action::SelectFavoriteCamera(num));
                        } else {
                            captured = false
                        }
                    }
                    _ => captured = false,
                }
            }
            _ => {}
        }

        // NOTE: Return the event if it has not been captured.
        if let Some(focus) = self.focus.filter(|_| !captured) {
            Some((event, focus))
        } else {
            None
        }
    }

    pub fn change_ui_size(&mut self, ui_size: UiSize, window: &Window) {
        self.ui_size = ui_size;
        self.resize(window.inner_size(), window.scale_factor());
        self.generate_textures();
    }

    fn change_split_(&mut self, split_mode: SplitMode) {
        match self.split_mode {
            SplitMode::Both => {
                let new_type = match split_mode {
                    SplitMode::Scene3D => self.element_3d,
                    SplitMode::Flat => self.element_2d,
                    SplitMode::Both => unreachable!(),
                };
                self.layout.merge(GuiComponentType::Scene, new_type);
            }
            SplitMode::Scene3D | SplitMode::Flat => {
                let id = self
                    .layout
                    .get_area_id(self.element_3d)
                    .or(self.layout.get_area_id(self.element_2d))
                    .unwrap();
                match split_mode {
                    SplitMode::Both => {
                        let (scene, flat_scene) = self.layout.vsplit(id, 0.5, true);
                        self.layout.attribute_element(scene, self.element_3d);
                        self.layout.attribute_element(flat_scene, self.element_2d);
                    }
                    SplitMode::Scene3D => self.layout.attribute_element(id, self.element_3d),
                    SplitMode::Flat => self.layout.attribute_element(id, self.element_2d),
                }
            }
        }
    }

    pub fn toggle_2d(&mut self) {
        log::info!("Toggle 2d");
        if log::log_enabled!(log::Level::Info) {
            println!("Old tree");
            self.layout.log_tree();
        }
        let old_element_2d = self.element_2d;
        if self.element_2d == GuiComponentType::FlatScene {
            self.element_2d = GuiComponentType::StereographicScene;
        } else {
            self.element_2d = GuiComponentType::FlatScene;
        }
        if let Some(id) = self.layout.get_area_id(old_element_2d) {
            self.layout.attribute_element(id, self.element_2d)
        }
        log::info!("new element_2d {:?}", self.element_2d);
        if log::log_enabled!(log::Level::Info) {
            println!("New tree");
            self.layout.log_tree();
        }
        self.generate_textures();
    }

    pub fn change_split(&mut self, split_mode: SplitMode) {
        if split_mode != self.split_mode {
            self.change_split_(split_mode)
        }
        self.split_mode = split_mode;
        self.generate_textures();
    }

    pub fn resize(&mut self, window_size: PhySize, scale_factor: f64) -> bool {
        let ret = self.window_size != window_size;
        let top_panel_prop =
            self.ui_size.top_bar_height() * scale_factor / window_size.height as f64;
        let scene_height = (1. - top_panel_prop) * window_size.height as f64;
        let status_bar_prop = MAX_STATUS_BAR_HEIGHT * scale_factor / scene_height;
        self.layout.resize(self.top_bar_split, top_panel_prop);
        self.layout
            .resize(self.status_bar_split, 1. - status_bar_prop);
        ret
    }

    fn texture(&mut self, element_type: GuiComponentType) -> Option<MultiplexerTexture> {
        let area = self.get_draw_area(element_type)?;
        log::debug!("texture of {:?}: {:?}", element_type, area);
        let texture = SampledTexture::create_target_texture(self.device.as_ref(), &area.size);
        Some(MultiplexerTexture { area, texture })
    }

    pub fn generate_textures(&mut self) {
        self.scene_texture = self.texture(GuiComponentType::Scene);
        self.top_bar_texture = self.texture(GuiComponentType::TopBar);
        self.left_panel_texture = self.texture(GuiComponentType::LeftPanel);
        self.grid_panel_texture = self.texture(GuiComponentType::GridPanel);
        self.flat_scene_texture = self.texture(GuiComponentType::FlatScene);
        self.status_bar_texture = self.texture(GuiComponentType::StatusBar);
        self.stereographic_scene_texture = self.texture(GuiComponentType::StereographicScene);

        self.overlays_textures.clear();
        for overlay in self.overlays.iter() {
            let position = overlay.position;
            let size = overlay.size;
            let texture = SampledTexture::create_target_texture(self.device.as_ref(), &size);

            self.overlays_textures.push(MultiplexerTexture {
                texture,
                area: DrawArea { size, position },
            });
        }
    }

    /// Maps *physical* pixels to an element
    fn pixel_to_element(&self, pixel: PhysicalPosition<f64>) -> PixelRegion {
        let pixel_u32 = pixel.cast::<u32>();
        for (n, overlay) in self.overlays.iter().enumerate() {
            if overlay.contains_pixel(pixel_u32) {
                return PixelRegion::Element(GuiComponentType::Overlay(n));
            }
        }
        self.layout.get_area_pixel(
            pixel.x / self.window_size.width as f64,
            pixel.y / self.window_size.height as f64,
        )
    }

    /// Get the drawing area attributed to an element.
    pub fn get_element_area(&self, element: GuiComponentType) -> Option<DrawArea> {
        self.get_draw_area(element)
    }

    /// Return the *physical* position of the cursor, in the focused element coordinates
    pub fn get_cursor_position(&self) -> PhysicalPosition<f64> {
        self.cursor_position
    }

    /// Return the focused element
    pub fn focused_element(&self) -> Option<GuiComponentType> {
        self.focus
    }

    pub fn set_overlays(&mut self, overlays: Vec<Overlay>) {
        self.overlays = overlays;
        self.overlays_textures.clear();
        for overlay in self.overlays.iter_mut() {
            let size = overlay.size;
            let texture = SampledTexture::create_target_texture(self.device.as_ref(), &size);
            self.overlays_textures.push(MultiplexerTexture {
                texture,
                area: DrawArea {
                    size,
                    position: overlay.position,
                },
            });
        }
    }

    pub fn is_showing(&self, area: &GuiComponentType) -> bool {
        match area {
            GuiComponentType::LeftPanel
            | GuiComponentType::TopBar
            | GuiComponentType::StatusBar => true,
            t if *t == self.element_3d => {
                self.split_mode == SplitMode::Scene3D || self.split_mode == SplitMode::Both
            }
            t if *t == self.element_2d => {
                self.split_mode == SplitMode::Flat || self.split_mode == SplitMode::Both
            }
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct Overlay {
    pub position: PhysicalPosition<u32>,
    pub size: PhysicalSize<u32>,
}

impl Overlay {
    pub fn contains_pixel(&self, pixel: PhysicalPosition<u32>) -> bool {
        pixel.x >= self.position.x
            && pixel.y >= self.position.y
            && pixel.x < self.position.x + self.size.width
            && pixel.y < self.position.y + self.size.height
    }
}

fn create_pipeline(device: &Device, bg_layout: &wgpu::BindGroupLayout) -> wgpu::RenderPipeline {
    let vs_module = &device.create_shader_module(wgpu::include_spirv!("multiplexer/draw.vert.spv"));
    let fs_module = &device.create_shader_module(wgpu::include_spirv!("multiplexer/draw.frag.spv"));
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[bg_layout],
        push_constant_ranges: &[],
        label: Some("multiplexer pipeline layout"),
    });

    let targets = &[Some(wgpu::ColorTargetState {
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
    })];

    let primitive = wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleStrip,
        strip_index_format: Some(wgpu::IndexFormat::Uint16),
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: None,
        ..Default::default()
    };

    let desc = wgpu::RenderPipelineDescriptor {
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vs_module,
            entry_point: "main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &fs_module,
            entry_point: "main",
            targets,
        }),
        primitive,
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        label: Some("multiplexer pipeline"),
        multiview: None,
    };

    device.create_render_pipeline(&desc)
}

/// Multiplexer state
enum State {
    Resizing {
        mouse_position: PhysicalPosition<f64>,
        clicked_position: PhysicalPosition<f64>,
        region: usize,
        old_proportion: f64,
    },
    Normal {
        mouse_position: PhysicalPosition<f64>,
    },
    Interacting {
        mouse_position: PhysicalPosition<f64>,
        element: GuiComponentType,
    },
}

impl State {
    fn mouse_position(&self) -> PhysicalPosition<f64> {
        match self {
            Self::Resizing { mouse_position, .. }
            | Self::Normal { mouse_position }
            | Self::Interacting { mouse_position, .. } => *mouse_position,
        }
    }
}

/// MaxOS, Windows and Linux compatible modifier key.
fn control_key(modifiers: &ModifiersState) -> bool {
    if cfg!(target_os = "macos") {
        modifiers.super_key() // ❖ or ⌘
    } else {
        modifiers.control_key() // Ctrl
    }
}

use crate::ensnano_interactor::Multiplexer as GuiMultiplexer;

impl GuiMultiplexer for Multiplexer {
    fn get_draw_area(&self, element_type: GuiComponentType) -> Option<DrawArea> {
        self.get_texture_size(element_type)
    }

    fn get_texture_view(&self, element_type: GuiComponentType) -> Option<&wgpu::TextureView> {
        self.get_texture_view(element_type)
    }

    fn get_cursor_position(&self) -> PhysicalPosition<f64> {
        self.get_cursor_position()
    }

    fn focused_element(&self) -> Option<GuiComponentType> {
        self.focused_element()
    }
}

fn keycode_to_num(key: &Key, _location: &KeyLocation) -> Option<u32> {
    match key {
        // NOTE: We make no distinction on the key location here.
        //       Specify it if you need to.
        Key::Character(char) => match char.as_str() {
            "0" => Some(0),
            "1" => Some(1),
            "2" => Some(2),
            "3" => Some(3),
            "4" => Some(4),
            "5" => Some(5),
            "6" => Some(6),
            "7" => Some(7),
            "8" => Some(8),
            "9" => Some(9),
            _ => None,
        },
        _ => None,
    }
    //if keycode as u32 >= VirtualKeyCode::Key1 as u32
    //    && keycode as u32 <= VirtualKeyCode::Key0 as u32
    //{
    //    Some(keycode as u32 - VirtualKeyCode::Key1 as u32)
    //} else if keycode == VirtualKeyCode::Numpad0 {
    //    Some(9)
    //} else if keycode as u32 >= VirtualKeyCode::Numpad1 as u32
    //    && keycode as u32 <= VirtualKeyCode::Numpad9 as u32
    //{
    //    Some(keycode as u32 - VirtualKeyCode::Numpad1 as u32)
    //} else {
    //    None
    //}
}

struct MultiplexerTexture {
    area: DrawArea,
    texture: SampledTexture,
}
