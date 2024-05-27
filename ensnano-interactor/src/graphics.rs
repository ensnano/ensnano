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

use iced_winit::winit;
use serde::{Deserialize, Serialize};
use ultraviolet::Vec3;
use winit::dpi::{PhysicalPosition, PhysicalSize};
#[derive(Clone, Debug, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum RenderingMode {
    Normal,
    Cartoon,
}

pub const ALL_RENDERING_MODE: [RenderingMode; 2] = [RenderingMode::Normal, RenderingMode::Cartoon];

impl Default for RenderingMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum Background3D {
    Sky,
    White,
}

pub const ALL_BACKGROUND3D: [Background3D; 2] = [Background3D::Sky, Background3D::White];

impl Default for Background3D {
    fn default() -> Self {
        Self::Sky
    }
}

impl std::fmt::Display for Background3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = match self {
            Self::White => "White",
            Self::Sky => "Sky",
        };
        write!(f, "{}", ret)
    }
}

impl std::fmt::Display for RenderingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = match self {
            Self::Normal => "Normal",
            Self::Cartoon => "Cartoon",
        };
        write!(f, "{}", ret)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HBondDisplay {
    No,
    Stick,
    Ellipsoid,
}

impl Default for HBondDisplay {
    fn default() -> Self {
        Self::No
    }
}

impl std::fmt::Display for HBondDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ret = match self {
            Self::No => "No",
            Self::Stick => "Sticks",
            Self::Ellipsoid => "Ellipsoid",
        };
        write!(f, "{}", ret)
    }
}

pub mod fog_kind {
    pub const NO_FOG: u32 = 0;
    pub const TRANSPARENT_FOG: u32 = 1;
    pub const DARK_FOG: u32 = 2;
    pub const REVERSED_FOG: u32 = 3;
}

/// Parameters for the Distance Fog effect in the 3D scene.
#[derive(Debug, Clone)]
pub struct FogParameters {
    // Softness of the Distance Fog cutoff.
    pub softness: f32,
    // Deepness of the Distance Fog.
    pub length: f32,
    pub fog_kind: u32,
    // Compute Distance Fog from the camera or pivot position.
    pub from_camera: bool,
    pub alt_fog_center: Option<Vec3>,
}

impl FogParameters {
    pub fn new() -> Self {
        Self {
            softness: 10.,
            length: 10.,
            fog_kind: fog_kind::NO_FOG,
            from_camera: true,
            alt_fog_center: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplitMode {
    Flat,
    Scene3D,
    Both,
}

pub type PhySize = PhysicalSize<u32>;

/// A structure that represents an area on which an element can be drawn
#[derive(Clone, Copy, Debug)]
pub struct DrawArea {
    /// The top left corner of the element
    pub position: PhysicalPosition<u32>,
    /// The *physical* size of the element
    pub size: PhySize,
}

/// The different elements represented on the scene. Each element is instanciated once.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum GuiComponentType {
    /// The top menu bar
    TopBar,
    /// The 3D scene
    Scene,
    /// The flat Scene
    FlatScene,
    /// The Left Panel
    LeftPanel,
    /// The status bar
    StatusBar,
    GridPanel,
    /// An overlay area
    Overlay(usize),
    /// An area that has not been attributed to an element
    Unattributed,
    /// A stereographic version of the 3D view
    StereographicScene,
}

/// GUI component types are grouped by category.
impl GuiComponentType {
    /// A panel is filled with buttons, menus, and other dialogs.
    pub fn is_panel(&self) -> bool {
        match self {
            GuiComponentType::TopBar
            | GuiComponentType::LeftPanel
            | GuiComponentType::StatusBar => true,
            _ => false,
        }
    }

    /// A scene represent a view to the DNA.
    pub fn is_scene(&self) -> bool {
        match self {
            GuiComponentType::StereographicScene
            | GuiComponentType::Scene
            | GuiComponentType::FlatScene => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LoopoutNucl {
    pub position: Vec3,
    pub color: u32,
    /// The identifier of the bond representing the whole loopout involving this nucleotide
    pub repr_bond_identifier: u32,
    pub basis: Option<char>,
}

#[derive(Clone, Debug)]
pub struct LoopoutBond {
    pub position_prime5: Vec3,
    pub position_prime3: Vec3,
    pub color: u32,
    /// The identifier of the bond representing the whole loopout involving this bond
    pub repr_bond_identifier: u32,
}
