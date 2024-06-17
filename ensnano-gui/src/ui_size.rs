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
use serde_derive::{Deserialize, Serialize};

pub const ALL_UI_SIZES: [UiSize; 3] = [UiSize::Small, UiSize::Medium, UiSize::Large];

/// Size handler of the GUI
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum UiSize {
    Small,
    Medium,
    Large,
}

impl Default for UiSize {
    fn default() -> Self {
        Self::Medium
    }
}

impl UiSize {
    // Text related messages

    pub fn smaller_text(&self) -> u16 {
        match self {
            Self::Small => 10,
            Self::Medium => 12,
            Self::Large => 16,
        }
    }

    pub fn main_text(&self) -> f32 {
        match self {
            Self::Small => 12.0,
            Self::Medium => 16.0,
            Self::Large => 20.0,
        }
    }

    pub fn head_text(&self) -> u16 {
        match self {
            Self::Small => 18,
            Self::Medium => 24,
            Self::Large => 30,
        }
    }

    pub fn intermediate_text(&self) -> u16 {
        match self {
            Self::Small => 15,
            Self::Medium => 20,
            Self::Large => 25,
        }
    }

    /// Size of an icon
    pub fn icon(&self) -> f32 {
        match self {
            Self::Small => 14.0,
            Self::Medium => 20.0,
            Self::Large => 30.0,
        }
    }

    pub fn checkbox(&self) -> u16 {
        match self {
            Self::Small => 15,
            Self::Medium => 15,
            Self::Large => 15,
        }
    }

    pub fn checkbox_spacing(&self) -> u16 {
        5
    }

    /// Height of a button.
    pub fn button(&self) -> f32 {
        self.icon() + 6.0
    }

    /// Padding around button content.
    pub fn button_pad(&self) -> f32 {
        5.0 // This is the iced default.
    }

    /// Minimum space around buttons.
    pub fn button_spacing(&self) -> f32 {
        5.0
    }

    /// Larger space between button groups.
    pub fn button_group_spacing(&self) -> f32 {
        20.0
    }

    /// The full height of the top_bar
    pub fn top_bar(&self) -> f64 {
        (self.button() + 2.0 * self.button_pad() + 2.0 * self.button_spacing()) as f64
        // NOTE: We need this additional 10.0, but I don't understand why.
    }
}

impl std::fmt::Display for UiSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = match self {
            UiSize::Small => "Small",
            UiSize::Medium => "Medium",
            UiSize::Large => "Large",
        };
        write!(f, "{}", ret)
    }
}
