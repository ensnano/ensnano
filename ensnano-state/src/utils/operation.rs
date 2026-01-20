use crate::design::operation::{
    BezierPlaneHomothethy, DesignOperation, DesignRotation, DesignTranslation, IsometryTarget,
};
use ensnano_design::{
    bezier_plane::{BezierPlaneId, BezierVertexId},
    curves::bezier::BezierControlPoint,
    grid::{GridId, HelixGridPosition},
    nucl::Nucl,
    organizer_tree::GroupId,
};
use std::sync::Arc;
use ultraviolet::{Bivec3, Rotor3, Vec2, Vec3};

pub trait Operation: std::fmt::Debug + Sync + Send {
    /// The effect of self that must be sent as a notifications to the targeted designs
    fn effect(&self) -> DesignOperation;

    /// A description of self of display in the GUI
    fn description(&self) -> String;

    /// Produce an new operation by setting the value of the `n`-th parameter to `val`.
    fn with_new_value(&self, _n: usize, _val: String) -> Option<Arc<dyn Operation>> {
        None
    }

    /// The set of parameters that can be modified via a GUI component
    fn parameters(&self) -> &[&'static str] {
        &[]
    }

    /// The values associated to the parameters.
    fn values(&self) -> Vec<String> {
        vec![]
    }

    /// If true, this new operation is applied to the last initial state instead
    fn replace_previous(&self) -> bool {
        false
    }
}

pub struct CurrentOpState {
    pub current_operation: Arc<dyn Operation>,
    pub operation_id: usize,
}

#[derive(Clone, Debug)]
pub struct GridRotation {
    pub origin: Vec3,
    pub design_id: usize,
    pub grid_ids: Vec<GridId>,
    pub angle: f32,
    pub plane: Bivec3,
    pub group_id: Option<GroupId>,
    pub replace: bool,
}

impl Operation for GridRotation {
    fn parameters(&self) -> &[&'static str] {
        &["angle"]
    }

    fn values(&self) -> Vec<String> {
        vec![self.angle.to_degrees().to_string()]
    }

    fn effect(&self) -> DesignOperation {
        let rotor = Rotor3::from_angle_plane(self.angle, self.plane);
        DesignOperation::Rotation(DesignRotation {
            rotation: rotor,
            origin: self.origin,
            target: IsometryTarget::Grids(self.grid_ids.clone()),
            group_id: self.group_id,
        })
    }

    fn description(&self) -> String {
        format!(
            "Rotate grids {:?} of design {}",
            self.grid_ids, self.design_id
        )
    }

    fn with_new_value(&self, n: usize, val: String) -> Option<Arc<dyn Operation>> {
        if n == 0 {
            let degrees: f32 = val.parse().ok()?;
            Some(Arc::new(Self {
                angle: degrees.to_radians(),
                replace: true,
                ..self.clone()
            }))
        } else {
            None
        }
    }

    fn replace_previous(&self) -> bool {
        self.replace
    }
}

#[derive(Clone, Debug)]
pub struct HelixRotation {
    pub origin: Vec3,
    pub design_id: usize,
    pub helices: Vec<usize>,
    pub angle: f32,
    pub plane: Bivec3,
    pub group_id: Option<GroupId>,
    pub replace: bool,
}

impl Operation for HelixRotation {
    fn parameters(&self) -> &[&'static str] {
        &["angle"]
    }

    fn values(&self) -> Vec<String> {
        vec![self.angle.to_degrees().to_string()]
    }

    fn effect(&self) -> DesignOperation {
        let rotor = Rotor3::from_angle_plane(self.angle, self.plane);
        DesignOperation::Rotation(DesignRotation {
            rotation: rotor,
            origin: self.origin,
            target: IsometryTarget::Helices(self.helices.clone(), false),
            group_id: self.group_id,
        })
    }

    fn description(&self) -> String {
        format!(
            "Rotate helices {:?} of design {}",
            self.helices, self.design_id
        )
    }

    fn with_new_value(&self, n: usize, val: String) -> Option<Arc<dyn Operation>> {
        if n == 0 {
            let degrees: f32 = val.parse().ok()?;
            Some(Arc::new(Self {
                angle: degrees.to_radians(),
                replace: true,
                ..self.clone()
            }))
        } else {
            None
        }
    }

    fn replace_previous(&self) -> bool {
        self.replace
    }
}

#[derive(Debug, Clone)]
pub struct BezierControlPointTranslation {
    pub control_points: Vec<(usize, BezierControlPoint)>,
    pub right: Vec3,
    pub top: Vec3,
    pub dir: Vec3,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub group_id: Option<GroupId>,
}

