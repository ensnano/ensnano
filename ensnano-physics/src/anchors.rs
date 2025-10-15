use ensnano_design::{Helix, HelixParameters, Vec3};
use rapier3d::{
    na::{Const, OVector, Rotation2, Unit, UnitQuaternion},
    prelude::*,
};

/// An internal structure, which is made from an Helix
/// and computes the ideal spring anchor points to enforce
/// a rigid, straight DNA or RNA strand
pub(crate) struct SpringsAnchorReference {
    nucleotide_forward: OVector<f32, Const<3>>,
    nucleotide_backward: OVector<f32, Const<3>>,

    // from center of the pair to the center of
    // the next pair
    up: OVector<f32, Const<3>>,
    down: OVector<f32, Const<3>>,

    up_forward_anchor: OVector<f32, Const<3>>,
    up_backward_anchor: OVector<f32, Const<3>>,
    up_left_anchor: OVector<f32, Const<3>>,
    up_right_anchor: OVector<f32, Const<3>>,
    down_forward_anchor: OVector<f32, Const<3>>,
    down_backward_anchor: OVector<f32, Const<3>>,
    down_left_anchor: OVector<f32, Const<3>>,
    down_right_anchor: OVector<f32, Const<3>>,
}

/// Conversion method
pub(crate) fn vec_to_point(v: Vec3) -> OVector<f32, Const<3>> {
    vector![v.x, v.y, v.z].into()
}

fn turn_anchors(
    forward: OVector<f32, Const<3>>,
    backward: OVector<f32, Const<3>>,
    axis: OVector<f32, Const<3>>,
) -> (OVector<f32, Const<3>>, OVector<f32, Const<3>>) {
    let center = (forward + backward) / 2.0;
    let c_forward = forward - center;
    let c_backward = backward - center;

    let rotation =
        UnitQuaternion::from_axis_angle(&Unit::new_normalize(axis), std::f32::consts::FRAC_PI_2);

    (
        center + rotation * c_forward,
        center + rotation * c_backward,
    )
}

impl SpringsAnchorReference {
    pub(crate) fn new(helix: Helix, distance: u32, default_parameters: &HelixParameters) -> Self {
        let nucleotide_forward = vec_to_point(helix.space_pos(default_parameters, 0, true));
        let nucleotide_backward = vec_to_point(helix.space_pos(default_parameters, 0, false));

        let center = (nucleotide_forward + nucleotide_backward) / 2.0;

        let up_nucleotide_forward =
            vec_to_point(helix.space_pos(default_parameters, distance as isize, true));
        let up_nucleotide_backward =
            vec_to_point(helix.space_pos(default_parameters, distance as isize, false));

        let up_center = (up_nucleotide_forward + up_nucleotide_backward) / 2.0;

        let up = up_center - center;

        let down_nucleotide_forward =
            vec_to_point(helix.space_pos(default_parameters, -(distance as isize), true));
        let down_nucleotide_backward =
            vec_to_point(helix.space_pos(default_parameters, -(distance as isize), false));

        let down_center = (down_nucleotide_forward + down_nucleotide_backward) / 2.0;

        let down = down_center - center;

        let up_forward_anchor = (nucleotide_forward + up_nucleotide_forward) / 2.0;
        let up_backward_anchor = (nucleotide_backward + up_nucleotide_backward) / 2.0;
        let (up_left_anchor, up_right_anchor) =
            turn_anchors(up_forward_anchor, up_backward_anchor, up);

        let down_forward_anchor = (nucleotide_forward + down_nucleotide_forward) / 2.0;
        let down_backward_anchor = (nucleotide_backward + down_nucleotide_backward) / 2.0;
        let (down_left_anchor, down_right_anchor) =
            turn_anchors(down_forward_anchor, down_backward_anchor, down);

        Self {
            nucleotide_forward: nucleotide_forward - center,
            nucleotide_backward: nucleotide_backward - center,
            up,
            down,
            up_forward_anchor: up_forward_anchor - center,
            up_backward_anchor: up_backward_anchor - center,
            down_forward_anchor: down_forward_anchor - center,
            down_backward_anchor: down_backward_anchor - center,
            up_left_anchor,
            up_right_anchor,
            down_left_anchor,
            down_right_anchor,
        }
    }

    // assumes the center of the target is also 0
    fn first_rotation(&self, target_forward: OVector<f32, Const<3>>) -> UnitQuaternion<f32> {
        UnitQuaternion::rotation_between_axis(
            &Unit::new_normalize(self.nucleotide_forward),
            &Unit::new_normalize(target_forward),
        )
        .expect("Failed constructing first rotation")
    }

