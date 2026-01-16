use crate::{
    fonts::material_icons::{MaterialIcon, MaterialIconStyle},
    helpers::{
        button_text_wrapper, extra_jump, fixed_text_button, material_icon, material_icon_button,
        section, subsection,
    },
    left_panel::Message,
    state::GuiAppState,
};
use ensnano_design::CameraId;
use ensnano_utils::{keyboard_priority::keyboard_priority, ui_size::UiSize};
use iced::{
    Alignment, Command, Length,
    alignment::Horizontal,
    widget::{Column, Space, column, row, scrollable, text, text_input},
};
use std::f32::consts::PI;
use ultraviolet::Vec3;

/// A named camera.
///
/// Orientation is defined by the direction pointed by the camera lens, and the direction pointed
/// by the top of the camera (representing the top of the screen view).
#[derive(Debug, Clone, Copy)]
enum NamedCamera {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
}

impl NamedCamera {
    fn name(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Top => "Top",
            Self::Back => "Back",
            Self::Front => "Front",
            Self::Bottom => "Bottom",
        }
    }

    fn direction(self) -> Vec3 {
        match self {
            Self::Left => Vec3::new(-1., 0., 0.),
            Self::Right => Vec3::new(1., 0., 0.),
            Self::Top => Vec3::new(0., 1., 0.),
            Self::Back => Vec3::new(0., 0., 1.),
            Self::Front => Vec3::new(0., 0., -1.),
            Self::Bottom => Vec3::new(0., -1., 0.),
        }
    }

    fn up(self) -> Vec3 {
        match self {
            Self::Top => Vec3::new(0., 0., 1.),
            Self::Bottom => Vec3::new(0., 0., -1.),
            Self::Left | Self::Right | Self::Back | Self::Front => Vec3::new(0., 1., 0.),
        }
    }

    /// Generate a message to set camera to desired position.
    fn message<State: GuiAppState>(self) -> Message<State> {
        Message::FixPoint(self.direction(), self.up())
    }

    /// Turn a [`NamedCamera`] into a button.
    fn button<'a, State: GuiAppState>(self, ui_size: UiSize) -> iced::Element<'a, Message<State>> {
        fixed_text_button(self.name(), 2.0, ui_size)
            .on_press(self.message())
            .into()
    }
}

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
    fn angles(self) -> (f32, f32, f32) {
        const ROTATION_AMOUNT: f32 = PI / 12.;
        match self {
            Self::PositiveX => (ROTATION_AMOUNT, 0., 0.),
            Self::NegativeX => (-ROTATION_AMOUNT, 0., 0.),
            Self::PositiveY => (0., ROTATION_AMOUNT, 0.),
            Self::NegativeY => (0., -ROTATION_AMOUNT, 0.),
            Self::PositiveZ => (0., 0., ROTATION_AMOUNT),
            Self::NegativeZ => (0., 0., -ROTATION_AMOUNT),
        }
    }

    /// Generate the message that request rotation.
    fn message<State: GuiAppState>(&self) -> Message<State> {
        let (x, y, z) = self.angles();
        Message::RotateCam(x, y, z)
    }

    fn button<'a, State: GuiAppState>(self, ui_size: UiSize) -> iced::Element<'a, Message<State>> {
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
    fn view<State: GuiAppState>(&self, ui_size: UiSize) -> iced::Element<'_, Message<State>> {
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
    scroll_state: scrollable::State,
    camera_input_name: Option<String>,
    camera_being_edited: Option<CameraId>,
    camera_widgets: Vec<CameraWidget>,
}

impl CameraShortcutPanel {
    pub fn new(width: u16) -> Self {
        Self {
            width,
            scroll_state: Default::default(),
            camera_input_name: None,
            camera_being_edited: None,
            camera_widgets: vec![],
        }
    }

    pub fn new_width(&mut self, width: u16) {
        self.width = width;
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

    fn set_camera_widget<State: GuiAppState>(&mut self, app: &State) {
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

    pub fn update<State: GuiAppState>(&mut self, app_state: &State) -> Command<Message<State>> {
        self.set_camera_widget(app_state);
        Command::none()
    }

    pub fn view<State: GuiAppState>(&self, ui_size: UiSize) -> iced::Element<'_, Message<State>> {
        const NAMED_CAMERA_GRID: [[NamedCamera; 3]; 2] = [
            [NamedCamera::Left, NamedCamera::Top, NamedCamera::Front],
            [NamedCamera::Right, NamedCamera::Bottom, NamedCamera::Back],
        ];

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
                    column(NAMED_CAMERA_GRID.iter().map(|camera_row| {
                        row(camera_row.iter().map(|cam| cam.button(ui_size)))
                            .spacing(ui_size.button_spacing())
                            .into()
                    }))
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
