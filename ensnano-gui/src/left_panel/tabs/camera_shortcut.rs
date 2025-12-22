use std::f32::consts::PI;

use crate::{
    AppState, CameraId,
    fonts::material_icons::{MaterialIcon, MaterialIconStyle},
    helpers::{
        button_text_wrapper, extra_jump, fixed_text_button, material_icon, material_icon_button,
        section, subsection,
    },
    left_panel::Message,
};
use ensnano_iced::{ui_size::UiSize, widgets::keyboard_priority::keyboard_priority};
use iced::{
    Alignment, Command, Length,
    alignment::Horizontal,
    widget::{Column, Space, column, row, scrollable, text, text_input},
};
use ultraviolet::Vec3;

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
    /// Generate a message to set camera to desired position.
    fn message<State: AppState>(&self) -> Message<State> {
        Message::FixPoint(self.direction, self.up)
    }

    /// Turn a NamedCameraPosition into a button.
    fn button<State: AppState>(&self, ui_size: UiSize) -> iced::Element<'_, Message<State>> {
        fixed_text_button(self.name, 2.0, ui_size)
            .on_press(self.message())
            .into()
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

#[derive(Debug, Clone, Copy)]
enum Rotation {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl Rotation {
    /// Generate the message that request rotation.
    fn message<State: AppState>(&self) -> Message<State> {
        const ROTATION_AMOUNT: f32 = PI / 12.;
        let (angle_x, angle_y, angle_z) = match self {
            Self::PositiveX => (ROTATION_AMOUNT, 0., 0.),
            Self::NegativeX => (-ROTATION_AMOUNT, 0., 0.),
            Self::PositiveY => (0., ROTATION_AMOUNT, 0.),
            Self::NegativeY => (0., -ROTATION_AMOUNT, 0.),
            Self::PositiveZ => (0., 0., ROTATION_AMOUNT),
            Self::NegativeZ => (0., 0., -ROTATION_AMOUNT),
        };
        // TODO: x, y, z
        Message::RotateCam(angle_y, angle_x, angle_z)
    }

    fn button<'a, State: AppState>(self, ui_size: UiSize) -> iced::Element<'a, Message<State>> {
        let icon = match self {
            Self::NegativeY => MaterialIcon::ArrowBack,
            Self::PositiveY => MaterialIcon::ArrowForward,
            Self::NegativeX => MaterialIcon::ArrowUpward,
            Self::PositiveX => MaterialIcon::ArrowDownward,
            Self::NegativeZ => MaterialIcon::Undo,
            Self::PositiveZ => MaterialIcon::Redo,
        };

        button_text_wrapper!(
            material_icon(icon, MaterialIconStyle::Dark, ui_size).height(ui_size.button()),
            ui_size
        )
        .on_press(self.message())
        .into()
    }
}

// Custom camera editor.
struct CameraWidget {
    // Name of the custom camera orientation.
    name: String,
    // Whether the name is being edited.
    being_edited: bool,
    // An id for this camera.
    camera_id: CameraId,
}

impl CameraWidget {
    fn view<State: AppState>(&self, ui_size: UiSize) -> iced::Element<'_, Message<State>> {
        let name_field: iced::Element<'_, _> = if self.being_edited {
            keyboard_priority(
                "Camera name",
                Message::SetKeyboardPriority,
                text_input("Camera name", &self.name)
                    .on_input(Message::EditCameraName)
                    .on_submit(Message::SubmitCameraName),
            )
            .into()
        } else {
            text(&self.name).into()
        };

        row![
            name_field,
            Space::with_width(3),
            // edit button
            material_icon_button(MaterialIcon::Edit, MaterialIconStyle::Light, ui_size)
                .on_press(Message::StartEditCameraName(self.camera_id)),
            //
            Space::with_width(Length::Fill),
            //select camera button
            material_icon_button(MaterialIcon::Visibility, MaterialIconStyle::Light, ui_size)
                .on_press(Message::SelectCamera(self.camera_id)),
            // delete button
            material_icon_button(MaterialIcon::Delete, MaterialIconStyle::Light, ui_size)
                .on_press(Message::DeleteCamera(self.camera_id)),
        ]
        .into()
    }
}

pub struct CameraShortcutPanel {
    width: u16,
    // Camera angles
    xz: isize,
    yz: isize,
    xy: isize,
    scroll_state: scrollable::State,
    camera_input_name: Option<String>,
    camera_being_edited: Option<CameraId>,
    camera_widgets: Vec<CameraWidget>,
}

impl CameraShortcutPanel {
    pub fn new(width: u16) -> Self {
        Self {
            width,
            xz: 0,
            yz: 0,
            xy: 0,
            scroll_state: Default::default(),
            camera_input_name: None,
            camera_being_edited: None,
            camera_widgets: vec![],
        }
    }

    pub fn new_width(&mut self, width: u16) {
        self.width = width;
    }

    pub fn reset_angles(&mut self) {
        self.xz = 0;
        self.yz = 0;
        self.xy = 0;
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
        id.zip(name)
    }

