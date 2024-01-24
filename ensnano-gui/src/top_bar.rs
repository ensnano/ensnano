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
use super::{AppState, TopBarState, UiSize};
// NOTE: I would like to rename AppState to ApplicationState, and name AppState the structures that
//       implement it.
use crate::helpers::*;
use ensnano_interactor::{ActionMode, SelectionMode};
use iced::{theme, Element, Length, Padding};
use iced_native::widget::{self, helpers::*};
use iced_native::{Command, Program};
use iced_wgpu;
use iced_winit::winit::dpi::LogicalSize;
//use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use super::material_icons_light::LightIcon;

use super::{Requests, SplitMode};

/// Top bar object
pub struct TopBar<R: Requests, S: AppState> {
    /// ENSnano request to which forwards messages.
    requests: Arc<Mutex<R>>,
    logical_size: LogicalSize<f64>,
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
    Split2d,
    // Receive an new application state.
    NewApplicationState(S),
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
    type Renderer = iced_wgpu::Renderer;
    type Message = Message<S>;

    fn update(&mut self, message: Message<S>) -> Command<Message<S>> {
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
            Message::Split2d => self.requests.lock().unwrap().toggle_2d_view_split(),
            Message::NewApplicationState(app_state) => self.app_state = app_state,
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
            Message::ThickHelices(b) => self.requests.lock().unwrap().set_thick_helices(b),
            Message::AlignHorizon => self.requests.lock().unwrap().align_horizon(),
            Message::Import3D => self.requests.lock().unwrap().import_3d_object(),
        };
        Command::none()
    }

    fn view(&self) -> Element<Message<S>, Self::Renderer> {
        let build_helix_mode = self.get_build_helix_mode();
        // List of action modes to add in the top bar.
        let action_modes_to_display = [
            ActionMode::Normal,
            ActionMode::Translate,
            ActionMode::Rotate,
            build_helix_mode.clone(),
        ];
        let height = self.logical_size.cast::<f32>().height;
        let button_fit = light_icon_button(LightIcon::ViewInAr, self.ui_size)
            .on_press(Message::SceneFitRequested)
            .height(Length::Fixed(height));

        let button_horizon =
            light_icon_button(LightIcon::WbTwilight, self.ui_size).on_press(Message::AlignHorizon);

        let button_new_empty_design = tooltip(
            light_icon_button(LightIcon::InsertDriveFile, self.ui_size)
                .on_press(Message::ButtonNewEmptyDesignPressed),
            "Start a new empty design.",
            widget::tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_add_file = tooltip(
            light_icon_button(LightIcon::FolderOpen, self.ui_size)
                .on_press(Message::OpenFileButtonPressed),
            "Add file.",
            widget::tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let mut button_reload = light_icon_button(LightIcon::RestorePage, self.ui_size);

        if self.state.can_reload {
            button_reload = button_reload.on_press(Message::Reload);
        }

        /*
        let button_save = bottom_tooltip_icon_btn(
            &mut self.button_save,
            MaterialIcon::Save,
            &top_size_info,
            "Save As..",
            Some(save_message),
        );*/
        let button_save = if self.state.need_save {
            dark_icon_button(LightIcon::Save, self.ui_size).on_press(Message::FileSaveRequested)
        } else {
            light_icon_button(LightIcon::Save, self.ui_size).on_press(Message::FileSaveRequested)
        };

        let button_save_as = if self.state.need_save {
            dark_icon_button(LightIcon::DriveFileMove, self.ui_size)
                .on_press(Message::SaveAsRequested)
        } else {
            light_icon_button(LightIcon::DriveFileMove, self.ui_size)
                .on_press(Message::SaveAsRequested)
        };

        let mut button_undo = dark_icon_button(LightIcon::Undo, self.ui_size);
        if self.state.can_undo {
            button_undo = button_undo.on_press(Message::Undo)
        }

        let mut button_redo = dark_icon_button(LightIcon::Redo, self.ui_size);
        if self.state.can_redo {
            button_redo = button_redo.on_press(Message::Redo)
        }

        let button_2d = tooltip(
            text_button("2D", self.ui_size).on_press(Message::ToggleView(SplitMode::Flat)),
            "Switch to flatscene only view",
            widget::tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);
        let button_3d =
            text_button("3D", self.ui_size).on_press(Message::ToggleView(SplitMode::Scene3D));
        let button_thick_helices = if self.app_state.want_thick_helices() {
            light_icon_button(LightIcon::Dehaze, self.ui_size)
                .on_press(Message::ThickHelices(false))
        } else {
            light_icon_button(LightIcon::Water, self.ui_size).on_press(Message::ThickHelices(true))
        };
        let button_split = tooltip(
            text_button("3D+2D", self.ui_size).on_press(Message::ToggleView(SplitMode::Both)),
            "Switch to both flat and 3d view",
            widget::tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_oxdna =
            light_icon_button(LightIcon::Upload, self.ui_size).on_press(Message::ExportRequested);
        let oxdna_tooltip = button_oxdna;

        let button_3d_import =
            light_icon_button(LightIcon::Coronavirus, self.ui_size).on_press(Message::Import3D);

        let split_icon = if self.state.splited_2d {
            LightIcon::BorderOuter
        } else {
            LightIcon::BorderHorizontal
        };

        let mut button_split_2d = light_icon_button(split_icon, self.ui_size);

        if self.state.can_split2d {
            button_split_2d = button_split_2d.on_press(Message::Split2d);
        }

        let mut button_toggle_2d = text_button("Toggle 2D", self.ui_size);

        if self.state.can_toggle_2d {
            button_toggle_2d = button_toggle_2d.on_press(Message::Toggle2D);
        }

        let mut button_flip_split =
            light_icon_button(LightIcon::SwapVert, self.ui_size).height(self.ui_size.button());
        if self.state.splited_2d {
            button_flip_split = button_flip_split.on_press(Message::FlipSplitViews);
        }

        let button_help = text_button("Help", self.ui_size).on_press(Message::ForceHelp);

        let button_tutorial =
            text_button("Tutorials", self.ui_size).on_press(Message::ShowTutorial);

        let ui_size = self.ui_size.clone();
        let action_mode_buttons: Vec<_> = action_modes_to_display
            .iter()
            .map(|mode| {
                action_mode_btn(
                    mode,
                    self.app_state.get_action_mode(),
                    ui_size.button(),
                    self.app_state.get_widget_basis().is_axis_aligned(),
                )
                .into()
            })
            .collect();

        // List of selection modes to add to the top bar.
        let selection_modes_to_display = [
            SelectionMode::Helix,
            SelectionMode::Strand,
            SelectionMode::Nucleotide,
        ];

        let selection_mode_buttons: Vec<_> = SelectionMode::ALL
            .iter()
            .filter(|mode| selection_modes_to_display.contains(mode))
            .map(|mode| {
                selection_mode_btn(mode, self.app_state.get_selection_mode(), ui_size.button())
                    .into()
            })
            .collect();

        let buttons = iced_native::row![
            // “File” group
            iced_native::row![
                button_new_empty_design,
                button_add_file,
                button_reload,
                button_save,
                button_save_as,
                oxdna_tooltip,
                button_3d_import,
            ],
            // “View” group
            iced_native::row![
                button_3d,
                button_thick_helices,
                button_2d,
                button_split,
                button_split_2d,
                button_toggle_2d,
                button_flip_split,
            ],
            iced_native::row![button_fit, button_horizon,],
            // “Edition” group
            iced_native::row![button_undo, button_redo,],
            // “Action” group
            row(action_mode_buttons),
            // “Selection” group
            row(selection_mode_buttons),
            iced_native::row![button_help, button_tutorial,].spacing(2),
            // ENSnano logo, placed on the right.
            text("\u{e91c}")
                .width(Length::Fill)
                .horizontal_alignment(iced::alignment::Horizontal::Right)
                .vertical_alignment(iced::alignment::Vertical::Center),
        ]
        .spacing(10) // Space between button groups.
        .width(Length::Fill)
        .height(height);

        container(buttons)
            .width(self.logical_size.width as f32)
            .style(theme::Container::Box)
            .padding(Padding::from([1, 0])) // HACK: A small padding allow tooltip messages to
            //                                       disappear properly.
            .into()
    }
}

struct ToolTipStyle;
impl widget::container::StyleSheet for ToolTipStyle {
    type Style = ();
    fn appearance(&self, _style: &Self::Style) -> widget::container::Appearance {
        widget::container::Appearance {
            text_color: Some(iced::Color::BLACK),
            ..Default::default()
        }
    }
}

//#[derive(Default, Debug, Clone)]
//struct SelectionModeState {
//    pub nucleotide: button::State,
//    pub strand: button::State,
//    pub helix: button::State,
//}
//
//impl SelectionModeState {
//    fn get_states<'a>(&'a mut self) -> BTreeMap<SelectionMode, &'a mut button::State> {
//        let mut ret = BTreeMap::new();
//        ret.insert(SelectionMode::Nucleotide, &mut self.nucleotide);
//        ret.insert(SelectionMode::Strand, &mut self.strand);
//        ret.insert(SelectionMode::Helix, &mut self.helix);
//        ret
//    }
//}
//
//#[derive(Default, Debug, Clone)]
//struct ActionModeState {
//    pub select: button::State,
//    pub translate: button::State,
//    pub rotate: button::State,
//    pub build: button::State,
//}
//
//impl ActionModeState {
//    fn get_states<'a>(
//        &'a mut self,
//        build_helix_mode: ActionMode,
//    ) -> BTreeMap<ActionMode, &'a mut button::State> {
//        let mut ret = BTreeMap::new();
//        ret.insert(ActionMode::Normal, &mut self.select);
//        ret.insert(ActionMode::Translate, &mut self.translate);
//        ret.insert(ActionMode::Rotate, &mut self.rotate);
//        ret.insert(build_helix_mode, &mut self.build);
//        ret
//    }
//}

//struct ButtonStyle(bool);
//
//impl iced_native::widget::button::StyleSheet for ButtonStyle {
//    type Style = ();
//    fn active(&self, _style: &Self::Style) -> iced_native::widget::button::Appearance {
//        iced_native::widget::button::Appearance {
//            border_width: if self.0 { 3_f32 } else { 1_f32 },
//            border_radius: if self.0 { 3_f32 } else { 2_f32 },
//            border_color: if self.0 {
//                Color::BLACK
//            } else {
//                [0.7, 0.7, 0.7].into()
//            },
//            background: Some(Background::Color([0.87, 0.87, 0.87].into())),
//            //background: Some(Background::Color(BACKGROUND)),
//            ..Default::default()
//        }
//    }
//}

//impl From<ButtonStyle> for iced::theme::Container {
//    fn from(_value: ButtonStyle) -> Self {
//        Default::default()
//    }
//}
//
//impl From<ButtonStyle> for iced::theme::Button {
//    fn from(_value: ButtonStyle) -> Self {
//        Default::default()
//    }
//}

use super::icon::{HasIcon, HasIconDependentOnAxis};
fn action_mode_btn<'a, S: AppState>(
    mode: &ActionMode,
    current_action_mode: ActionMode,
    button_size: impl Into<Length>,
    axis_aligned: bool,
) -> widget::Button<'a, Message<S>, iced_wgpu::Renderer> {
    let icon_path = if current_action_mode == *mode {
        mode.icon_on(axis_aligned)
    } else {
        mode.icon_off(axis_aligned)
    };

    button(image(icon_path))
        .on_press(Message::ActionModeChanged(mode.clone()))
        //.style(ButtonStyle(fixed_mode == mode))
        // TODO: Reimplement fixed_mode
        .width(button_size)
}

fn selection_mode_btn<'a, S: AppState>(
    mode: &SelectionMode,
    current_mode: SelectionMode,
    button_size: impl Into<Length>,
) -> widget::Button<'a, Message<S>, iced_wgpu::Renderer> {
    let icon_path = if current_mode == *mode {
        mode.icon_on()
    } else {
        mode.icon_off()
    };

    button(image(icon_path))
        .on_press(Message::SelectionModeChanged(mode.clone()))
        //.style(ButtonStyle(fixed_mode == mode))
        // TODO: Reimplement fixed_mode
        .width(button_size)
}
