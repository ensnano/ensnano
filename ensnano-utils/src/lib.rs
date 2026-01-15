//! This modules defines types and operations used by the graphical component of ENSnano to
//! interact with the design.

pub mod app_state_parameters;
pub mod application;
pub mod bindgroup_manager;
pub mod buffer_dimensions;
pub mod colors;
pub mod consts;
pub mod filename;
pub mod graphics;
pub mod instance;
pub mod multiplexer_ext;
pub mod obj_loader;
pub mod operation;
pub mod strand_builder;
pub mod surfaces;
pub mod text;
pub mod texture;
pub mod torsion;
pub mod ui_size;

use ensnano_design::{grid::GridId, nucl::Nucl};
use wgpu::util::{BufferInitDescriptor, DeviceExt as _};

use crate::graphics::PhySize;

pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ObjectType {
    /// A nucleotide identified by its identifier
    Nucleotide(u32),
    /// A bond, identified by the identifier of the two nucleotides that it binds.
    Bond(u32, u32),
    /// A bond, identified by the identifier of the four nucleotides prev_nucl, nucl1, nucl2, next_nucl. If prev == nucl1 or newt == nucl2, it needs a lid
    SlicedBond(u32, u32, u32, u32),
    /// A Helix cylinder, identified by the identifier of the two nucleotides at its ends.
    HelixCylinder(u32, u32),
    /// A Helix cylinder, identified by the identifier of the two nucleotides at its ends, together with the list of the colors of the slices.
    ColoredHelixCylinder(u32, u32, Vec<u32>),
}

impl ObjectType {
    pub fn is_bond(&self) -> bool {
        matches!(self, Self::Bond(_, _))
    }

    pub fn is_helix_cylinder(&self) -> bool {
        matches!(self, Self::HelixCylinder(_, _))
    }

    pub fn same_type(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// The referential in which one wants to get an element's coordinates
#[derive(Debug, Clone, Copy)]
pub enum Referential {
    World,
    Model,
}

#[derive(Clone, Debug)]
pub struct RollRequest {
    pub target_helices: Option<Vec<usize>>,
}

#[derive(Clone, Copy, Debug)]
pub enum RapierSimulationRequest {
    Start,
}

#[derive(Clone, Debug)]
pub struct RigidBodyConstants {
    pub k_spring: f32,
    pub k_friction: f32,
    pub mass: f32,
    pub volume_exclusion: bool,
    pub brownian_motion: bool,
    pub brownian_rate: f32,
    pub brownian_amplitude: f32,
}

impl Default for RigidBodyConstants {
    fn default() -> Self {
        Self {
            k_friction: 1.,
            k_spring: 1.,
            mass: 1.,
            volume_exclusion: false,
            brownian_amplitude: 1.,
            brownian_rate: 1.,
            brownian_motion: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScaffoldInfo {
    pub id: usize,
    pub length: usize,
    pub starting_nucl: Option<Nucl>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SimulationState {
    #[default]
    None,
    Rolling,
    RigidGrid,
    RigidHelices,
    Paused,
    Twisting {
        grid_id: GridId,
    },
    Relaxing,
}

impl SimulationState {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_rolling(&self) -> bool {
        matches!(self, Self::Rolling)
    }

    pub fn simulating_grid(&self) -> bool {
        matches!(self, Self::RigidGrid)
    }

    pub fn simulating_helices(&self) -> bool {
        matches!(self, Self::RigidHelices)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self, Self::Paused)
    }

    pub fn is_running(&self) -> bool {
        !matches!(self, Self::Paused | Self::None)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum WidgetBasis {
    #[default]
    World,
    Object,
}

impl WidgetBasis {
    pub fn toggle(&mut self) {
        *self = if self.is_axis_aligned() {
            Self::Object
        } else {
            Self::World
        };
    }

    pub fn is_axis_aligned(&self) -> bool {
        matches!(self, Self::World)
    }
}

/// Information about the domain being elongated
#[derive(Debug, Clone)]
pub struct StrandBuildingStatus {
    pub nt_length: usize,
    pub nm_length: f32,
    pub prime3: Nucl,
    pub prime5: Nucl,
    pub dragged_nucl: Nucl,
}

impl StrandBuildingStatus {
    pub fn to_info(&self) -> String {
        format!(
            "Current domain length: {} nt ({:.2} nm). 5': {}, 3': {}",
            self.nt_length, self.nm_length, self.prime5.position, self.prime3.position
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PastingStatus {
    Copy,
    Duplication,
    None,
}

impl PastingStatus {
    pub fn is_pasting(&self) -> bool {
        match self {
            Self::Copy | Self::Duplication => true,
            Self::None => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// One of the standard scaffold sequence shipped with ENSnano
#[derive(Default)]
pub enum StandardSequence {
    P4844,
    #[default]
    P7249,
    P7560,
    P8064,
}

impl StandardSequence {
    pub fn description(&self) -> &'static str {
        match self {
            Self::P4844 => "m13 p4844",
            Self::P7249 => "m13 p7249",
            Self::P7560 => "m13 p7560",
            Self::P8064 => "m13 p8064",
        }
    }

    pub fn sequence(&self) -> &'static str {
        match self {
            Self::P4844 => include_str!("../p4844-Tilibit.txt"),
            Self::P7249 => include_str!("../p7249-Tilibit.txt"),
            Self::P7560 => include_str!("../p7560.txt"),
            Self::P8064 => include_str!("../m13-p8064.txt"),
        }
    }

    /// Return the variant of Self whose associated sequence length is closest to n
    pub fn from_length(n: usize) -> Self {
        let mut best_score = isize::MAX;
        let mut ret = Self::default();
        for candidate in [Self::P4844, Self::P7249, Self::P7560, Self::P8064] {
            let score = (candidate.sequence().len() as isize - (n as isize)).abs();
            if score < best_score {
                best_score = score;
                ret = candidate;
            }
        }
        ret
    }
}

pub fn create_buffer_with_data(
    device: &wgpu::Device,
    data: &[u8],
    usage: wgpu::BufferUsages,
    label: &str,
) -> wgpu::Buffer {
    let descriptor = BufferInitDescriptor {
        label: Some(label),
        contents: data,
        usage,
    };
    device.create_buffer_init(&descriptor)
}

pub fn apply_update<T: Clone, F>(obj: &mut T, update_func: F)
where
    F: FnOnce(T) -> T,
{
    let tmp = obj.clone();
    *obj = update_func(tmp);
}

pub fn convert_size_f32(size: PhySize) -> iced::Size<f32> {
    iced::Size::new(size.width as f32, size.height as f32)
}

pub fn convert_size_u32(size: PhySize) -> iced::Size<u32> {
    iced::Size::new(size.width, size.height)
}
