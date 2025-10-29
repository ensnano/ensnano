use ensnano_design::{Helix, HelixParameters};
use rapier3d::{
    na::{Const, OVector, Rotation2, Unit, UnitQuaternion},
    prelude::*,
};
use ultraviolet::{Rotor3, Vec3};

use crate::vec_to_vector;

/// An internal structure, which is made from an Helix
/// and computes the ideal spring anchor points to enforce
/// a rigid, straight DNA or RNA strand
#[derive(Clone, Debug)]
pub(crate) struct SpringAnchorsReference {
    nucleotide_forward: OVector<f32, Const<3>>,

    // from center of the pair to the center of
    // the next pair
    up: OVector<f32, Const<3>>,

    up_forward_anchor: OVector<f32, Const<3>>,
    up_backward_anchor: OVector<f32, Const<3>>,
    up_left_anchor: OVector<f32, Const<3>>,
    up_right_anchor: OVector<f32, Const<3>>,
    down_forward_anchor: OVector<f32, Const<3>>,
    down_backward_anchor: OVector<f32, Const<3>>,
    down_left_anchor: OVector<f32, Const<3>>,
    down_right_anchor: OVector<f32, Const<3>>,
}

/// Turns two nucleotides 90° around the "up" axis,
/// resulting in "left" and "right" anchors for
/// better stability
fn turn_points(
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
        rotation * c_forward + center,
        rotation * c_backward + center,
    )
}

