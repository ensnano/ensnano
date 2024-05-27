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
use super::{rotation_message, AppState, Message, UiSize, Vec3};
use crate::helpers::*;
use crate::{
    material_icons_light::{self, LightIcon},
    CameraId,
};
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
    /// Generate message to set camera to desired position.
    fn message<S: AppState>(&self) -> Message<S> {
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
fn named_camera_to_button<'a, S: AppState>(
    position: &NamedCameraPosition,
    ui_size: UiSize,
) -> Element<'a, Message<S>> {
    button(text(position.name).size(ui_size.main_text()))
        .on_press(position.message())
        .height(ui_size.button())
        .width(2.0 * ui_size.button()) // Twice the button's height.
        .into()
}

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

    fn set_camera_widget<S: AppState>(&mut self, app: &S) {
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
                CameraWidget::new(name.to_string(), being_edited, cam.0)
            })
            .collect();
    }

    pub fn update<S: AppState>(&mut self, app: &S) {
        self.set_camera_widget(app);
    }

    pub fn view<State>(
        &self,
        ui_size: UiSize,
        _app: &State,
    ) -> Element<Message<State>, crate::Theme, crate::Renderer>
    where
        State: AppState,
    {
        // Create button widget for each predefined target.

        //let rotate_buttons = self::column![
        //    row(IntoIterator::into_iter([4, 2, 5])
        //        .map(|i| {
        //            button(rotation_text(i, ui_size))
        //                .on_press(rotation_message(i, self.xz, self.yz, self.xy))
        //                .width(ui_size.button())
        //                .into()
        //        })
        //        .collect())
        //    .spacing(ui_size.button_pad()),
        //    row(IntoIterator::into_iter([0, 3, 1])
        //        .map(|i| {
        //            button(rotation_text(i, ui_size))
        //                .on_press(rotation_message(i, self.xz, self.yz, self.xy))
        //                .width(ui_size.button())
        //                .into()
        //        })
        //        .collect())
        //    .spacing(ui_size.button_pad()),
        //]
        //.spacing(ui_size.button_pad());

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

        let content = self::column![
            self::column![
                section("Camera", ui_size),
                row![
                    Space::with_width(ui_size.button_pad()),
                    // add_target_buttons!
                    self::column![
                        subsection("Fixed", ui_size)
                            .height(ui_size.button())
                            .horizontal_alignment(Horizontal::Center),
                        extra_jump(),
                        //row![
                        //    self::column![
                        //        named_camera_to_button(&PREDEFINED_CAMERA_ORIENTATION[0], ui_size),
                        //        named_camera_to_button(&PREDEFINED_CAMERA_ORIENTATION[1], ui_size),
                        //    ]
                        //    .spacing(ui_size.button_pad()),
                        //    self::column![
                        //        named_camera_to_button(&PREDEFINED_CAMERA_ORIENTATION[2], ui_size),
                        //        named_camera_to_button(&PREDEFINED_CAMERA_ORIENTATION[3], ui_size),
                        //    ]
                        //    .spacing(ui_size.button_pad()),
                        //    self::column![
                        //        named_camera_to_button(&PREDEFINED_CAMERA_ORIENTATION[4], ui_size),
                        //        named_camera_to_button(&PREDEFINED_CAMERA_ORIENTATION[5], ui_size),
                        //    ]
                        //    .spacing(ui_size.button_pad()),
                        //]
                        //.spacing(ui_size.button_pad()),
                        //TODO: REACTIVATE ME!
                    ]
                    .align_items(Alignment::Center),
                    Space::with_width(2.0 * ui_size.button_pad()),
                    // add_rotate_buttons!
                    self::column![
                        subsection("Rotation", ui_size)
                            .height(ui_size.button())
                            .horizontal_alignment(Horizontal::Center),
                        extra_jump(),
                        //rotate_buttons,
                        //TODO: REACTIVATE ME!
                        // Idem.
                    ]
                    .align_items(Alignment::Center),
                    Space::with_width(2.0 * ui_size.button_pad()),
                    // add_screenshot_button!
                    self::column![
                        material_icons_light::dark_icon(LightIcon::PhotoCamera, ui_size)
                            .height(ui_size.button()),
                        extra_jump(),
                        self::column![
                            text_button("2D", ui_size)
                                .height(ui_size.button())
                                .on_press(Message::ScreenShot2D),
                            text_button("3D", ui_size)
                                .height(ui_size.button())
                                .on_press(Message::ScreenShot3D),
                        ]
                        .spacing(ui_size.button_pad()),
                    ]
                    .align_items(Alignment::Center),
                    Space::with_width(2.0 * ui_size.button_pad()),
                    // add_stl_export_button!
                    self::column![
                        extra_jump(),
                        self::column![text_button("STL", ui_size)
                            .width(2.0 * ui_size.button())
                            .height(ui_size.button())
                            .on_press(Message::StlExport),]
                        .spacing(ui_size.button_pad()),
                    ]
                    .align_items(Alignment::End),
                    Space::with_width(ui_size.button_pad()),
                ]
                .align_items(Alignment::Center),
            ]
            .align_items(Alignment::Center),
            self::column![
                // add_custom_camera_row!
                row![
                    section("Custom cameras", ui_size),
                    Space::with_width(ui_size.button_pad()),
                    light_icon_button(LightIcon::AddAPhoto, ui_size)
                        .on_press(Message::NewCustomCamera),
                ],
                // add_camera_widgets!
                //Column::with_children(
                //    self.camera_widgets
                //        .iter()
                //        .map(|w| w.view(ui_size).into())
                //        .collect()
                //)
                //TODO: REACTIVATE ME!
            ]
            .align_items(Alignment::Center)
            .width(Length::Fill),
        ]
        .align_items(Alignment::Center)
        .spacing(20.0);

        scrollable(content).into()
        // NOTE: Background and size are handled in left_panel.rs
    }

    pub fn scroll_down(&mut self) {
        self.scroll_state.snap_to(scrollable::RelativeOffset::END);
    }
}

// Custom camera editor.
struct CameraWidget {
    // Name of the custom camera orientation
    name: String,
    // Wether the name is being edited.
    being_edited: bool,
    camera_id: CameraId,
}

impl CameraWidget {
    fn new(name: String, being_edited: bool, camera_id: CameraId) -> Self {
        Self {
            name,
            being_edited,
            camera_id,
        }
    }

    fn view<S>(&self, ui_size: UiSize) -> Element<Message<S>>
    where
        S: AppState,
    {
        let name_field: Element<Message<S>> = if self.being_edited {
            text_input("Camera name", &self.name)
                .on_input(Message::EditCameraName)
                .on_submit(Message::<S>::SubmitCameraName)
                .into()
        } else {
            text(&self.name).into()
        };

        row![
            name_field,
            Space::with_width(3),
            // edit button
            light_icon_button(LightIcon::Edit, ui_size)
                .on_press(Message::<S>::StartEditCameraName(self.camera_id)),
            //
            Space::with_width(Length::Fill),
            //select camera button
            light_icon_button(LightIcon::Visibility, ui_size)
                .on_press(Message::<S>::SelectCamera(self.camera_id)),
            // delete button
            light_icon_button(LightIcon::Delete, ui_size)
                .on_press(Message::<S>::DeleteCamera(self.camera_id)),
        ]
        .into()
    }
}

#[derive(Debug, Clone, Default)]
struct CameraWidgetState {
    //select_camera_btn: widget::button::State,
    //edit_name_btn: widget::button::State,
    //delete_btn: widget::button::State,
    name_input: text_input::State<iced_graphics::text::Paragraph>,
}
