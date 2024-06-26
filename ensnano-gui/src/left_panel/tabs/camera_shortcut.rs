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
use super::{AppState, Message, UiSize, Vec3};
use crate::{helpers::*, CameraId};
use iced::{alignment::Horizontal, Alignment, Element, Length};

/// A named camera orientation.
///
/// Orientation is defined by the direction pointed by the camera lens, and the direction pointed
/// by the top of the camera (representing the top of the screen view).
#[derive(Clone)]
struct NamedCameraPosition {
    name: &'static str,
    // Direction pointed the camera lens.
    direction: Vec3,
    // Direction pointed the top of the camera.
    up: Vec3,
}

impl NamedCameraPosition {
    /// Generate a message to to ENSnano to set camera to desired position.
    fn message<State: AppState>(&self) -> Message<State> {
        Message::FixPoint(self.direction, self.up)
    }
}

/// Six predefined positions.
const PREDEFINED_CAMERA_ORIENTATION: [NamedCameraPosition; 6] = [
    NamedCameraPosition {
        name: "Left",
        direction: Vec3::new(-1., 0., 0.),
        up: Vec3::new(0., 1., 0.),
    },
    NamedCameraPosition {
        name: "Right",
        direction: Vec3::new(1., 0., 0.),
        up: Vec3::new(0., 1., 0.),
    },
    NamedCameraPosition {
        name: "Top",
        direction: Vec3::new(0., 1., 0.),
        up: Vec3::new(0., 0., 1.),
    },
    NamedCameraPosition {
        name: "Back",
        direction: Vec3::new(0., 0., 1.),
        up: Vec3::new(0., 1., 0.),
    },
    NamedCameraPosition {
        name: "Front",
        direction: Vec3::new(0., 0., -1.),
        up: Vec3::new(0., 1., 0.),
    },
    NamedCameraPosition {
        name: "Bottom",
        direction: Vec3::new(0., -1., 0.),
        up: Vec3::new(0., 0., -1.),
    },
];

/// Turn a NamedCameraPosition into a button.
fn named_camera_to_button<'a, State: AppState>(
    position: &NamedCameraPosition,
    ui_size: UiSize,
) -> Element<'a, Message<State>, crate::Theme, crate::Renderer> {
    fixed_text_button(position.name, 2.0, ui_size)
        .on_press(position.message())
        .into()
}

/// Generate the message that request rotation.
fn rotation_message<State: AppState>(
    i: usize,
    _xz: isize,
    _yz: isize,
    _xy: isize,
) -> Message<State> {
    let angle_xz = match i {
        0 => 15f32.to_radians(),
        1 => -15f32.to_radians(),
        _ => 0f32,
    };
    let angle_yz = match i {
        2 => -15f32.to_radians(),
        3 => 15f32.to_radians(),
        _ => 0f32,
    };
    let angle_xy = match i {
        4 => 15f32.to_radians(),
        5 => -15f32.to_radians(),
        _ => 0f32,
    };
    Message::RotateCam(angle_xz, angle_yz, angle_xy)
}

// Custom camera editor.
struct CameraWidget {
    // Name of the custom camera orientation.
    name: String,
    // Wether the name is being edited.
    being_edited: bool,
    // An id for this camera.
    camera_id: CameraId,
}

impl CameraWidget {
    fn view<State: AppState>(
        &self,
        ui_size: UiSize,
    ) -> Element<Message<State>, crate::Theme, crate::Renderer> {
        let name_field: Element<_, _, _> = if self.being_edited {
            text_input("Camera name", &self.name)
                .on_input(Message::EditCameraName)
                .on_submit(Message::<State>::SubmitCameraName)
                .into()
        } else {
            text(&self.name).into()
        };

        row![
            name_field,
            Space::with_width(3),
            // edit button
            material_icon_button(MaterialIcon::Edit, MaterialIconStyle::Light, ui_size)
                .on_press(Message::<State>::StartEditCameraName(self.camera_id)),
            //
            Space::with_width(Length::Fill),
            //select camera button
            material_icon_button(MaterialIcon::Visibility, MaterialIconStyle::Light, ui_size)
                .on_press(Message::<State>::SelectCamera(self.camera_id)),
            // delete button
            material_icon_button(MaterialIcon::Delete, MaterialIconStyle::Light, ui_size)
                .on_press(Message::<State>::DeleteCamera(self.camera_id)),
        ]
        .into()
    }
}

#[derive(Debug, Clone, Default)]
struct CameraWidgetState {
    name_input: text_input::State<iced_graphics::text::Paragraph>,
}

