//! This modules defines operations that can be performed on a design to modify it.
//! The functions that apply these operations take a mutable reference to the design that they are
//! modifying and may return an `ErrOperation` if the operation could not be applied.

use crate::ensnano_design::{
    Design,
    bezier_plane::{BezierPathId, BezierVertexId},
    curves::CurveDescriptor,
    grid::{GridId, GridObject, GridPosition, HelicesTranslator, HelixGridPosition},
};
use std::sync::Arc;
use ultraviolet::{Rotor3, Vec3};

/// An error that occurred when trying to apply an operation.
#[derive(Debug)]
pub enum ErrDesignOperation {
    NotEnoughHelices { actual: usize, needed: usize },
    GridPositionAlreadyUsed,
    HelixDoesNotExists(usize),
    GridDoesNotExist(GridId),
    HelixCollisionDuringTranslation,
    NotEnoughBezierPoints,
    HelixIsNotPiecewiseBezier,
    CouldNotGetPath(BezierPathId),
    CouldNotGetVertex(BezierVertexId),
}

/// The minimum number of helices required to infer a grid
pub const MIN_HELICES_TO_MAKE_GRID: usize = 4;

/// Try to create a grid from a set of helices.
// TODO: rename this or super::grid::make_grid_from_helices to avoid collision
pub fn make_grid_from_helices(
    design: &mut Design,
    helices: &[usize],
) -> Result<(), ErrDesignOperation> {
    super::grid::make_grid_from_helices(design, helices)?;
    Ok(())
}

/// Attach an helix to a grid. The target grid position must be empty
pub fn attach_object_to_grid(
    design: &mut Design,
    object: GridObject,
    grid: GridId,
    x: isize,
    y: isize,
) -> Result<(), ErrDesignOperation> {
    let grid_manager = design.get_updated_grid_data();
    if matches!(grid_manager.pos_to_object(GridPosition{
        grid, x, y
    }), Some(obj) if obj != object)
    {
        Err(ErrDesignOperation::GridPositionAlreadyUsed)
    } else {
        let mut helices_mut = design.helices.make_mut();
        let helix_ref = helices_mut
            .get_mut(&object.helix())
            .ok_or_else(|| ErrDesignOperation::HelixDoesNotExists(object.helix()))?;
        // take previous axis position if there were one
        match object {
            GridObject::Helix(_) => {
                let axis_pos = helix_ref
                    .grid_position
                    .map(|pos| pos.axis_pos)
                    .unwrap_or_default();
                let roll = helix_ref
                    .grid_position
                    .map(|pos| pos.roll)
                    .unwrap_or_default();
                helix_ref.grid_position = Some(HelixGridPosition {
                    grid,
                    x,
                    y,
                    axis_pos,
                    roll,
                });
            }
            GridObject::BezierPoint { n, .. } => {
                let desc: Option<&mut CurveDescriptor> =
                    if let Some(desc) = helix_ref.curve.as_mut() {
                        Some(Arc::make_mut(desc))
                    } else {
                        None
                    };
                if let Some(CurveDescriptor::PiecewiseBezier { points, .. }) = desc {
                    if let Some(point) = points.get_mut(n) {
                        point.position = GridPosition { grid, x, y };
                    } else {
                        return Err(ErrDesignOperation::NotEnoughBezierPoints);
                    }
                }
            }
        }
        Ok(())
    }
}

/// Translate helices by a given translation.
///
/// If snap is true, the helices are mapped to grid position.
/// If this translation would cause helices to compete with other helices for a grid position,
/// an error is returned.
pub fn translate_helices(
    design: &mut Design,
    snap: bool,
    helices: Vec<usize>,
    translation: Vec3,
) -> Result<(), ErrDesignOperation> {
    let mut helices_translator = HelicesTranslator::from_design(design);
    helices_translator.translate_helices(snap, helices, translation)
}

/// Rotate helices by a given rotation
///
/// If snap is true, the helices are mapped to grid position.
/// If this rotation would cause helices to compete with other helices for a grid position,
/// an error is returned.
pub fn rotate_helices_3d(
    design: &mut Design,
    snap: bool,
    helices: Vec<usize>,
    rotation: Rotor3,
    origin: Vec3,
) -> Result<(), ErrDesignOperation> {
    let mut helices_translator = HelicesTranslator::from_design(design);
    helices_translator.rotate_helices_3d(snap, helices, rotation, origin)
}