    pub fn start_editing(&mut self, id: CameraId) {
        for cam in &self.camera_widgets {
            if cam.camera_id == id {
                self.camera_being_edited = Some(id);
            }
        }
    }

    fn set_camera_widget<State: AppState>(&mut self, app: &State) {
        self.camera_widgets = app
            .get_reader()
            .get_all_cameras()
            .into_iter()
            .map(|(id, name)| {
                let being_edited = self.camera_being_edited == Some(id);
                let name = if being_edited {
                    self.camera_input_name.as_deref().unwrap_or(name)
                } else {
                    name
                };
                CameraWidget {
                    name: name.to_owned(),
                    being_edited,
                    camera_id: id,
                }
            })
            .collect();
    }

    pub fn scroll_down(&mut self) {
        self.scroll_state.snap_to(scrollable::RelativeOffset::END);
    }

    pub fn update<State: AppState>(&mut self, app_state: &State) -> Command<Message<State>> {
        self.set_camera_widget(app_state);
        Command::none()
    }

    pub fn view<State: AppState>(&self, ui_size: UiSize) -> iced::Element<'_, Message<State>> {
        const ROTATION_GRID: [[Rotation; 3]; 2] = [
            [
                Rotation::NegativeZ,
                Rotation::NegativeX,
                Rotation::PositiveZ,
            ],
            [
                Rotation::NegativeY,
                Rotation::PositiveX,
                Rotation::PositiveY,
            ],
        ];

        let content = column![
            column![
                section("Camera", ui_size),
                Space::with_width(ui_size.button_spacing()),
                // add_target_buttons!
                column![
                    subsection("Fixed", ui_size)
                        .height(ui_size.button())
                        .horizontal_alignment(Horizontal::Center),
                    extra_jump(),
                    row![
                        column![
                            PREDEFINED_CAMERA_ORIENTATION[0].button(ui_size),
                            PREDEFINED_CAMERA_ORIENTATION[1].button(ui_size),
                        ]
                        .spacing(ui_size.button_spacing()),
                        column![
                            PREDEFINED_CAMERA_ORIENTATION[2].button(ui_size),
                            PREDEFINED_CAMERA_ORIENTATION[3].button(ui_size),
                        ]
                        .spacing(ui_size.button_spacing()),
                        column![
                            PREDEFINED_CAMERA_ORIENTATION[4].button(ui_size),
                            PREDEFINED_CAMERA_ORIENTATION[5].button(ui_size),
                        ]
                        .spacing(ui_size.button_spacing()),
                    ]
                    .spacing(ui_size.button_spacing()),
                ]
                .align_items(Alignment::Center),
                Space::with_height(2.0 * ui_size.button_spacing()),
                row![
                    // add_rotate_buttons!
                    column![
                        subsection("Rotation", ui_size)
                            .height(ui_size.button())
                            .horizontal_alignment(Horizontal::Center),
                        extra_jump(),
                        column(ROTATION_GRID.iter().map(|rotation_row| {
                            row(rotation_row.iter().map(|rotation| rotation.button(ui_size)))
                                .spacing(ui_size.button_spacing())
                                .into()
                        }))
                        .spacing(ui_size.button_spacing()),
                    ]
                    .align_items(Alignment::Center),
                    Space::with_width(2.0 * ui_size.button_spacing()),
                    // add_screenshot_button!
                    column![
                        material_icon(MaterialIcon::PhotoCamera, MaterialIconStyle::Dark, ui_size)
                            .height(ui_size.button()),
                        extra_jump(),
                        column![
                            fixed_text_button("2D", 1.0, ui_size).on_press(Message::ScreenShot2D),
                            fixed_text_button("3D", 1.0, ui_size).on_press(Message::ScreenShot3D),
                        ]
                        .spacing(ui_size.button_spacing()),
                    ]
                    .align_items(Alignment::Center),
                    Space::with_width(2.0 * ui_size.button_spacing()),
                    // add_stl_export_button!
                    // add_nucleotides_positions_export_button!
                    column![
                        Space::with_height(ui_size.button()),
                        extra_jump(),
                        column![
                            fixed_text_button("STL", 2.0, ui_size).on_press(Message::StlExport),
                            fixed_text_button("Nucl", 2.0, ui_size)
                                .on_press(Message::SaveNucleotidesPositions),
                        ]
                        .spacing(ui_size.button_spacing()),
                    ]
                    .align_items(Alignment::End),
                    Space::with_width(ui_size.button_spacing()),
                ],
            ]
            .align_items(Alignment::Center),
            // add_custom_camera_row!
            row![
                section("Custom cameras", ui_size),
                Space::with_width(ui_size.button_spacing()),
                material_icon_button(MaterialIcon::AddAPhoto, MaterialIconStyle::Light, ui_size)
                    .on_press(Message::NewCustomCamera),
            ],
            // add_camera_widgets!
            Column::with_children(self.camera_widgets.iter().map(|w| w.view(ui_size)))
        ]
        .align_items(Alignment::Center)
        .width(Length::Fill)
        .max_width(self.width - 2)
        .spacing(20.0);

        scrollable(content).into()
        // NOTE: Background and size are handled in left_panel.rs
    }
}
