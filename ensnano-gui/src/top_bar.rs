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
//! Implementation of the top bar part of the GUI.
//!
//! The top bar consist of a row of buttons that covers various actions: load/save a model, change
//! the selection mode, modify the layout of the window, etc.
//!
//! Drawing the top bar, and triggering events from it is handled here.
use super::{AppState, TopBarState};
// NOTE: I would like to rename AppState to ApplicationState, and name AppState the structures that
//       implement it.
use ensnano_iced::{
    fonts::{MaterialIcon, MaterialIconStyle},
    helpers::*,
    iced::{self, color, Element, Length, Padding},
    iced_runtime::{Command, Program},
    iced_winit::winit::dpi::LogicalSize,
    icon_to_svg, icondata, UiSize,
};
use ensnano_interactor::{ActionMode, SelectionMode};
use std::sync::{Arc, Mutex};

use super::{Requests, SplitMode};

/// Top bar object
pub struct TopBar<R: Requests, S: AppState> {
    /// ENSnano requests handle to which forwards messages.
    requests: Arc<Mutex<R>>,
    /// Area occupied by the top bar.
    logical_size: LogicalSize<f64>,
    /// A copy of the UI sizes.
    ui_size: UiSize,
    /// State of the whole application.
    app_state: S,
    /// More local state stuff.
    state: TopBarState,
}

#[derive(Debug, Clone)]
pub enum Message<S: AppState> {
    SceneFitRequested,
    AlignHorizon,
    OpenFileButtonPressed,
    /// Request to save file, e.g. clicked on “Save” button
    FileSaveRequested,
    /// Request to save file as, e.g. clicked on “Save As” button
    SaveAsRequested,
    Resize(LogicalSize<f64>),
    ToggleView(SplitMode),
    UiSizeChanged(UiSize),
    ExportRequested,
    Split2D,
    // Receive an new application state.
    NewApplicationState((S, TopBarState)),
    ForceHelp,
    ShowTutorial,
    Undo,
    Redo,
    ButtonNewEmptyDesignPressed,
    ActionModeChanged(ActionMode),
    SelectionModeChanged(SelectionMode),
    Toggle2D,
    Reload,
    FlipSplitViews,
    ThickHelices(bool),
    Import3D,
}

impl<R: Requests, S: AppState> TopBar<R, S> {
    pub fn new(
        requests: Arc<Mutex<R>>,
        logical_size: LogicalSize<f64>,
        app_state: S,
        state: TopBarState,
        ui_size: UiSize,
    ) -> Self {
        Self {
            requests,
            logical_size,
            ui_size,
            app_state,
            state,
        }
    }

    // Set the top bar to `logical_size`.
    pub fn resize(&mut self, logical_size: LogicalSize<f64>) {
        self.logical_size = logical_size;
    }

    fn get_build_helix_mode(&self) -> ActionMode {
        self.app_state.get_build_helix_mode()
    }
}

