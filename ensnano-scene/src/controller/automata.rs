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
use super::*;
use crate::controller::automata::dragging_state::translating_grid_object;
use crate::DesignReader;
use ensnano_design::ultraviolet::Vec2;
use ensnano_design::{
    grid::{GridId, GridObject},
    BezierPlaneId,
};
use ensnano_interactor::{ActionMode, CursorIcon};
use std::borrow::Cow;
use std::cell::RefCell;
use std::time::Instant;

use super::AppState;

mod dragging_state;
use dragging_state::*;
mod point_and_click_state;
use point_and_click_state::PointAndClicking;
mod event_context;
pub use event_context::EventContext;
use event_context::*;

pub(super) type State<S> = RefCell<Box<dyn ControllerState<S>>>;

pub(super) fn initial_state<S: AppState>() -> State<S> {
    RefCell::new(Box::new(NormalState {
        mouse_position: PhysicalPosition::new(-1., -1.),
    }))
}

pub(super) struct Transition<S: AppState> {
    pub new_state: Option<Box<dyn ControllerState<S>>>,
    pub consequences: Consequence,
}

impl<S: AppState> Transition<S> {
    pub fn nothing() -> Self {
        Self {
            new_state: None,
            consequences: Consequence::Nothing,
        }
    }

    pub fn consequence(consequences: Consequence) -> Self {
        Self {
            new_state: None,
            consequences,
        }
    }
}

pub(super) trait ControllerState<S: AppState> {
    fn input<'a>(
        &mut self,
        event: &WindowEvent,
        context: &'a mut EventContext<'a, S>,
    ) -> Transition<S>;

    #[allow(dead_code)]
    fn display(&self) -> Cow<'static, str>;

    fn transition_from(&self, _controller: &Controller<S>) -> TransistionConsequence {
        TransistionConsequence::Nothing
    }

    fn transition_to(&self, _controller: &Controller<S>) -> TransistionConsequence {
        TransistionConsequence::Nothing
    }

    fn check_timers(&mut self, _controller: &Controller<S>) -> Transition<S> {
        Transition::nothing()
    }

    fn handles_color_system(&self) -> Option<HandleColors> {
        None
    }

    fn element_being_selected(&self) -> Option<SceneElement> {
        None
    }

    fn notify_scroll(&mut self) {
        ()
    }
    fn cursor(&self) -> Option<ensnano_interactor::CursorIcon> {
        None
    }
}

pub struct NormalState {
    pub mouse_position: PhysicalPosition<f64>,
}

