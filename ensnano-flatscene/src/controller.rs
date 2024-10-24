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
//! The [Controller] struct handles the event that happens on the drawing area of the scene.
//!
//! The [Controller] is internally implemented as a finite automata that transitions when a event
//! happens. In addition to the transition in the automata, a [Consequence] is returned to the
//! scene, that describes the consequences that the input must have on the view or the data held by
//! the scene.
use super::data::{ClickResult, FreeEnd};
use super::{
    ActionMode, AppState, CameraPtr, DataPtr, FlatHelix, FlatNucl, PhySize, PhysicalPosition,
    Selection, ViewPtr, WindowEvent,
};

use ensnano_design::ultraviolet;
use ensnano_utils::winit::event::*;
use std::cell::RefCell;
use ultraviolet::Vec2;

mod automata;
use automata::{ctrl, ControllerState, NormalState, Transition};

pub struct Controller<S: AppState> {
    #[allow(dead_code)]
    view: ViewPtr,
    data: DataPtr<S::Reader>,
    #[allow(dead_code)]
    window_size: PhySize,
    area_size: PhySize,
    camera_top: CameraPtr,
    camera_bottom: CameraPtr,
    splited: bool,
    state: RefCell<Box<dyn ControllerState<S>>>,
    action_mode: ActionMode,
    modifiers: ModifiersState,
    mouse_position: PhysicalPosition<f64>,
}

#[derive(Debug)]
pub enum Consequence {
    #[allow(dead_code)]
    GlobalsChanged,
    Nothing,
    Xover(FlatNucl, FlatNucl),
    Cut(FlatNucl),
    CutCross(FlatNucl, FlatNucl),
    FreeEnd(Option<FreeEnd>),
    CutFreeEnd(FlatNucl, Option<FreeEnd>),
    NewCandidate(Option<FlatNucl>),
    NewHelixCandidate(FlatHelix),
    RmStrand(FlatNucl),
    RmHelix(FlatHelix),
    FlipVisibility(FlatHelix, bool),
    Built,
    FlipGroup(FlatHelix),
    FollowingSuggestion(FlatNucl, bool),
    Centering(FlatNucl, bool),
    DrawingSelection(PhysicalPosition<f64>, PhysicalPosition<f64>),
    ReleasedSelection(Option<Vec<Selection>>),
    PasteRequest(Option<FlatNucl>),
    AddClick(ClickResult, bool),
    SelectionChanged(Vec<Selection>),
    ClearSelection,
    DoubleClick(ClickResult),
    MoveBuilders(isize),
    InitBuilding(FlatNucl),
    Helix2DMvmtEnded,
    Snap {
        pivots: Vec<FlatNucl>,
        translation: Vec2,
    },
    Rotation {
        helices: Vec<FlatHelix>,
        center: Vec2,
        angle: f32,
    },
    Symmetry {
        helices: Vec<FlatHelix>,
        centers: Vec<Vec2>,
        symmetry: Vec2,
    },
    PngExport(Vec2, Vec2),
}

impl<S: AppState> Controller<S> {
    pub fn new(
        view: ViewPtr,
        data: DataPtr<S::Reader>,
        window_size: PhySize,
        area_size: PhySize,
        camera_top: CameraPtr,
        camera_bottom: CameraPtr,
        splited: bool,
    ) -> Self {
        Self {
            view,
            data,
            window_size,
            area_size,
            camera_top,
            camera_bottom,
            state: RefCell::new(Box::new(NormalState {
                mouse_position: PhysicalPosition::new(-1., -1.),
            })),
            splited,
            action_mode: ActionMode::Normal,
            modifiers: ModifiersState::empty(),
            mouse_position: PhysicalPosition::from((0., 0.)),
        }
    }

    pub fn update_modifiers(&mut self, modifiers: ModifiersState) {
        self.modifiers = modifiers;
    }

    pub fn resize(&mut self, window_size: PhySize, area_size: PhySize) {
        self.area_size = area_size;
        self.window_size = window_size;
        self.update_globals();
    }

    pub fn set_splited(&mut self, splited: bool, refit: bool) {
        self.splited = splited;
        let old_rectangle_top = self.camera_top.borrow().get_visible_rectangle();
        self.update_globals();
        if refit {
            if splited {
                let (new_top, new_bottom) = old_rectangle_top.split_vertically();
                self.camera_top.borrow_mut().fit_center(new_top);
                self.camera_bottom.borrow_mut().fit_center(new_bottom);
            } else {
                let new_top = old_rectangle_top.double_height();
                self.camera_top.borrow_mut().fit_center(new_top);
            }
        }
    }