impl<R: Requests, S: AppState> Program for TopBar<R, S> {
    type Message = Message<S>;
    type Theme = ensnano_iced::Theme;
    type Renderer = crate::Renderer;

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::SceneFitRequested => {
                self.requests.lock().unwrap().fit_design_in_scenes();
            }
            Message::OpenFileButtonPressed => {
                self.requests.lock().unwrap().open_file();
            }
            Message::FileSaveRequested => {
                self.requests.lock().unwrap().save();
            }
            Message::SaveAsRequested => {
                self.requests.lock().unwrap().save_as();
            }
            Message::Resize(size) => self.resize(size),
            Message::ToggleView(b) => self.requests.lock().unwrap().change_split_mode(b),
            Message::UiSizeChanged(ui_size) => self.ui_size = ui_size,
            Message::ExportRequested => self.requests.lock().unwrap().set_exporting(true),
            Message::Split2D => self.requests.lock().unwrap().toggle_2d_view_split(),
            Message::NewApplicationState((app_state, state)) => {
                self.app_state = app_state;
                self.state = state;
            }
            Message::Undo => self.requests.lock().unwrap().undo(),
            Message::Redo => self.requests.lock().unwrap().redo(),
            Message::ForceHelp => self.requests.lock().unwrap().force_help(),
            Message::ShowTutorial => self.requests.lock().unwrap().show_tutorial(),
            Message::ButtonNewEmptyDesignPressed => self.requests.lock().unwrap().new_design(),
            Message::Reload => self.requests.lock().unwrap().reload_file(),
            Message::SelectionModeChanged(selection_mode) => {
                if selection_mode != self.app_state.get_selection_mode() {
                    self.requests
                        .lock()
                        .unwrap()
                        .change_selection_mode(selection_mode);
                }
            }
            Message::ActionModeChanged(action_mode) => {
                if self.app_state.get_action_mode() != action_mode {
                    self.requests
                        .lock()
                        .unwrap()
                        .change_action_mode(action_mode)
                } else {
                    match action_mode {
                        ActionMode::Rotate | ActionMode::Translate => {
                            self.requests.lock().unwrap().toggle_widget_basis();
                        }
                        _ => (),
                    }
                }
            }
            Message::Toggle2D => {
                self.requests.lock().unwrap().toggle_2d();
            }
            Message::FlipSplitViews => self.requests.lock().unwrap().flip_split_views(),
            Message::ThickHelices(b) => self.requests.lock().unwrap().set_all_helices_on_axis(b),
            // TODO: Consider rename message ThickHelices → AllHelicesOnAxis
            Message::AlignHorizon => self.requests.lock().unwrap().align_horizon(),
            Message::Import3D => self.requests.lock().unwrap().import_3d_object(),
        };
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Self::Renderer> {
        let build_helix_mode = self.get_build_helix_mode();

        let button_new_empty_design = tooltip(
            material_icon_button(
                MaterialIcon::InsertDriveFile,
                MaterialIconStyle::Light,
                self.ui_size,
            )
            .on_press(Message::ButtonNewEmptyDesignPressed),
            "Start a new design",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_add_file = tooltip(
            material_icon_button(
                MaterialIcon::FolderOpen,
                MaterialIconStyle::Light,
                self.ui_size,
            )
            .on_press(Message::OpenFileButtonPressed),
            "Open file…",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_reload = tooltip(
            material_icon_button(
                MaterialIcon::RestorePage,
                MaterialIconStyle::Light,
                self.ui_size,
            )
            .on_press_maybe(self.state.can_reload.then_some(Message::Reload)),
            "Reload file",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_save = tooltip(
            material_icon_button(
                MaterialIcon::Save,
                if self.state.need_save {
                    MaterialIconStyle::Dark
                } else {
                    MaterialIconStyle::Light
                },
                self.ui_size,
            )
            .on_press(Message::FileSaveRequested),
            "Save design…",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_save_as = tooltip(
            material_icon_button(
                MaterialIcon::DriveFileMove,
                if self.state.need_save {
                    MaterialIconStyle::Dark
                } else {
                    MaterialIconStyle::Light
                },
                self.ui_size,
            )
            .on_press(Message::SaveAsRequested),
            "Save design to…",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_oxdna = tooltip(
            material_icon_button(MaterialIcon::Upload, MaterialIconStyle::Light, self.ui_size)
                .on_press(Message::ExportRequested),
            "Export",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let oxdna_tooltip = button_oxdna;

        let button_3d_import = tooltip(
            material_icon_button(
                MaterialIcon::Coronavirus,
                MaterialIconStyle::Light,
                self.ui_size,
            )
            .on_press(Message::Import3D),
            "Import 3D",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_2d = tooltip(
            fixed_text_button("2D", 1.0, self.ui_size)
                .on_press(Message::ToggleView(SplitMode::Flat)),
            "Switch to flatscene only view",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_3d = tooltip(
            fixed_text_button("3D", 1.0, self.ui_size)
                .on_press(Message::ToggleView(SplitMode::Scene3D)),
            "Switch to scene only view",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        // TODO: Consider rename thick_helices → all_helices_on_axis
        let button_thick_helices = tooltip(
            if self.app_state.want_all_helices_on_axis() {
                material_icon_button(MaterialIcon::Dehaze, MaterialIconStyle::Light, self.ui_size)
                    .on_press(Message::ThickHelices(false))
            } else {
                material_icon_button(MaterialIcon::Water, MaterialIconStyle::Light, self.ui_size)
                    .on_press(Message::ThickHelices(true))
            },
            "Toggle helices",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        // TODO: Remove 3D+2D button.

        // WARN: More tricky than expected: in 3D and 2D button we will need to get the current
        //       split mode, but it is defined in Multiplexer and not directly accessible. Need
        //       to find an elegent way to do this.

        let button_split = tooltip(
            text_button("3D+2D", self.ui_size).on_press(Message::ToggleView(SplitMode::Both)),
            "Switch to both flat and 3d view",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_split_2d = tooltip(
            material_icon_button(
                if self.state.splited_2d {
                    MaterialIcon::BorderOuter
                } else {
                    MaterialIcon::BorderHorizontal
                },
                MaterialIconStyle::Light,
                self.ui_size,
            )
            .on_press_maybe(self.state.can_split_2d.then_some(Message::Split2D)),
            "Toggle split of flat scene",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_toggle_2d = tooltip(
            text_button("Toggle 2D", self.ui_size)
                .on_press_maybe(self.state.can_toggle_2d.then_some(Message::Toggle2D)),
            "Toggle flat view",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_flip_split = tooltip(
            material_icon_button(
                MaterialIcon::SwapVert,
                MaterialIconStyle::Light,
                self.ui_size,
            )
            .on_press_maybe(self.state.splited_2d.then_some(Message::FlipSplitViews)),
            "Swap flat views",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_fit = tooltip(
            material_icon_button(
                MaterialIcon::ViewInAr,
                MaterialIconStyle::Light,
                self.ui_size,
            )
            .on_press(Message::SceneFitRequested),
            "Request fit",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_horizon = tooltip(
            material_icon_button(
                MaterialIcon::WbTwilight,
                MaterialIconStyle::Light,
                self.ui_size,
            )
            .on_press(Message::AlignHorizon),
            "Align horizon",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_undo = tooltip(
            material_icon_button(MaterialIcon::Undo, MaterialIconStyle::Dark, self.ui_size)
                .on_press_maybe(self.state.can_undo.then_some(Message::Undo)),
            "Undo",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_redo = tooltip(
            material_icon_button(MaterialIcon::Redo, MaterialIconStyle::Dark, self.ui_size)
                .on_press_maybe(self.state.can_redo.then_some(Message::Redo)),
            "Redo",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        //NOTE: List of action modes to add in the top bar.
        let action_modes_to_display = [
            ActionMode::Normal,
            ActionMode::Translate,
            ActionMode::Rotate,
            build_helix_mode.clone(),
        ];

        let action_mode_buttons: Vec<Element<'_, _, _, _>> = action_modes_to_display
            .iter()
            .map(|mode| {
                tooltip(
                    action_mode_btn(
                        mode,
                        self.app_state.get_action_mode(),
                        self.ui_size.button(),
                        self.app_state.get_widget_basis().is_axis_aligned(),
                        self.ui_size,
                    ),
                    mode.tooltip_description(),
                    tooltip::Position::FollowCursor,
                )
                .style(theme::Container::Box)
                .into()
            })
            .collect();

        //NOTE: List of selection modes to add to the top bar.
        let selection_modes_to_display = [
            SelectionMode::Helix,
            SelectionMode::Strand,
            SelectionMode::Nucleotide,
        ];

        let selection_mode_buttons: Vec<Element<'_, _, _, _>> = SelectionMode::ALL
            .iter()
            .filter(|mode| selection_modes_to_display.contains(mode))
            .map(|mode| {
                tooltip(
                    selection_mode_btn(mode, self.app_state.get_selection_mode(), self.ui_size),
                    mode.tooltip_description(),
                    tooltip::Position::FollowCursor,
                )
                .style(theme::Container::Box)
                .into()
            })
            .collect();

        let button_help = text_button("Help", self.ui_size).on_press(Message::ForceHelp);

        let button_tutorial =
            text_button("Tutorials", self.ui_size).on_press(Message::ShowTutorial);

        let bar = row![
            // “File” group
            row![
                button_new_empty_design,
                button_add_file,
                button_reload,
                button_save,
                button_save_as,
                oxdna_tooltip,
                button_3d_import,
            ]
            .spacing(self.ui_size.button_spacing()),
            // “View” group
            row![
                button_3d,
                button_thick_helices,
                button_2d,
                button_split,
                button_split_2d,
                button_toggle_2d,
                button_flip_split,
            ]
            .spacing(self.ui_size.button_spacing()),
            row![button_fit, button_horizon,].spacing(self.ui_size.button_spacing()),
            // “Edition” group
            row![button_undo, button_redo,].spacing(self.ui_size.button_spacing()),
            // “Action” group
            Row::from_vec(action_mode_buttons).spacing(self.ui_size.button_spacing()),
            // “Selection” group
            Row::from_vec(selection_mode_buttons).spacing(self.ui_size.button_spacing()),
            row![button_help, button_tutorial,].spacing(self.ui_size.button_spacing()),
            // TODO: delete this test
            row![icon_to_svg(icondata::MdiYoutubeStudio)
                .width(Length::Fixed(32.0))
                .height(Length::Fixed(32.0))
                .style(theme::Svg::custom_fn(|_theme| svg::Appearance {
                    color: Some(color!(0xff0000)),
                }))]
            .spacing(self.ui_size.button_spacing()),
            // ENSnano logo, placed on the right.
            text("\u{e91c}")
                .font(crate::fonts::ENSNANO_FONT)
                .width(Length::Fill)
                .horizontal_alignment(iced::alignment::Horizontal::Right)
                .vertical_alignment(iced::alignment::Vertical::Center),
        ]
        .spacing(self.ui_size.button_group_spacing())
        .width(Length::Fill);

        container(bar)
            .width(self.logical_size.width as f32)
            .style(ensnano_iced::theme::GuiBackground)
            // HACK: A small padding allow tooltip messages to disappear properly.
            .padding(Padding::from([
                self.ui_size.button_spacing(),
                0.0,
                self.ui_size.button_spacing(),
                self.ui_size.button_spacing(),
            ]))
            .into()
    }
}

use super::icon::{HasIcon, HasIconDependentOnAxis};
fn action_mode_btn<'a, State: AppState>(
    mode: &ActionMode,
    current_action_mode: ActionMode,
    _button_size: impl Into<Length>,
    axis_aligned: bool,
    ui_size: UiSize,
) -> Button<'a, Message<State>, ensnano_iced::Theme, crate::Renderer> {
    let icon_path = if current_action_mode == *mode {
        mode.icon_on(axis_aligned)
    } else {
        mode.icon_off(axis_aligned)
    };

    image_button(image(icon_path), ui_size)
        .on_press(Message::ActionModeChanged(mode.clone()))
        .style(if current_action_mode == *mode {
            theme::Button::Positive
        } else {
            theme::Button::Primary
        })
    // TODO: Use SelectionMode Copy trait.
}

fn selection_mode_btn<'a, State: AppState>(
    mode: &SelectionMode,
    current_mode: SelectionMode,
    ui_size: UiSize,
) -> Button<'a, Message<State>, ensnano_iced::Theme, crate::Renderer> {
    let icon_path = if current_mode == *mode {
        mode.icon_on()
    } else {
        mode.icon_off()
    };

    image_button(image(icon_path), ui_size)
        .on_press(Message::SelectionModeChanged(mode.clone()))
        .style(if current_mode == *mode {
            theme::Button::Positive
        } else {
            theme::Button::Primary
        })
    // TODO: Use SelectionMode Copy trait.
}
