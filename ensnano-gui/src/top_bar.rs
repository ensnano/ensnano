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
use iced_runtime::{Command, Program};
use iced_wgpu;
use iced_winit::winit::dpi::LogicalSize;
//use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

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
    type Message = Message<S>;
    type Theme = crate::Theme;
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

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Self::Renderer> {
        let build_helix_mode = self.get_build_helix_mode();
        // List of action modes to add in the top bar.
        let action_modes_to_display = [
            ActionMode::Normal,
            ActionMode::Translate,
            ActionMode::Rotate,
            build_helix_mode.clone(),
        ];
        let height = self.ui_size.button();
        let button_fit = material_icon_button(
            MaterialIcon::ViewInAr,
            MaterialIconStyle::Light,
            self.ui_size,
        )
        .on_press(Message::SceneFitRequested);

        let button_horizon = material_icon_button(
            MaterialIcon::WbTwilight,
            MaterialIconStyle::Light,
            self.ui_size,
        )
        .on_press(Message::AlignHorizon);

        let button_new_empty_design: Tooltip<'_, _, _, _> = tooltip(
            material_icon_button(
                MaterialIcon::InsertDriveFile,
                MaterialIconStyle::Light,
                self.ui_size,
            )
            .on_press(Message::ButtonNewEmptyDesignPressed),
            "Start a new design.",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_add_file: Tooltip<'_, _, _, _> = tooltip(
            material_icon_button(
                MaterialIcon::FolderOpen,
                MaterialIconStyle::Light,
                self.ui_size,
            )
            .on_press(Message::OpenFileButtonPressed),
            "Add file.",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_reload = material_icon_button(
            MaterialIcon::RestorePage,
            MaterialIconStyle::Light,
            self.ui_size,
        )
        .on_press_maybe(self.state.can_reload.then_some(Message::Reload));

        /*
        let button_save = bottom_tooltip_icon_btn(
            &mut self.button_save,
            MaterialIcon::Save,
            &top_size_info,
            "Save As..",
            Some(save_message),
        );*/
        let button_save = material_icon_button(
            MaterialIcon::Save,
            if self.state.need_save {
                MaterialIconStyle::Dark
            } else {
                MaterialIconStyle::Light
            },
            self.ui_size,
        )
        .on_press(Message::FileSaveRequested);

        let button_save_as = material_icon_button(
            MaterialIcon::DriveFileMove,
            if self.state.need_save {
                MaterialIconStyle::Dark
            } else {
                MaterialIconStyle::Light
            },
            self.ui_size,
        )
        .on_press(Message::SaveAsRequested);

        let button_undo: Button<'_, Self::Message, Self::Theme, Self::Renderer> =
            material_icon_button(MaterialIcon::Undo, MaterialIconStyle::Dark, self.ui_size)
                .on_press_maybe(self.state.can_undo.then_some(Message::Undo));

        let button_redo: Button<'_, Self::Message, Self::Theme, Self::Renderer> =
            material_icon_button(MaterialIcon::Redo, MaterialIconStyle::Dark, self.ui_size)
                .on_press_maybe(self.state.can_redo.then_some(Message::Redo));

        let button_2d = tooltip(
            fixed_text_button("2D", 1.0, self.ui_size)
                .on_press(Message::ToggleView(SplitMode::Flat)),
            "Switch to flatscene only view",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);
        let button_3d: Button<'_, _, _, _> = fixed_text_button("3D", 1.0, self.ui_size)
            .on_press(Message::ToggleView(SplitMode::Scene3D));
        let button_thick_helices: Button<'_, _, _, _> = if self.app_state.want_thick_helices() {
            material_icon_button(MaterialIcon::Dehaze, MaterialIconStyle::Light, self.ui_size)
                .on_press(Message::ThickHelices(false))
        } else {
            material_icon_button(MaterialIcon::Water, MaterialIconStyle::Light, self.ui_size)
                .on_press(Message::ThickHelices(true))
        };
        let button_split: Tooltip<'_, _, _, _> = tooltip(
            text_button("3D+2D", self.ui_size).on_press(Message::ToggleView(SplitMode::Both)),
            "Switch to both flat and 3d view",
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box);

        let button_oxdna: Button<'_, _, _, _> =
            material_icon_button(MaterialIcon::Upload, MaterialIconStyle::Light, self.ui_size)
                .on_press(Message::ExportRequested);
        let oxdna_tooltip = button_oxdna;

        let button_3d_import: Button<'_, _, _, _> = material_icon_button(
            MaterialIcon::Coronavirus,
            MaterialIconStyle::Light,
            self.ui_size,
        )
        .on_press(Message::Import3D);

        let split_icon = if self.state.splited_2d {
            MaterialIcon::BorderOuter
        } else {
            MaterialIcon::BorderHorizontal
        };

        let mut button_split_2d: Button<'_, _, _, _> =
            material_icon_button(split_icon, MaterialIconStyle::Light, self.ui_size);

        if self.state.can_split2d {
            button_split_2d = button_split_2d.on_press(Message::Split2d);
        }

        let mut button_toggle_2d: Button<'_, _, _, _> = text_button("Toggle 2D", self.ui_size);

        if self.state.can_toggle_2d {
            button_toggle_2d = button_toggle_2d.on_press(Message::Toggle2D);
        }

        let mut button_flip_split: Button<'_, _, _, _> = material_icon_button(
            MaterialIcon::SwapVert,
            MaterialIconStyle::Light,
            self.ui_size,
        );
        if self.state.splited_2d {
            button_flip_split = button_flip_split.on_press(Message::FlipSplitViews);
        }

        let button_help: Button<'_, _, _, _> =
            text_button("Help", self.ui_size).on_press(Message::ForceHelp);

        let button_tutorial: Button<'_, _, _, _> =
            text_button("Tutorials", self.ui_size).on_press(Message::ShowTutorial);

        let action_mode_buttons: Vec<Element<'_, _, _, _>> = action_modes_to_display
            .iter()
            .map(|mode| {
                action_mode_btn(
                    mode,
                    self.app_state.get_action_mode(),
                    self.ui_size.button(),
                    self.app_state.get_widget_basis().is_axis_aligned(),
                    self.ui_size,
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

        let selection_mode_buttons: Vec<Element<'_, _, _, _>> = SelectionMode::ALL
            .iter()
            .filter(|mode| selection_modes_to_display.contains(mode))
            .map(|mode| {
                selection_mode_btn(mode, self.app_state.get_selection_mode(), self.ui_size).into()
            })
            .collect();

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
            //row![button_undo, button_redo,].spacing(self.ui_size.button_spacing()),
            // “Action” group
            Row::from_vec(action_mode_buttons).spacing(self.ui_size.button_spacing()),
            // “Selection” group
            Row::from_vec(selection_mode_buttons).spacing(self.ui_size.button_spacing()),
            row![button_help, button_tutorial,].spacing(self.ui_size.button_spacing()),
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
            .style(crate::theme::GuiBackground)
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
fn action_mode_btn<'a, State>(
    mode: &ActionMode,
    current_action_mode: ActionMode,
    button_size: impl Into<Length>,
    axis_aligned: bool,
    ui_size: UiSize,
) -> Button<'a, Message<State>, crate::Theme, crate::Renderer>
where
    State: AppState,
    //Theme: button::StyleSheet,
    //Renderer: iced::advanced::Renderer + iced::advanced::image::Renderer,
    //<Renderer as iced::advanced::image::Renderer>::Handle: From<image::Handle>,
{
    let icon_path = if current_action_mode == *mode {
        mode.icon_on(axis_aligned)
    } else {
        mode.icon_off(axis_aligned)
    };

    image_button(image(icon_path), ui_size).on_press(Message::ActionModeChanged(mode.clone()))
    //.style(ButtonStyle(fixed_mode == mode))
    // TODO: Reimplement fixed_mode
    // TODO: Use SelectionMode Copy trait.
}

fn selection_mode_btn<'a, S: AppState>(
    mode: &SelectionMode,
    current_mode: SelectionMode,
    ui_size: UiSize,
) -> Button<'a, Message<S>, crate::Theme, crate::Renderer> {
    let icon_path = if current_mode == *mode {
        mode.icon_on()
    } else {
        mode.icon_off()
    };

    image_button(image(icon_path), ui_size).on_press(Message::SelectionModeChanged(mode.clone()))
    //.style(ButtonStyle(fixed_mode == mode))
    // TODO: Reimplement fixed_mode
    // TODO: Use SelectionMode Copy trait.
}
