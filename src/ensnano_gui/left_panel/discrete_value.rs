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
use super::AppState;
use super::Message;
use ensnano_iced::iced;
use ensnano_iced::{
    helpers::*,
    iced::{Alignment, Length, Pixels},
    theme,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValueId(pub usize);

pub trait Requestable {
    type Request;
    fn request_from_values(&self, values: &[f32]) -> Self::Request;
    fn nb_values(&self) -> usize;
    fn initial_value(&self, n: usize) -> f32;
    fn min_val(&self, n: usize) -> f32;
    fn max_val(&self, n: usize) -> f32;
    fn step_val(&self, n: usize) -> f32;
    fn name_val(&self, n: usize) -> String;

    fn make_request(&self, values: &[f32], request: &mut Option<Self::Request>) {
        *request = Some(self.request_from_values(values))
    }

    fn hidden(&self, _: usize) -> bool {
        false
    }
}

pub struct RequestFactory<R: Requestable> {
    values: BTreeMap<ValueId, DiscreteValue>,
    pub requestable: R,
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum FactoryId {
    HelixRoll,
    Hyperboloid,
    Scroll,
    RigidBody,
    Brownian,
}

impl<R: Requestable> RequestFactory<R> {
    pub fn new(factory_id: FactoryId, requestable: R) -> Self {
        let mut values = BTreeMap::new();
        for n in 0..requestable.nb_values() {
            let default = requestable.initial_value(n);
            let min_val = requestable.min_val(n);
            let max_val = requestable.max_val(n);
            let step_val = requestable.step_val(n);
            let name = requestable.name_val(n);
            values.insert(
                ValueId(n),
                DiscreteValue::new(
                    default,
                    step_val,
                    min_val,
                    max_val,
                    name,
                    factory_id,
                    ValueId(n),
                    requestable.hidden(n),
                ),
            );
        }
        Self {
            values,
            requestable,
        }
    }

    pub fn view<State: AppState>(
        &self,
        active: bool,
        size: impl Into<Pixels>,
    ) -> Vec<iced::Element<'_, Message<State>>> {
        let s = size.into();
        self.values
            .values()
            .filter(|v| !v.hidden)
            .map(|v| v.view(active, s))
            .collect()
    }

    pub fn update_request(
        &mut self,
        value_id: ValueId,
        new_val: f32,
        request: &mut Option<R::Request>,
    ) {
        self.values
            .get_mut(&value_id)
            .unwrap()
            .update_value(new_val);
        let values: Vec<f32> = self.values.values().map(|v| v.get_value()).collect();
        self.requestable.make_request(&values, request)
    }

    pub fn update_value(&mut self, value_id: ValueId, new_val: f32) -> R::Request {
        self.values
            .get_mut(&value_id)
            .unwrap()
            .update_value(new_val);
        let values: Vec<f32> = self.values.values().map(|v| v.get_value()).collect();
        self.requestable.request_from_values(&values)
    }

    pub fn make_request(&self, request: &mut Option<R::Request>) {
        let values: Vec<f32> = self.values.values().map(|v| v.get_value()).collect();
        self.requestable.make_request(&values, request)
    }
}

/// A DiscreteValue allow the user to chose a numerical value with prescibed constraints.
///
/// The value must be chosen in the discrete range between [min_val], [max_val] by increments of
/// [step].
struct DiscreteValue {
    // Current selected value.
    value: f32,
    step: f32,
    min_val: f32,
    max_val: f32,
    name: String,
    owner_id: FactoryId,
    value_id: ValueId,
    hidden: bool,
}

impl DiscreteValue {
    fn new(
        default: f32,
        step: f32,
        min_val: f32,
        max_val: f32,
        name: String,
        owner_id: FactoryId,
        value_id: ValueId,
        hidden: bool,
    ) -> Self {
        Self {
            value: default,
            step,
            min_val,
            max_val,
            name,
            owner_id,
            value_id,
            hidden,
        }
    }

    fn view<State: AppState>(
        &self,
        active: bool,
        name_size: impl Into<Pixels>,
    ) -> iced::Element<'_, Message<State>> {
        let decr_button = if active && self.value - self.step >= self.min_val {
            button(text("-")).on_press(Message::DiscreteValue {
                factory_id: self.owner_id,
                value_id: self.value_id,
                value: self.value - self.step,
            })
        } else {
            button(text("-"))
        };
        let incr_button = if active && self.value + self.step <= self.max_val {
            button(text("+")).on_press(Message::DiscreteValue {
                factory_id: self.owner_id,
                value_id: self.value_id,
                value: self.value + self.step,
            })
        } else {
            button(text("+"))
        };
        let factory_id = self.owner_id.clone();
        let value_id = self.value_id.clone();
        let slider = if active {
            slider(self.min_val..=self.max_val, self.value, move |value| {
                Message::DiscreteValue {
                    factory_id,
                    value_id,
                    value,
                }
            })
            .step(self.step)
        } else {
            slider(self.min_val..=self.max_val, self.value, |_| {
                Message::Nothing
            })
            .style(theme::DeactivatedSlider)
        };

        let mut name_text = text(self.name.clone()).size(name_size);

        if !active {
            name_text = name_text.style(theme::DISABLED_TEXT);
        }

        row![
            // On the left: print the name of the parameter being selected.
            row![name_text, Space::with_width(Length::Fill),]
                .align_items(Alignment::Center)
                .width(Length::FillPortion(8)),
            // On the middle: print the currently selected value.
            row![text(format!("{:.1}", self.value)),].width(Length::FillPortion(3)),
            // One the right: the buttons and slider that allow to modify the currently selected
            // value.
            row![decr_button, incr_button, Space::with_width(2), slider,]
                .width(Length::FillPortion(10)),
            //
            Space::with_width(Length::FillPortion(1)),
        ]
        .align_items(Alignment::Center)
        .into()
    }

    fn get_value(&self) -> f32 {
        self.value
    }

    fn update_value(&mut self, new_val: f32) {
        self.value = new_val
    }
}