// NOTE: Messages used here:
//                            Message::FixPoint(direction, up),
//                            Message::RotateCam(angle_xz, angle_yz, angle_xy)
//                            Message::ScreenShot2D,
//                            Message::ScreenShot3D,
//                            Message::StlExport,
//                            Message::SaveNucleotidesPositions,
//                            Message::NewCustomCamera,
//                            Message::EditCameraName
//                            Message::SubmitCameraName
//                            Message::StartEditCameraName(camera_id),
//                            Message::SelectCamera(camera_id),
//                            Message::DeleteCamera(camera_id),

pub struct CameraShortcutPanel {
    // Camera angles
    xz: isize,
    yz: isize,
    xy: isize,
    scroll_state: scrollable::State,
    camera_input_name: Option<String>,
    camera_being_edited: Option<CameraId>,
    camera_widgets: Vec<CameraWidget>,
    camera_widget_states: Vec<CameraWidgetState>,
}

impl CameraShortcutPanel {
    pub fn new() -> Self {
        Self {
            xz: 0,
            yz: 0,
            xy: 0,
            scroll_state: Default::default(),
            camera_input_name: None,
            camera_being_edited: None,
            camera_widgets: vec![],
            camera_widget_states: Default::default(),
        }
    }

    pub fn reset_angles(&mut self) {
        self.xz = 0;
        self.yz = 0;
        self.xy = 0
    }

    pub fn set_angles(&mut self, xz: isize, yz: isize, xy: isize) {
        self.xz += xz;
        self.yz += yz;
        self.xy += xy;
    }

    pub fn set_camera_input_name(&mut self, name: String) {
        self.camera_input_name = Some(name);
    }

    pub fn stop_editing(&mut self) -> Option<(CameraId, String)> {
        let name = self.camera_input_name.take();
        let id = self.camera_being_edited.take();
        for s in self.camera_widget_states.iter_mut() {
            s.name_input.unfocus();
        }
        id.zip(name)
    }

    pub fn start_editing(&mut self, id: CameraId) {
        for (c, s) in self
            .camera_widgets
            .iter()
            .zip(self.camera_widget_states.iter_mut())
        {
            if c.camera_id == id {
                self.camera_being_edited = Some(id);
                s.name_input.focus();
                s.name_input.select_all();
            }
        }
    }

    pub fn has_keyboard_priority(&self) -> bool {
        self.camera_widget_states
            .iter()
            .any(|s| s.name_input.is_focused())
    }

    fn set_camera_widget<State: AppState>(&mut self, app: &State) {
        self.camera_widgets = app
            .get_reader()
            .get_all_cameras()
            .iter()
            .map(|cam| {
                let being_edited = self.camera_being_edited == Some(cam.0);
                let name = if being_edited {
                    self.camera_input_name
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or(cam.1)
                } else {
                    cam.1
                };
                CameraWidget {
                    name: name.to_string(),
                    being_edited,
                    camera_id: cam.0,
                }
            })
            .collect();
    }

    pub fn scroll_down(&mut self) {
        self.scroll_state.snap_to(scrollable::RelativeOffset::END);
    }
}

// TODO: Implement the Component trait.
//
//impl<State> Component<Message<State>, crate::Theme, crate::Renderer> for CameraShortcutPanel
//where
//    State: AppState,
//{
//    type State = (UiSize, State);
//    type Event = ();
//
//    fn update(&mut self, state: &mut Self::State, _: Self::Event) -> Option<Message<State>> {
//        let (_, app_state) = state;
//        self.set_camera_widget(app_state);
//        None
//    }
impl CameraShortcutPanel {
    pub fn update<State: AppState>(&mut self, app_state: &State) {
        self.set_camera_widget(app_state);
    }

    pub fn view<State: AppState>(
        &self,
        ui_size: UiSize,
        _state: &State,
    ) -> Element<'_, Message<State>> {
        //let (ui_size, _) = state;
        //let ui_size = ui_size.to_owned();

        // Create button widget for each predefined target.

        let rotate_buttons: Column<Message<State>, crate::Theme, crate::Renderer> = self::column![
            row(IntoIterator::into_iter([4, 2, 5]).map(|i| {
                rotation_icon_button(i, ui_size)
                    .on_press(rotation_message(i, self.xz, self.yz, self.xy))
                    .into()
            }))
            .spacing(ui_size.button_spacing()),
            row(IntoIterator::into_iter([0, 3, 1]).map(|i| {
                rotation_icon_button(i, ui_size)
                    .on_press(rotation_message(i, self.xz, self.yz, self.xy))
                    .into()
            }))
            .spacing(ui_size.button_spacing()),
        ]
        .spacing(ui_size.button_spacing());

