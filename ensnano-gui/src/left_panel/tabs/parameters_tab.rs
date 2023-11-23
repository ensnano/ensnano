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

use super::{AppState, FactoryId, Message, RequestFactory, ScrollSensitivity, UiSize, ValueId};
use crate::helpers::*;
use iced::Element;
use iced_native::widget::helpers::*;

pub struct ParametersTab {
    scroll_sensitivity_factory: RequestFactory<ScrollSensitivity>,
    pub invert_y_scroll: bool,
}

impl ParametersTab {
    pub fn new<S: AppState>(app_state: &S) -> Self {
        Self {
            scroll_sensitivity_factory: RequestFactory::new(
                FactoryId::Scroll,
                ScrollSensitivity {
                    initial_value: app_state.get_scroll_sensitivity(),
                },
            ),
            invert_y_scroll: false,
        }
    }

    pub fn view<S>(&mut self, ui_size: UiSize, app_state: &S) -> Element<Message<S>>
    where
        S: AppState,
    {
        let dna_params = &app_state.get_dna_parameters();

        let content = iced_native::column![
            section("Parameters", ui_size),
            extra_jump(),
            subsection("Font size", ui_size),
            pick_list(
                &super::super::super::ALL_UI_SIZES[..],
                Some(ui_size.clone()),
                Message::UiSizePicked,
            ),
            extra_jump(),
            subsection("Scrolling", ui_size),
            iced_native::widget::Column::with_children(
                self.scroll_sensitivity_factory
                    .view(true, ui_size.main_text())
            ),
            right_checkbox(
                app_state.get_invert_y_scroll(),
                "Inverse direction",
                Message::InvertScroll,
                ui_size.clone(),
            ),
            jump_by(10),
            section("P-stick model", ui_size),
            pick_list(
                &ensnano_design::NAMED_DNA_PARAMETERS[..],
                Some(app_state.get_dna_parameters().name().clone()),
                Message::NewDnaParameters,
            ),
            iced_native::column![
                text(format!("  Radius: {:.3} nm", dna_params.helix_radius)),
                text(format!("  Radius: {:.3} nm", dna_params.helix_radius)),
                text(format!("  Rise: {:.3} nm", dna_params.z_step)),
                text(format!("  Inclination {:.3} nm", dna_params.inclination)),
                text(format!("  Helicity: {:.2} bp", dna_params.bases_per_turn)),
                text(format!(
                    "  Axis: {:.1}°",
                    dna_params.groove_angle.to_degrees()
                )),
                text(format!(
                    "  Inter helix gap: {:.2} nm",
                    dna_params.inter_helix_gap
                )),
                text(format!(
                    " Expected xover length: {:.2} nm",
                    dna_params.dist_ac()
                )),
            ],
            jump_by(10),
            section("About", ui_size),
            text(format!("Version {}", ensnano_design::ensnano_version())),
            subsection("Development:", ui_size),
            text("Nicolas Levy"),
            extra_jump(),
            subsection("Conception:", ui_size),
            text("Nicolas Levy"),
            text("Nicolas Schabanel"),
            extra_jump(),
            subsection("License:", ui_size),
            text("GPLv3"),
        ];
        scrollable(content).into()
    }

    pub fn update_scroll_request(
        &mut self,
        value_id: ValueId,
        value: f32,
        request: &mut Option<f32>,
    ) {
        self.scroll_sensitivity_factory
            .update_request(value_id, value, request);
    }
}