impl SpringAnchorsReference {
    /// Initializes a new reference with the given Helix's parameters,
    /// and to a provided distance. Higher distance means
    pub(crate) fn new(helix: &Helix, distance: u32, default_parameters: &HelixParameters) -> Self {
        let helix_parameters = helix.helix_parameters.clone();
        let mut helix = Helix::new(Vec3::default(), Rotor3::default());
        helix.helix_parameters = helix_parameters;

        let nucleotide_forward = vec_to_vector(helix.space_pos(default_parameters, 0, true));
        let nucleotide_backward = vec_to_vector(helix.space_pos(default_parameters, 0, false));

        let center = (nucleotide_forward + nucleotide_backward) / 2.0;

        let up_nucleotide_forward =
            vec_to_vector(helix.space_pos(default_parameters, distance as isize, true));
        let up_nucleotide_backward =
            vec_to_vector(helix.space_pos(default_parameters, distance as isize, false));

        let up_center = (up_nucleotide_forward + up_nucleotide_backward) / 2.0;

        let up = up_center - center;

        let up_up_nucleotide_forward =
            vec_to_vector(helix.space_pos(default_parameters, 2 * (distance as isize), true));
        let up_up_nucleotide_backward =
            vec_to_vector(helix.space_pos(default_parameters, 2 * (distance as isize), false));

        let up_up_center = (up_up_nucleotide_forward + up_up_nucleotide_backward) / 2.0;

        // up's up direction
        let up_up = up_up_center - up_center;

        let down_nucleotide_forward =
            vec_to_vector(helix.space_pos(default_parameters, -(distance as isize), true));
        let down_nucleotide_backward =
            vec_to_vector(helix.space_pos(default_parameters, -(distance as isize), false));

        let down_center = (down_nucleotide_forward + down_nucleotide_backward) / 2.0;

        // down's up direction
        let down_up = center - down_center;

        // we compute the left and right anchors by rotating each nucletide pair in its
        // local up axis

        let (up_left, up_right) = turn_points(up_nucleotide_forward, up_nucleotide_backward, up_up);
        let (left, right) = turn_points(nucleotide_forward, nucleotide_backward, up);
        let (down_left, down_right) =
            turn_points(down_nucleotide_forward, down_nucleotide_backward, down_up);

        let up_forward_anchor = (nucleotide_forward + up_nucleotide_forward) / 2.0;
        let up_backward_anchor = (nucleotide_backward + up_nucleotide_backward) / 2.0;
        let up_left_anchor = (up_left + left) / 2.0;
        let up_right_anchor = (up_right + right) / 2.0;

        let down_forward_anchor = (nucleotide_forward + down_nucleotide_forward) / 2.0;
        let down_backward_anchor = (nucleotide_backward + down_nucleotide_backward) / 2.0;
        let down_left_anchor = (down_left + left) / 2.0;
        let down_right_anchor = (down_right + right) / 2.0;

        Self {
            nucleotide_forward: nucleotide_forward - center,
            up,
            up_forward_anchor: up_forward_anchor - center,
            up_backward_anchor: up_backward_anchor - center,
            down_forward_anchor: down_forward_anchor - center,
            down_backward_anchor: down_backward_anchor - center,
            up_left_anchor: up_left_anchor - center,
            up_right_anchor: up_right_anchor - center,
            down_left_anchor: down_left_anchor - center,
            down_right_anchor: down_right_anchor - center,
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
    /// Return order : (forward, backward, left, right)
    pub(crate) fn get_up_spring_anchors(
        &self,
        forward_nucleotide: OVector<f32, Const<3>>,
        backward_nucleotide: OVector<f32, Const<3>>,
        up: OVector<f32, Const<3>>,
    ) -> (Point<f32>, Point<f32>, Point<f32>, Point<f32>) {
        let target_center = (forward_nucleotide + backward_nucleotide) / 2.0;

        let target_forward = forward_nucleotide - target_center;
        let target_up = up;

        let first_rotation = self.first_rotation(target_forward);

        let current_up = first_rotation * self.up;
        let current_anchor_forward = first_rotation * self.up_forward_anchor;
        let current_anchor_backward = first_rotation * self.up_backward_anchor;
        let current_anchor_left = first_rotation * self.up_left_anchor;
        let current_anchor_right = first_rotation * self.up_right_anchor;

        let second_rotation = Self::second_rotation(current_up, target_forward, target_up);

        (
            (second_rotation * current_anchor_forward + target_center).into(),
            (second_rotation * current_anchor_backward + target_center).into(),
            (second_rotation * current_anchor_left + target_center).into(),
            (second_rotation * current_anchor_right + target_center).into(),
        )
    }

    /// Returns the spring anchors required for the base pair provided in parameters,
    /// and a vector pointing to the center of the upper targeted pair.
    /// This uses both uses the precomputed reference in self,
    /// and a sequence of two rotations to place it in the orientation
    /// of the desired nucleotide.
    /// Return order : (forward, backward, left, right)
    pub(crate) fn get_down_spring_anchors(
        &self,
        forward_nucleotide: OVector<f32, Const<3>>,
        backward_nucleotide: OVector<f32, Const<3>>,
        up: OVector<f32, Const<3>>,
    ) -> (Point<f32>, Point<f32>, Point<f32>, Point<f32>) {
        let target_center = (forward_nucleotide + backward_nucleotide) / 2.0;

        let target_forward = forward_nucleotide - target_center;
        let target_up = up;

        let first_rotation = self.first_rotation(target_forward);

        let current_up = first_rotation * self.up;
        let current_anchor_forward = first_rotation * self.down_forward_anchor;
        let current_anchor_backward = first_rotation * self.down_backward_anchor;
        let current_anchor_left = first_rotation * self.down_left_anchor;
        let current_anchor_right = first_rotation * self.down_right_anchor;

        let second_rotation = Self::second_rotation(current_up, target_forward, target_up);

        (
            (second_rotation * current_anchor_forward + target_center).into(),
            (second_rotation * current_anchor_backward + target_center).into(),
            (second_rotation * current_anchor_left + target_center).into(),
            (second_rotation * current_anchor_right + target_center).into(),
        )
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use ensnano_design::HelixParameters;

    #[test]
    fn anchors() {
        // "artificial" custom parameters
        // that make it easy to predict positions
        let parameters = HelixParameters {
            rise: 1.0,
            helix_radius: 1.0,
            bases_per_turn: 4.0,
            groove_angle: std::f32::consts::PI,
            inter_helix_gap: 1.0,
            inclination: 0.0,
        };

        let helix = Helix::new(Vec3::zero(), Rotor3::default());

        let reference = SpringAnchorsReference::new(&helix, 1, &parameters);
        let reference_backward_nucleotide = vec_to_vector(helix.space_pos(&parameters, 0, false));

        let eps: f32 = 1e-5;

        assert!(
            reference
                .nucleotide_forward
                .metric_distance(&vector![0.0, -1.0, 0.0])
                < eps
        );
        assert!(reference.up.metric_distance(&vector![1.0, 0.0, 0.0]) < eps);
        assert!(
            reference
                .up_forward_anchor
                .metric_distance(&vector![0.5, -0.5, -0.5])
                < eps
        );
        assert!(
            reference
                .up_backward_anchor
                .metric_distance(&vector![0.5, 0.5, 0.5])
                < eps
        );
        assert!(
            reference
                .up_left_anchor
                .metric_distance(&vector![0.5, 0.5, -0.5])
                < eps
        );
        assert!(
            reference
                .up_right_anchor
                .metric_distance(&vector![0.5, -0.5, 0.5])
                < eps
        );
        assert!(
            reference
                .down_forward_anchor
                .metric_distance(&vector![-0.5, -0.5, 0.5])
                < eps
        );
        assert!(
            reference
                .down_backward_anchor
                .metric_distance(&vector![-0.5, 0.5, -0.5])
                < eps
        );
        assert!(
            reference
                .down_left_anchor
                .metric_distance(&vector![-0.5, -0.5, -0.5])
                < eps
        );
        assert!(
            reference
                .down_right_anchor
                .metric_distance(&vector![-0.5, 0.5, 0.5])
                < eps
        );

        // test with identity transform
        let forward_nucleotide = reference.nucleotide_forward;
        let backward_nucleotide = reference_backward_nucleotide;
        let up = reference.up;

        let (forward, backward, left, right) =
            reference.get_up_spring_anchors(forward_nucleotide, backward_nucleotide, up);

        assert!(forward.coords.metric_distance(&reference.up_forward_anchor) < eps);
        assert!(
            backward
                .coords
                .metric_distance(&reference.up_backward_anchor)
                < eps
        );
        assert!(left.coords.metric_distance(&reference.up_left_anchor) < eps);
        assert!(right.coords.metric_distance(&reference.up_right_anchor) < eps);

        let (forward, backward, left, right) =
            reference.get_down_spring_anchors(forward_nucleotide, backward_nucleotide, up);

        assert!(
            forward
                .coords
                .metric_distance(&reference.down_forward_anchor)
                < eps
        );
        assert!(
            backward
                .coords
                .metric_distance(&reference.down_backward_anchor)
                < eps
        );
        assert!(left.coords.metric_distance(&reference.down_left_anchor) < eps);
        assert!(right.coords.metric_distance(&reference.down_right_anchor) < eps);

        // test with a translation
        let offset = vector![32.0, -43.5, 0.111];
        let forward_nucleotide = reference.nucleotide_forward + offset;
        let backward_nucleotide = reference_backward_nucleotide + offset;
        let up = reference.up;

        let (forward, backward, left, right) =
            reference.get_up_spring_anchors(forward_nucleotide, backward_nucleotide, up);

        assert!(
            forward
                .coords
                .metric_distance(&(reference.up_forward_anchor + offset))
                < eps
        );
        assert!(
            backward
                .coords
                .metric_distance(&(reference.up_backward_anchor + offset))
                < eps
        );
        assert!(
            left.coords
                .metric_distance(&(reference.up_left_anchor + offset))
                < eps
        );
        assert!(
            right
                .coords
                .metric_distance(&(reference.up_right_anchor + offset))
                < eps
        );

        let (forward, backward, left, right) =
            reference.get_down_spring_anchors(forward_nucleotide, backward_nucleotide, up);

        assert!(
            forward
                .coords
                .metric_distance(&(reference.down_forward_anchor + offset))
                < eps
        );
        assert!(
            backward
                .coords
                .metric_distance(&(reference.down_backward_anchor + offset))
                < eps
        );
        assert!(
            left.coords
                .metric_distance(&(reference.down_left_anchor + offset))
                < eps
        );
        assert!(
            right
                .coords
                .metric_distance(&(reference.down_right_anchor + offset))
                < eps
        );

        // test with a rotation
        let rotation = UnitQuaternion::from_euler_angles(0.45, 0.111, -1.5);
        let forward_nucleotide = rotation * reference.nucleotide_forward;
        let backward_nucleotide = rotation * reference_backward_nucleotide;
        let up = rotation * reference.up;

        let (forward, backward, left, right) =
            reference.get_up_spring_anchors(forward_nucleotide, backward_nucleotide, up);

        assert!(
            forward
                .coords
                .metric_distance(&(rotation * reference.up_forward_anchor))
                < eps
        );
        assert!(
            backward
                .coords
                .metric_distance(&(rotation * reference.up_backward_anchor))
                < eps
        );
        assert!(
            left.coords
                .metric_distance(&(rotation * reference.up_left_anchor))
                < eps
        );
        assert!(
            right
                .coords
                .metric_distance(&(rotation * reference.up_right_anchor))
                < eps
        );

        let (forward, backward, left, right) =
            reference.get_down_spring_anchors(forward_nucleotide, backward_nucleotide, up);

        assert!(
            forward
                .coords
                .metric_distance(&(rotation * reference.down_forward_anchor))
                < eps
        );
        assert!(
            backward
                .coords
                .metric_distance(&(rotation * reference.down_backward_anchor))
                < eps
        );
        assert!(
            left.coords
                .metric_distance(&(rotation * reference.down_left_anchor))
                < eps
        );
        assert!(
            right
                .coords
                .metric_distance(&(rotation * reference.down_right_anchor))
                < eps
        );

        // test with a rotation
        let rotation = UnitQuaternion::from_euler_angles(-0.3, 0.4, 0.9);
        let offset = vector![10.0, 22.222222, -3.0];
        let forward_nucleotide = rotation * reference.nucleotide_forward + offset;
        let backward_nucleotide = rotation * reference_backward_nucleotide + offset;
        let up = rotation * reference.up;

        let (forward, backward, left, right) =
            reference.get_up_spring_anchors(forward_nucleotide, backward_nucleotide, up);

        assert!(
            forward
                .coords
                .metric_distance(&(rotation * reference.up_forward_anchor + offset))
                < eps
        );
        assert!(
            backward
                .coords
                .metric_distance(&(rotation * reference.up_backward_anchor + offset))
                < eps
        );
        assert!(
            left.coords
                .metric_distance(&(rotation * reference.up_left_anchor + offset))
                < eps
        );

        assert!(
            right
                .coords
                .metric_distance(&(rotation * reference.up_right_anchor + offset))
                < eps
        );

        let (forward, backward, left, right) =
            reference.get_down_spring_anchors(forward_nucleotide, backward_nucleotide, up);

        assert!(
            forward
                .coords
                .metric_distance(&(rotation * reference.down_forward_anchor + offset))
                < eps
        );
        assert!(
            backward
                .coords
                .metric_distance(&(rotation * reference.down_backward_anchor + offset))
                < eps
        );
        assert!(
            left.coords
                .metric_distance(&(rotation * reference.down_left_anchor + offset))
                < eps
        );
        assert!(
            right
                .coords
                .metric_distance(&(rotation * reference.down_right_anchor + offset))
                < eps
        );
    }
}