        //let mut ret = Column::new();
        //while rotate_buttons.len() > 0 {
        //    let mut row = Row::new();
        //    row = row.push(rotate_buttons.remove(0)).spacing(5);
        //    let mut space = ui_size.button() + 5;
        //    while space + ui_size.button() < width && rotate_buttons.len() > 0 {
        //        row = row.push(rotate_buttons.remove(0)).spacing(5);
        //        space += ui_size.button() + 5;
        //    }
        //    ret = ret.spacing(5).push(row)
        //}
        // TODO: Reimplement this with:
        //  https://docs.rs/iced/latest/iced/advanced/layout/flex/index.html

        let content = self::column![
            self::column![
                section("Camera", ui_size),
                row![
                    Space::with_width(ui_size.button_spacing()),
                    // add_target_buttons!
                    self::column![
                        subsection("Fixed", ui_size)
                            .height(ui_size.button())
                            .horizontal_alignment(Horizontal::Center),
                        extra_jump(),
                        row![
                            self::column![
                                named_camera_to_button(&PREDEFINED_CAMERA_ORIENTATION[0], ui_size),
                                named_camera_to_button(&PREDEFINED_CAMERA_ORIENTATION[1], ui_size),
                            ]
                            .spacing(ui_size.button_spacing()),
                            self::column![
                                named_camera_to_button(&PREDEFINED_CAMERA_ORIENTATION[2], ui_size),
                                named_camera_to_button(&PREDEFINED_CAMERA_ORIENTATION[3], ui_size),
                            ]
                            .spacing(ui_size.button_spacing()),
                            self::column![
                                named_camera_to_button(&PREDEFINED_CAMERA_ORIENTATION[4], ui_size),
                                named_camera_to_button(&PREDEFINED_CAMERA_ORIENTATION[5], ui_size),
                            ]
                            .spacing(ui_size.button_spacing()),
                        ]
                        .spacing(ui_size.button_spacing()),
                    ]
                    .align_items(Alignment::Center),
                    Space::with_width(2.0 * ui_size.button_spacing()),
                    // add_rotate_buttons!
                    self::column![
                        subsection("Rotation", ui_size)
                            .height(ui_size.button())
                            .horizontal_alignment(Horizontal::Center),
                        extra_jump(),
                        rotate_buttons,
                    ]
                    .align_items(Alignment::Center),
                    Space::with_width(2.0 * ui_size.button_spacing()),
                    // add_screenshot_button!
                    self::column![
                        material_icon(MaterialIcon::PhotoCamera, MaterialIconStyle::Dark, ui_size)
                            .height(ui_size.button()),
                        extra_jump(),
                        self::column![
                            fixed_text_button("2D", 1.0, ui_size).on_press(Message::ScreenShot2D),
                            fixed_text_button("3D", 1.0, ui_size).on_press(Message::ScreenShot3D),
                        ]
                        .spacing(ui_size.button_spacing()),
                    ]
                    .align_items(Alignment::Center),
                    Space::with_width(2.0 * ui_size.button_spacing()),
                    // add_stl_export_button!
                    // add_nucleotides_positions_export_button!
                    self::column![
                        Space::with_height(ui_size.button()),
                        extra_jump(),
                        self::column![
                            fixed_text_button("STL", 2.0, ui_size).on_press(Message::StlExport),
                            fixed_text_button("Nucl", 2.0, ui_size)
                                .on_press(Message::SaveNucleotidesPositions),
                        ]
                        .spacing(ui_size.button_spacing()),
                    ]
                    .align_items(Alignment::End),
                    Space::with_width(ui_size.button_spacing()),
                ]
                .align_items(Alignment::Center),
            ]
            .align_items(Alignment::Center),
            self::column![
                // add_custom_camera_row!
                row![
                    section("Custom cameras", ui_size),
                    Space::with_width(ui_size.button_spacing()),
                    material_icon_button(
                        MaterialIcon::AddAPhoto,
                        MaterialIconStyle::Light,
                        ui_size
                    )
                    .on_press(Message::NewCustomCamera),
                ],
                // add_camera_widgets!
                Column::with_children(self.camera_widgets.iter().map(|w| w.view(ui_size)))
            ]
            .align_items(Alignment::Center)
            .width(Length::Fill),
        ]
        .align_items(Alignment::Center)
        .spacing(20.0);

        scrollable(content).into()
        // NOTE: Background and size are handled in left_panel.rs
    }
}