impl<S: AppState> ControllerState<S> for NormalState {
    fn input<'a>(
        &mut self,
        event: &WindowEvent,
        context: &'a mut EventContext<'a, S>,
    ) -> Transition<S> {
        let path_id = context.get_id_of_bezier_bath_being_eddited();
        match event {
            WindowEvent::CursorMoved { .. } if context.is_pasting() => {
                self.mouse_position = context.cursor_position;
                let element = context.get_element_under_cursor();
                let element = context.convert_grid_to_grid_disc(element);
                Transition::consequence(Consequence::PasteCandidate(element))
            }
            WindowEvent::CursorMoved { .. } => {
                self.mouse_position = context.cursor_position;
                let element = context.get_element_under_cursor();
                let element = context.convert_grid_to_grid_disc(element);
                Transition::consequence(Consequence::Candidate(element))
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } if context.get_modifiers().alt() => {
                let click_info = ClickInfo::new(MouseButton::Left, context.cursor_position);
                Transition {
                    new_state: Some(Box::new(dragging_state::translating_camera(click_info))),
                    consequences: Consequence::Nothing,
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } if context.is_pasting() => {
                let element = context.get_element_under_cursor();
                let element = context.convert_grid_to_grid_disc(element);
                Transition {
                    new_state: Some(Box::new(PointAndClicking::pasting(
                        context.cursor_position,
                        element,
                    ))),
                    consequences: Consequence::Nothing,
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                let element = context.get_element_under_cursor();
                log::info!("Clicked on {:?}", element);
                if let Some(SceneElement::PlaneCorner {
                    plane_id,
                    corner_type,
                }) = element
                {
                    let fixed_corner_position =
                        context.get_position_of_opposite_plane_corner(plane_id, corner_type);
                    return Transition {
                        new_state: Some(Box::new(MovingBezierCorner {
                            plane_id,
                            clicked_position: context.cursor_position,
                            fixed_corner_position,
                        })),
                        consequences: Consequence::Nothing,
                    };
                } else if let Some(SceneElement::BezierVertex { vertex_id, path_id }) = element {
                    let click_info = ClickInfo::new(MouseButton::Left, context.cursor_position);
                    return Transition {
                        new_state: Some(Box::new(dragging_state::moving_bezier_vertex(
                            click_info,
                            MovingBezierVertex::Existing { vertex_id, path_id },
                        ))),
                        consequences: Consequence::Nothing,
                    };
                } else if path_id.is_some() {
                    if let Some((plane_id, intersection)) = context.get_plane_under_cursor() {
                        let click_info = ClickInfo::new(MouseButton::Left, context.cursor_position);
                        return Transition {
                            new_state: Some(Box::new(dragging_state::moving_bezier_vertex(
                                click_info,
                                MovingBezierVertex::New { plane_id },
                            ))),
                            consequences: Consequence::CreateBezierVertex {
                                vertex: BezierVertex::new(
                                    plane_id,
                                    Vec2::new(intersection.x, intersection.y),
                                ),
                                path: path_id.unwrap(),
                            },
                        };
                    }
                }
                match element {
                    Some(SceneElement::GridCircle(d_id, grid_position)) => {
                        if let ActionMode::BuildHelix {
                            position: position_helix,
                            length,
                        } = context.get_action_mode()
                        {
                            Transition {
                                new_state: Some(Box::new(PointAndClicking::building_helix(
                                    BuildingHelix {
                                        position_helix,
                                        length_helix: length,
                                        x_helix: grid_position.x,
                                        y_helix: grid_position.y,
                                        grid_id: grid_position.grid,
                                        design_id: d_id,
                                        clicked_position: context.cursor_position,
                                    },
                                ))),
                                consequences: Consequence::Nothing,
                            }
                        } else {
                            let adding =
                                context.get_modifiers().shift() || ctrl(context.get_modifiers());
                            let new_state = PointAndClicking::selecting(
                                context.cursor_position,
                                element,
                                adding,
                            );
                            Transition {
                                new_state: Some(Box::new(new_state)),
                                consequences: Consequence::Nothing,
                            }
                        }
                    }
                    Some(SceneElement::Grid(d_id, _)) => {
                        let grid_intersection = context.get_grid_intersection_with_cursor();
                        if let ActionMode::BuildHelix {
                            position: helix_position,
                            length,
                        } = context.get_action_mode()
                        {
                            if let Some(intersection) = grid_intersection {
                                if let Some(object) =
                                    context.get_object_at_grid_pos(intersection.grid_position())
                                {
                                    let click_info =
                                        ClickInfo::new(MouseButton::Left, context.cursor_position);
                                    let new_state = translating_grid_object(
                                        click_info,
                                        dragging_state::TranslatingGridObject {
                                            grid_id: intersection.grid_id,
                                            object: object.clone(),
                                            x: intersection.x,
                                            y: intersection.y,
                                        },
                                    );
                                    Transition {
                                        new_state: Some(Box::new(new_state)),
                                        consequences: Consequence::HelixSelected(object.helix()),
                                    }
                                } else {
                                    Transition {
                                        new_state: Some(Box::new(
                                            PointAndClicking::building_helix(BuildingHelix {
                                                position_helix: helix_position,
                                                length_helix: length,
                                                x_helix: intersection.x,
                                                y_helix: intersection.y,
                                                grid_id: intersection.grid_id,
                                                design_id: d_id,
                                                clicked_position: context.cursor_position,
                                            }),
                                        )),
                                        consequences: Consequence::Nothing,
                                    }
                                }
                            } else {
                                let adding = context.get_modifiers().shift()
                                    || ctrl(context.get_modifiers());
                                let new_state = PointAndClicking::selecting(
                                    context.cursor_position,
                                    element,
                                    adding,
                                );
                                Transition {
                                    new_state: Some(Box::new(new_state)),
                                    consequences: Consequence::Nothing,
                                }
                            }
                        } else {
                            let clicked_element;
                            let object;
                            if let Some(intersection) = grid_intersection.as_ref() {
                                clicked_element = Some(SceneElement::GridCircle(
                                    d_id,
                                    intersection.grid_position(),
                                ));
                                object =
                                    context.get_object_at_grid_pos(intersection.grid_position());
                            } else {
                                clicked_element = element;
                                object = None;
                            };
                            if let Some(obj) = object {
                                // if helix is Some, intersection is also Some
                                let intersection = grid_intersection.unwrap();
                                let click_info =
                                    ClickInfo::new(MouseButton::Left, context.cursor_position);
                                let new_state = translating_grid_object(
                                    click_info,
                                    dragging_state::TranslatingGridObject {
                                        grid_id: intersection.grid_id,
                                        object: obj.clone(),
                                        x: intersection.x,
                                        y: intersection.y,
                                    },
                                );
                                Transition {
                                    new_state: Some(Box::new(new_state)),
                                    consequences: Consequence::HelixSelected(obj.helix()),
                                }
                            } else {
                                let adding = context.get_modifiers().shift()
                                    || ctrl(context.get_modifiers());
                                let new_state = PointAndClicking::selecting(
                                    context.cursor_position,
                                    clicked_element,
                                    adding,
                                );
                                Transition {
                                    new_state: Some(Box::new(new_state)),
                                    consequences: Consequence::Nothing,
                                }
                            }
                        }
                    }
                    Some(SceneElement::WidgetElement(widget_id)) => {
                        let normalized_cursor_position = context.normalized_cursor_position();
                        let translation_target = if context.get_modifiers().shift() {
                            WidgetTarget::Pivot
                        } else {
                            WidgetTarget::Object
                        };
                        match widget_id {
                            UP_HANDLE_ID | DIR_HANDLE_ID | RIGHT_HANDLE_ID => {
                                let click_info =
                                    ClickInfo::new(MouseButton::Left, context.cursor_position);
                                let new_state = dragging_state::translating_widget(
                                    click_info,
                                    HandleDir::from_widget_id(widget_id),
                                    translation_target,
                                );
                                Transition {
                                    new_state: Some(Box::new(new_state)),
                                    consequences: Consequence::InitTranslation(
                                        normalized_cursor_position.x,
                                        normalized_cursor_position.y,
                                        translation_target,
                                    ),
                                }
                            }
                            RIGHT_CIRCLE_ID | FRONT_CIRCLE_ID | UP_CIRCLE_ID => {
                                let target = if context.get_modifiers().shift() {
                                    WidgetTarget::Pivot
                                } else {
                                    WidgetTarget::Object
                                };

                                let click_info =
                                    ClickInfo::new(MouseButton::Left, context.cursor_position);
                                let new_state = dragging_state::rotating_widget(click_info, target);
                                Transition {
                                    new_state: Some(Box::new(new_state)),
                                    consequences: Consequence::InitRotation(
                                        RotationMode::from_widget_id(widget_id),
                                        normalized_cursor_position.x,
                                        normalized_cursor_position.y,
                                        target,
                                    ),
                                }
                            }
                            _ => {
                                println!("WARNING UNEXPECTED WIDGET ID");
                                Transition::nothing()
                            }
                        }
                    }
                    Some(SceneElement::DesignElement(_, _))
                        if ctrl(context.get_modifiers())
                            && context.element_to_nucl(&element, true).is_some() =>
                    {
                        if let Some(nucl) = context.element_to_nucl(&element, true) {
                            Transition::consequence(Consequence::QuickXoverAttempt {
                                nucl,
                                doubled: context.get_modifiers().shift(),
                            })
                        } else {
                            Transition::nothing()
                        }
                    }
                    _ => {
                        let adding =
                            context.get_modifiers().shift() || ctrl(context.get_modifiers());
                        let new_state =
                            PointAndClicking::selecting(context.cursor_position, element, adding);
                        Transition {
                            new_state: Some(Box::new(new_state)),
                            consequences: Consequence::Nothing,
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Middle,
                ..
            } if ctrl(context.get_modifiers()) => {
                let click_info = ClickInfo::new(MouseButton::Middle, context.cursor_position);
                Transition {
                    new_state: Some(Box::new(dragging_state::rotating_camera(click_info))),
                    consequences: Consequence::Nothing,
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Middle,
                ..
            } => {
                let click_info = ClickInfo::new(MouseButton::Middle, context.cursor_position);
                Transition {
                    new_state: Some(Box::new(dragging_state::translating_camera(click_info))),
                    consequences: Consequence::Nothing,
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                ..
            } => {
                let element = context.get_pivot_element();
                Transition {
                    new_state: Some(Box::new(PointAndClicking::setting_pivot(
                        context.cursor_position,
                        element,
                    ))),
                    consequences: Consequence::Nothing,
                }
            }
            _ => Transition::nothing(),
        }
    }

    fn display(&self) -> Cow<'static, str> {
        "Normal".into()
    }
}

/// What is being affected by the translation
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum WidgetTarget {
    /// The selected elements
    Object,
    /// The selection's pivot
    Pivot,
}

struct BuildingHelix {
    design_id: u32,
    grid_id: GridId,
    x_helix: isize,
    y_helix: isize,
    length_helix: usize,
    position_helix: isize,
    clicked_position: PhysicalPosition<f64>,
}

struct MovingBezierCorner {
    plane_id: BezierPlaneId,
    clicked_position: PhysicalPosition<f64>,
    fixed_corner_position: Vec2,
}

impl<S: AppState> ControllerState<S> for MovingBezierCorner {
    fn display(&self) -> Cow<'static, str> {
        "Moving bezier vertex".into()
    }

    fn input<'a>(
        &mut self,
        event: &WindowEvent,
        context: &'a mut EventContext<'a, S>,
    ) -> Transition<S> {
        match event {
            WindowEvent::CursorMoved { .. } => {
                let mouse_x = position.x / controller.area_size.width as f64;
                let mouse_y = position.y / controller.area_size.height as f64;
                let ray = controller
                    .camera_controller
                    .ray(mouse_x as f32, mouse_y as f32);
                let ray_origin = controller.camera_controller.ray(
                    self.clicked_position.x as f32 / controller.area_size.width as f32,
                    self.clicked_position.y as f32 / controller.area_size.height as f32,
                );
                if let Some((moving_corner, original_corner_position)) = app_state
                    .get_design_reader()
                    .get_bezier_planes()
                    .get(&self.plane_id)
                    .and_then(|plane| {
                        plane
                            .ray_intersection(ray.0, ray.1)
                            .zip(plane.ray_intersection(ray_origin.0, ray_origin.1))
                    })
                {
                    Transition::consequence(Consequence::MoveBezierCorner {
                        moving_corner: moving_corner.position(),
                        original_corner_position: original_corner_position.position(),
                        plane_id: self.plane_id,
                        fixed_corner_position: self.fixed_corner_position,
                    })
                } else {
                    Transition::nothing()
                }
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Released,
                ..
            } => Transition {
                new_state: Some(Box::new(NormalState {
                    mouse_position: position,
                })),
                consequences: Consequence::ReleaseBezierCorner,
            },
            _ => Transition::nothing(),
        }
    }
}

fn position_difference(a: PhysicalPosition<f64>, b: PhysicalPosition<f64>) -> f64 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

fn ctrl(modifiers: &ModifiersState) -> bool {
    if cfg!(target_os = "macos") {
        modifiers.logo()
    } else {
        modifiers.ctrl()
    }
}