impl Operation for BezierControlPointTranslation {
    fn parameters(&self) -> &[&'static str] {
        &["x", "y", "z"]
    }

    fn values(&self) -> Vec<String> {
        vec![self.x.to_string(), self.y.to_string(), self.z.to_string()]
    }

    fn effect(&self) -> DesignOperation {
        let translation = self.x * self.right + self.y * self.top + self.z * self.dir;
        DesignOperation::Translation(DesignTranslation {
            translation,
            target: IsometryTarget::ControlPoint(self.control_points.clone()),
            group_id: self.group_id,
        })
    }

    fn description(&self) -> String {
        format!("Translate control points {:?}", self.control_points,)
    }

    fn with_new_value(&self, n: usize, val: String) -> Option<Arc<dyn Operation>> {
        match n {
            0 => {
                let new_x: f32 = val.parse().ok()?;
                Some(Arc::new(Self {
                    x: new_x,
                    ..self.clone()
                }))
            }
            1 => {
                let new_y: f32 = val.parse().ok()?;
                Some(Arc::new(Self {
                    y: new_y,
                    ..self.clone()
                }))
            }
            2 => {
                let new_z: f32 = val.parse().ok()?;
                Some(Arc::new(Self {
                    z: new_z,
                    ..self.clone()
                }))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TranslateBezierPathVertex {
    pub vertices: Vec<BezierVertexId>,
    pub x: f32,
    pub y: f32,
}

impl Operation for TranslateBezierPathVertex {
    fn description(&self) -> String {
        String::from("Positioning BezierPath Vertex")
    }

    fn effect(&self) -> DesignOperation {
        DesignOperation::MoveBezierVertex {
            vertices: self.vertices.clone(),
            position: Vec2::new(self.x, self.y),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TranslateBezierSheetCorner {
    pub plane_id: BezierPlaneId,
    pub fixed_corner: Vec2,
    pub origin_moving_corner: Vec2,
    pub moving_corner: Vec2,
}

impl Operation for TranslateBezierSheetCorner {
    fn description(&self) -> String {
        String::from("Translating BezierSheet Corner")
    }

    fn effect(&self) -> DesignOperation {
        DesignOperation::ApplyHomothethyOnBezierPlane {
            homothethy: BezierPlaneHomothethy {
                plane_id: self.plane_id,
                fixed_corner: self.fixed_corner,
                origin_moving_corner: self.origin_moving_corner,
                moving_corner: self.moving_corner,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct HelixTranslation {
    pub design_id: usize,
    pub helices: Vec<usize>,
    pub right: Vec3,
    pub top: Vec3,
    pub dir: Vec3,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub snap: bool,
    pub group_id: Option<GroupId>,
    pub replace: bool,
}

impl Operation for HelixTranslation {
    fn parameters(&self) -> &[&'static str] {
        &["x", "y", "z"]
    }

    fn values(&self) -> Vec<String> {
        vec![self.x.to_string(), self.y.to_string(), self.z.to_string()]
    }

    fn effect(&self) -> DesignOperation {
        let translation = self.x * self.right + self.y * self.top + self.z * self.dir;
        DesignOperation::Translation(DesignTranslation {
            translation,
            target: IsometryTarget::Helices(self.helices.clone(), self.snap),
            group_id: self.group_id,
        })
    }

    fn description(&self) -> String {
        format!(
            "Translate helices {:?} of design {}",
            self.helices, self.design_id
        )
    }

    fn with_new_value(&self, n: usize, val: String) -> Option<Arc<dyn Operation>> {
        match n {
            0 => {
                let new_x: f32 = val.parse().ok()?;
                Some(Arc::new(Self {
                    x: new_x,
                    replace: true,
                    ..self.clone()
                }))
            }
            1 => {
                let new_y: f32 = val.parse().ok()?;
                Some(Arc::new(Self {
                    y: new_y,
                    replace: true,
                    ..self.clone()
                }))
            }
            2 => {
                let new_z: f32 = val.parse().ok()?;
                Some(Arc::new(Self {
                    z: new_z,
                    replace: true,
                    ..self.clone()
                }))
            }
            _ => None,
        }
    }

    fn replace_previous(&self) -> bool {
        self.replace
    }
}

#[derive(Debug, Clone)]
pub struct GridTranslation {
    pub design_id: usize,
    pub grid_ids: Vec<GridId>,
    pub right: Vec3,
    pub top: Vec3,
    pub dir: Vec3,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub group_id: Option<GroupId>,
    pub replace: bool,
}

impl Operation for GridTranslation {
    fn parameters(&self) -> &[&'static str] {
        &["x", "y", "z"]
    }

    fn values(&self) -> Vec<String> {
        vec![self.x.to_string(), self.y.to_string(), self.z.to_string()]
    }

    fn effect(&self) -> DesignOperation {
        let translation = self.x * self.right + self.y * self.top + self.z * self.dir;
        DesignOperation::Translation(DesignTranslation {
            translation,
            target: IsometryTarget::Grids(self.grid_ids.clone()),
            group_id: self.group_id,
        })
    }

    fn description(&self) -> String {
        format!(
            "Translate grids {:?} of design {}",
            self.grid_ids, self.design_id
        )
    }

    fn with_new_value(&self, n: usize, val: String) -> Option<Arc<dyn Operation>> {
        match n {
            0 => {
                let new_x: f32 = val.parse().ok()?;
                Some(Arc::new(Self {
                    x: new_x,
                    replace: true,
                    ..self.clone()
                }))
            }
            1 => {
                let new_y: f32 = val.parse().ok()?;
                Some(Arc::new(Self {
                    y: new_y,
                    replace: true,
                    ..self.clone()
                }))
            }
            2 => {
                let new_z: f32 = val.parse().ok()?;
                Some(Arc::new(Self {
                    z: new_z,
                    replace: true,
                    ..self.clone()
                }))
            }
            _ => None,
        }
    }

    fn replace_previous(&self) -> bool {
        self.replace
    }
}

#[derive(Debug, Clone)]
pub struct GridHelixCreation {
    pub design_id: usize,
    pub grid_id: GridId,
    pub x: isize,
    pub y: isize,
    pub position: isize,
    pub length: usize,
}

impl Operation for GridHelixCreation {
    fn values(&self) -> Vec<String> {
        vec![self.x.to_string(), self.y.to_string()]
    }

    fn effect(&self) -> DesignOperation {
        DesignOperation::AddGridHelix {
            position: HelixGridPosition {
                grid: self.grid_id,
                x: self.x,
                y: self.y,
                roll: 0f32,
                axis_pos: 0,
            },
            start: self.position,
            length: self.length,
        }
    }

    fn description(&self) -> String {
        format!(
            "Create helix on grid {:?} of design {}",
            self.grid_id, self.design_id
        )
    }

    fn with_new_value(&self, n: usize, val: String) -> Option<Arc<dyn Operation>> {
        match n {
            0 => {
                let new_x: f32 = val.parse().ok()?;
                Some(Arc::new(Self {
                    x: new_x as isize,
                    ..*self
                }))
            }
            1 => {
                let new_y: f32 = val.parse().ok()?;
                Some(Arc::new(Self {
                    y: new_y as isize,
                    ..*self
                }))
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
/// Cut a strand at a given nucleotide.
///
/// If the nucleotide is the 3' end of a cross-over, it will be the 5' end of the 3' half of the
/// split.
/// In all other cases, it will be the 3' end of the 5' end of the split.
pub struct Cut {
    pub nucl: Nucl,
}

impl Operation for Cut {
    fn effect(&self) -> DesignOperation {
        DesignOperation::Cut { nucl: self.nucl }
    }

    fn description(&self) -> String {
        format!("Cut on nucleotide {}", self.nucl)
    }

    fn with_new_value(&self, _n: usize, _val: String) -> Option<Arc<dyn Operation>> {
        None
    }
}

#[derive(Clone, Debug)]
pub struct Xover {
    pub prime5_id: usize,
    pub prime3_id: usize,
    pub undo: bool,
}

impl Operation for Xover {
    fn effect(&self) -> DesignOperation {
        DesignOperation::Xover {
            prime5_id: self.prime5_id,
            prime3_id: self.prime3_id,
        }
    }

    fn description(&self) -> String {
        if self.undo {
            "Undo Cut".to_owned()
        } else {
            "Do Cut".to_owned()
        }
    }
}

/// Cut the target strand at nucl, and make a cross over from the source strand.
#[derive(Clone, Debug)]
pub struct CrossCut {
    pub source_id: usize,
    pub target_id: usize,
    pub nucl: Nucl,
    /// True if the target strand will be the 3 prime part of the merged strand
    pub target_3prime: bool,
}

impl Operation for CrossCut {
    fn effect(&self) -> DesignOperation {
        DesignOperation::CrossCut {
            source_id: self.source_id,
            target_id: self.target_id,
            target_3prime: self.target_3prime,
            nucl: self.nucl,
        }
    }

    fn description(&self) -> String {
        format!(
            "Cross cut from strand {} on nucl {} (strand {})",
            self.source_id, self.nucl, self.target_id
        )
    }
}
