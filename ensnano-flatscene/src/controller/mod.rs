//! The [Controller] struct handles the event that happens on the drawing area of the scene.
//!
//! The [Controller] is internally implemented as a finite automata that transitions when a event
//! happens. In addition to the transition in the automata, a [Consequence] is returned to the
//! scene, that describes the consequences that the input must have on the view or the data held by
//! the scene.

mod automata;

use crate::{
    AppState, CameraPtr, DataPtr, ViewPtr,
    data::{ClickResult, strand::FreeEnd},
    flat_types::{FlatHelix, FlatNucl},
};
use automata::{ControllerState, NormalState, Transition, ctrl};
use ensnano_design::selection::{ActionMode, Selection};
use ensnano_utils::graphics::PhySize;
use std::cell::RefCell;
use ultraviolet::Vec2;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{Key, ModifiersState, NamedKey},
    window::CursorIcon,
};

pub(crate) struct Controller<S: AppState> {
    view: ViewPtr,
    data: DataPtr<S::Reader>,
    window_size: PhySize,
    area_size: PhySize,
    camera_top: CameraPtr,
    camera_bottom: CameraPtr,
    is_split: bool,
    state: RefCell<Box<dyn ControllerState<S>>>,
    action_mode: ActionMode,
    modifiers: ModifiersState,
    mouse_position: PhysicalPosition<f64>,
}

#[derive(Debug)]
pub(crate) enum Consequence {
    Nothing,
    Xover(FlatNucl, FlatNucl),
    Cut(FlatNucl),
    CutCross(FlatNucl, FlatNucl),
    FreeEnd(Option<FreeEnd>),
    CutFreeEnd(FlatNucl, Option<FreeEnd>),
    NewCandidate(Option<FlatNucl>),
    NewHelixCandidate(FlatHelix),
    FlipVisibility(FlatHelix, bool),
    Built,
    FlipGroup(FlatHelix),
    FollowingSuggestion(FlatNucl, bool),
    DrawingSelection(PhysicalPosition<f64>, PhysicalPosition<f64>),
    ReleasedSelection(Option<Vec<Selection>>),
    PasteRequest(Option<FlatNucl>),
    AddClick(ClickResult, bool),
    SelectionChanged(Vec<Selection>),
    ClearSelection,
    DoubleClick(ClickResult),
    MoveBuilders(isize),
    InitBuilding(FlatNucl),
    Helix2DMovementEnded,
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
    pub(crate) fn new(
        view: ViewPtr,
        data: DataPtr<S::Reader>,
        window_size: PhySize,
        area_size: PhySize,
        camera_top: CameraPtr,
        camera_bottom: CameraPtr,
        is_split: bool,
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
            is_split,
            action_mode: ActionMode::Normal,
            modifiers: ModifiersState::empty(),
            mouse_position: PhysicalPosition::from((0., 0.)),
        }
    }

    pub(crate) fn update_modifiers(&mut self, modifiers: ModifiersState) {
        self.modifiers = modifiers;
    }

    pub(crate) fn resize(&mut self, window_size: PhySize, area_size: PhySize) {
        self.area_size = area_size;
        self.window_size = window_size;
        self.update_globals();
    }

    pub(crate) fn set_split(&mut self, is_split: bool, refit: bool) {
        self.is_split = is_split;
        let old_rectangle_top = self.camera_top.borrow().get_visible_rectangle();
        self.update_globals();
        if refit {
            if is_split {
                let (new_top, new_bottom) = old_rectangle_top.split_vertically();
                self.camera_top.borrow_mut().fit_center(new_top);
                self.camera_bottom.borrow_mut().fit_center(new_bottom);
            } else {
                let new_top = old_rectangle_top.double_height();
                self.camera_top.borrow_mut().fit_center(new_top);
            }
        }
    }

    fn update_globals(&self) {
        if self.is_split {
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

    pub(crate) fn get_camera(&self, y: f64) -> CameraPtr {
        if self.is_split {
            if y > self.area_size.height as f64 / 2. {
                self.camera_bottom.clone()
            } else {
                self.camera_top.clone()
            }
        } else {
            self.camera_top.clone()
        }
    }

    pub(crate) fn get_other_camera(&self, y: f64) -> Option<CameraPtr> {
        if self.is_split {
            if y > self.area_size.height as f64 / 2. {
                Some(self.camera_top.clone())
            } else {
                Some(self.camera_bottom.clone())
            }
        } else {
            None
        }
    }

    pub(crate) fn input(
        &mut self,
        event: &WindowEvent,
        position: PhysicalPosition<f64>,
        app_state: &S,
    ) -> Consequence {
        self.update_hovered_nucl(position);
        self.mouse_position = position;

        let transition = if matches!(event, WindowEvent::Focused(false)) {
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
            self.state.borrow().transition_from(self);
            self.state = RefCell::new(state);
            self.state.borrow().transition_to(self);
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

    pub(crate) fn process_keyboard(&self, event: &WindowEvent) {
        if let WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    logical_key,
                    state: ElementState::Pressed,
                    ..
                },
            ..
        } = event
        {
            let camera = self.get_camera(self.mouse_position.y);
            match logical_key.as_ref() {
                Key::Named(NamedKey::ArrowLeft) if self.modifiers.alt_key() => {
                    camera.borrow_mut().tilt_left();
                }
                Key::Named(NamedKey::ArrowRight) if self.modifiers.alt_key() => {
                    camera.borrow_mut().tilt_right();
                }
                Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowRight) if ctrl(&self.modifiers) => {
                    camera.borrow_mut().apply_symmetry_x();
                }
                Key::Named(NamedKey::ArrowUp | NamedKey::ArrowDown) if ctrl(&self.modifiers) => {
                    camera.borrow_mut().apply_symmetry_y();
                }
                Key::Character("J") => {
                    self.data.borrow_mut().move_helix_backward();
                }
                Key::Character("K") => {
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
        if self.is_split {
            self.area_size.height / 2
        } else {
            self.area_size.height
        }
    }

    fn is_bottom(&self, y: f64) -> bool {
        if self.is_split {
            y > self.area_size.height as f64 / 2.
        } else {
            false
        }
    }

    pub(crate) fn check_timers(&mut self) -> Consequence {
        let transition = self.state.borrow_mut().check_timers(self);
        if let Some(state) = transition.new_state {
            log::info!("{}", state.display());
            self.state.borrow().transition_from(self);
            self.state = RefCell::new(state);
            self.state.borrow().transition_to(self);
        }
        transition.consequences
    }

    pub(crate) fn flip_split_views(&self) {
        self.camera_bottom
            .borrow_mut()
            .swap(&mut self.camera_top.borrow_mut());
    }

    pub(crate) fn get_icon(&self) -> Option<CursorIcon> {
        self.state.borrow().cursor()
    }
}
