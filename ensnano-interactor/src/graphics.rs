use serde::{Deserialize, Serialize};
use ultraviolet::Vec3;
use winit::dpi::{PhysicalPosition, PhysicalSize};

#[derive(Clone, Debug, PartialEq, Eq, Copy, Serialize, Deserialize, Default)]
pub enum RenderingMode {
    #[default]
    Normal,
    Cartoon,
    Outline,
    BlackAndWhite,
}
impl RenderingMode {
    pub fn requires_post_processing(&self) -> bool {
        matches!(self, Self::Outline | Self::BlackAndWhite)
    }
}

pub const ALL_RENDERING_MODE: &[RenderingMode] = &[
    RenderingMode::Normal,
    RenderingMode::Cartoon,
    RenderingMode::Outline,
    RenderingMode::BlackAndWhite,
];

impl std::fmt::Display for RenderingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ret = match self {
            Self::Normal => "Normal",
            Self::Cartoon => "Cartoon",
            Self::Outline => "Outline",
            Self::BlackAndWhite => "Black and white",
        };
        write!(f, "{ret}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy, Serialize, Deserialize, Default)]
pub enum Background3D {
    #[default]
    Sky,
    White,
}

pub const ALL_BACKGROUND3D: &[Background3D] = &[Background3D::Sky, Background3D::White];

impl std::fmt::Display for Background3D {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ret = match self {
            Self::White => "White",
            Self::Sky => "Sky",
        };
        write!(f, "{ret}")
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum HBondDisplay {
    #[default]
    No,
    Stick,
    Ellipsoid,
}

impl std::fmt::Display for HBondDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ret = match self {
            Self::No => "No",
            Self::Stick => "Sticks",
            Self::Ellipsoid => "Ellipsoid",
        };
        write!(f, "{ret}")
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

/// The different elements represented on the scene. Each element is instantiated once.
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
        matches!(self, Self::TopBar | Self::LeftPanel | Self::StatusBar)
    }

    /// A scene represent a view to the DNA.
    pub fn is_scene(&self) -> bool {
        matches!(
            self,
            Self::StereographicScene | Self::Scene | Self::FlatScene
        )
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