    fn update_globals(&mut self) {
        if self.splited {
            self.camera_top.borrow_mut().resize(
                self.area_size.width as f32,
                self.area_size.height as f32 / 2.,
            );
            self.camera_bottom.borrow_mut().resize(
                self.area_size.width as f32,
                self.area_size.height as f32 / 2.,
            );
        } else {
            self.camera_top
                .borrow_mut()
                .resize(self.area_size.width as f32, self.area_size.height as f32);
        }
    }

    pub fn get_camera(&self, y: f64) -> CameraPtr {
        if self.splited {
            if y > self.area_size.height as f64 / 2. {
                self.camera_bottom.clone()
            } else {
                self.camera_top.clone()
            }
        } else {
            self.camera_top.clone()
        }
    }

    pub fn get_other_camera(&self, y: f64) -> Option<CameraPtr> {
        if self.splited {
            if y > self.area_size.height as f64 / 2. {
                Some(self.camera_top.clone())
            } else {
                Some(self.camera_bottom.clone())
            }
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn fit(&mut self) {
        let rectangle = self.data.borrow().get_fit_rectangle();
        self.camera_top.borrow_mut().fit_center(rectangle);
        self.camera_bottom.borrow_mut().fit_center(rectangle);
    }

    pub fn input(
        &mut self,
        event: &WindowEvent,
        position: PhysicalPosition<f64>,
        app_state: &S,
    ) -> Consequence {
        self.update_hovered_nucl(position);
        self.mouse_position = position;
        let transition = if let WindowEvent::Focused(false) = event {
            Transition {
                new_state: Some(Box::new(NormalState {
                    mouse_position: PhysicalPosition::new(-1., -1.),
                })),
                consequences: Consequence::Nothing,
            }
        } else {
            self.state
                .borrow_mut()
                .input(event, position, self, app_state)
        };

        if let Some(state) = transition.new_state {
            log::info!("2D automata state: {}", state.display());
            self.state.borrow().transition_from(&self);
            self.state = RefCell::new(state);
            self.state.borrow().transition_to(&self);
        }
        transition.consequences
    }

    fn update_hovered_nucl(&self, position: PhysicalPosition<f64>) {
        let (x, y) = self
            .get_camera(position.y)
            .borrow()
            .screen_to_world(position.x as f32, position.y as f32);
        let click_result = self
            .data
            .borrow()
            .get_click(x, y, &self.get_camera(position.y));
        let nucl = if let ClickResult::Nucl(n) = click_result {
            Some(n)
        } else {
            None
        };
        self.view.borrow_mut().set_hovered_nucl(nucl);
    }

    pub fn process_keyboard(&self, event: &WindowEvent) {
        if let WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    virtual_keycode: Some(key),
                    state: ElementState::Pressed,
                    ..
                },
            ..
        } = event
        {
            let camera = self.get_camera(self.mouse_position.y);
            match *key {
                VirtualKeyCode::Left if self.modifiers.alt() => {
                    camera.borrow_mut().tilt_left();
                }
                VirtualKeyCode::Right if self.modifiers.alt() => {
                    camera.borrow_mut().tilt_right();
                }
                VirtualKeyCode::Left | VirtualKeyCode::Right if ctrl(&self.modifiers) => {
                    camera.borrow_mut().apply_symmetry_x()
                }
                VirtualKeyCode::Up | VirtualKeyCode::Down if ctrl(&self.modifiers) => {
                    camera.borrow_mut().apply_symmetry_y()
                }
                VirtualKeyCode::J => {
                    self.data.borrow_mut().move_helix_backward();
                }
                VirtualKeyCode::K => {
                    self.data.borrow_mut().move_helix_forward();
                }
                _ => (),
            }
        }
    }

    fn end_movement(&self) {
        self.camera_top.borrow_mut().end_movement();
        self.camera_bottom.borrow_mut().end_movement();
    }

    fn get_height(&self) -> u32 {
        if self.splited {
            self.area_size.height / 2
        } else {
            self.area_size.height
        }
    }

    fn is_bottom(&self, y: f64) -> bool {
        if self.splited {
            y > self.area_size.height as f64 / 2.
        } else {
            false
        }
    }

    pub fn check_timers(&mut self) -> Consequence {
        let transition = self.state.borrow_mut().check_timers(&self);
        if let Some(state) = transition.new_state {
            log::info!("{}", state.display());
            self.state.borrow().transition_from(&self);
            self.state = RefCell::new(state);
            self.state.borrow().transition_to(&self);
        }
        transition.consequences
    }

    pub fn flip_split_views(&mut self) {
        self.camera_bottom
            .borrow_mut()
            .swap(&mut self.camera_top.borrow_mut())
    }

    pub fn get_icon(&self) -> Option<ensnano_interactor::CursorIcon> {
        self.state.borrow().cursor()
    }
}