    // assumes both pairs are already aligned by the
    // first rotation
    fn second_rotation(
        current_up: OVector<f32, Const<3>>,
        target_forward: OVector<f32, Const<3>>,
        target_up: OVector<f32, Const<3>>,
    ) -> UnitQuaternion<f32> {
        // we define a plane orign at 0,
        // normal being the first nucleotide
        // (thanks to the first rotation,
        // current_forward and target_forward are
        // aligned)
        let plane_normal = target_forward.normalize();

        // we compute the projections of both up vectors onto that plane
        let dist_current_to_plane = current_up.dot(&plane_normal);
        let current_projected = current_up - plane_normal * dist_current_to_plane;

        let dist_target_to_plane = target_up.dot(&plane_normal);
        let target_projected = target_up - plane_normal * dist_target_to_plane;

        // we rotate those projections to be on the XY plane instead
        let plane_rotation = UnitQuaternion::rotation_between_axis(
            &Unit::new_unchecked(plane_normal),
            &Unit::new_unchecked(vector![0.0, 0.0, 1.0]),
        )
        .expect("Failed constructing second rotation plane rotation");

        let current_rotated = plane_rotation * current_projected;
        let current_rotated = OVector::<f32, Const<2>>::new(current_rotated.x, current_rotated.y);
        let target_rotated = plane_rotation * target_projected;
        let target_rotated = OVector::<f32, Const<2>>::new(target_rotated.x, target_rotated.y);

        let angle_between = Rotation2::rotation_between(&current_rotated, &target_rotated).angle();

        UnitQuaternion::from_axis_angle(&Unit::new_unchecked(plane_normal), angle_between)
    }

    /// Returns the spring anchors required for the base pair provided in parameters,
    /// and a vector pointing to the center of the upper targeted pair.
    /// This uses both uses the precomputed reference in self,
    /// and a sequence of two rotations to place it in the orientation
    /// of the desired nucleotide.
    pub(crate) fn get_up_spring_anchors(
        &self,
        forward_nucleotide: OVector<f32, Const<3>>,
        backward_nucleotide: OVector<f32, Const<3>>,
        up: OVector<f32, Const<3>>,
    ) -> (Point<f32>, Point<f32>, Point<f32>, Point<f32>) {
        let target_center = (forward_nucleotide + backward_nucleotide) / 2.0;

        let target_forward = forward_nucleotide - target_center;
        let target_backward = backward_nucleotide - target_center;
        let target_up = up;

        let first_rotation = self.first_rotation(target_forward);

        let current_up = first_rotation * self.up;
        let current_anchor_forward = first_rotation * self.up_forward_anchor;
        let current_anchor_backward = first_rotation * self.up_backward_anchor;
        let current_anchor_left = first_rotation * self.up_left_anchor;
        let current_anchor_right = first_rotation * self.up_right_anchor;

        let second_rotation = Self::second_rotation(current_up, target_forward, target_up);

        (
            (second_rotation * current_anchor_forward).into(),
            (second_rotation * current_anchor_backward).into(),
            (second_rotation * current_anchor_left).into(),
            (second_rotation * current_anchor_right).into(),
        )
    }

    /// Returns the spring anchors required for the base pair provided in parameters,
    /// and a vector pointing to the center of the upper targeted pair.
    /// This uses both uses the precomputed reference in self,
    /// and a sequence of two rotations to place it in the orientation
    /// of the desired nucleotide.
    pub(crate) fn get_down_spring_anchors(
        &self,
        forward_nucleotide: OVector<f32, Const<3>>,
        backward_nucleotide: OVector<f32, Const<3>>,
        down: OVector<f32, Const<3>>,
    ) -> (Point<f32>, Point<f32>, Point<f32>, Point<f32>) {
        let target_center = (forward_nucleotide + backward_nucleotide) / 2.0;

        let target_forward = forward_nucleotide - target_center;
        let target_backward = backward_nucleotide - target_center;
        let target_down = down;

        let first_rotation = self.first_rotation(target_forward);

        let current_down = first_rotation * self.down;
        let current_anchor_forward = first_rotation * self.up_forward_anchor;
        let current_anchor_backward = first_rotation * self.up_backward_anchor;
        let current_anchor_left = first_rotation * self.up_left_anchor;
        let current_anchor_right = first_rotation * self.up_right_anchor;

        let second_rotation = Self::second_rotation(current_down, target_forward, target_down);

        (
            (second_rotation * current_anchor_forward).into(),
            (second_rotation * current_anchor_backward).into(),
            // left / right order is inverted to match the upper order
            (second_rotation * current_anchor_right).into(),
            (second_rotation * current_anchor_left).into(),
        )
    }
}
