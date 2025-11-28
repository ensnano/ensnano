use crate::{
    rotor_utils::SafeRotor as _,
    view::dna_obj::{SlicedTubeInstance, TubeLidInstance},
};
use ensnano_consts::HELIX_CYLINDER_COLOR;
use ensnano_utils::instance::Instance;
use ultraviolet::{Rotor3, Vec3};

pub struct SausageRosary {
    pub positions: Vec<Vec3>,
    pub is_cyclic: bool,
}

impl SausageRosary {
    pub(crate) fn to_raw_dna_instances(
        &self,
        color: impl Fn(usize) -> u32,
        radius: f32,
        id: u32,
    ) -> (
        Vec<SlicedTubeInstance>,
        Option<(TubeLidInstance, TubeLidInstance)>,
    ) {
        let n = self.positions.len();

        if n <= 1 {
            return (vec![], None);
        }

        let prev_current_next_p1_p2 = if self.is_cyclic {
            let vecs = self
                .positions
                .iter()
                .cycle()
                .skip(n - 1)
                .zip(self.positions.iter())
                .map(|(prev, point)| *point - *prev)
                .collect::<Vec<Vec3>>();
            vecs.iter()
                .zip(vecs.iter().cycle().skip(1))
                .zip(vecs.iter().cycle().skip(2))
                .zip(self.positions.iter())
                .zip(self.positions.iter().cycle().skip(1))
                .map(|((((prev, cur), next), p1), p2)| (*prev, *cur, *next, *p1, *p2))
                .collect::<Vec<(Vec3, Vec3, Vec3, Vec3, Vec3)>>()
        } else {
            let mut vecs = vec![Vec3::zero()];
            vecs.extend(
                self.positions
                    .iter()
                    .zip(self.positions.iter().skip(1))
                    .map(|(prev, point)| *point - *prev),
            );
            vecs.push(Vec3::zero());
            vecs.iter()
                .zip(vecs.iter().skip(1))
                .zip(vecs.iter().skip(2))
                .zip(self.positions.iter())
                .zip(self.positions.iter().skip(1))
                .map(|((((prev, current), next), p1), p2)| (*prev, *current, *next, *p1, *p2))
                .collect::<Vec<(Vec3, Vec3, Vec3, Vec3, Vec3)>>()
        };

        let mut color_iter = (0..prev_current_next_p1_p2.len()).map(&color);

        let tubes = prev_current_next_p1_p2
            .into_iter()
            .map(|(prev, current, next, p1, p2)| {
                let position = (p1 + p2) / 2.;
                let normalized = current.normalized();
                let rotor = Rotor3::safe_from_rotation_from_unit_x_to(normalized);
                let rotor_inv = Rotor3::safe_from_rotation_to_unit_x_from(normalized);
                SlicedTubeInstance {
                    position,
                    rotor,
                    color: Instance::unclear_color_from_u32(
                        color_iter.next().unwrap_or(HELIX_CYLINDER_COLOR),
                    ),
                    id,
                    radius,
                    length: current.mag(),
                    prev: prev.rotated_by(rotor_inv),
                    next: next.rotated_by(rotor_inv),
                }
            })
            .collect::<Vec<SlicedTubeInstance>>();

        if self.is_cyclic {
            (tubes, None)
        } else {
            let (p0, p1) = (self.positions[0], self.positions[1]);
            let u = (p0 - p1).normalized();
            let rotor = Rotor3::safe_from_rotation_from_unit_x_to(u);

            let lid_left = TubeLidInstance {
                position: p0,
                color: Instance::unclear_color_from_u32(color(0)),
                rotor,
                id,
                radius,
            };

            let (p0, p1) = (self.positions[n - 2], self.positions[n - 1]);
            let u = (p1 - p0).normalized();
            let rotor = Rotor3::safe_from_rotation_from_unit_x_to(u);

            let lid_right = TubeLidInstance {
                position: p1,
                color: Instance::unclear_color_from_u32(color(n - 1)),
                rotor,
                id,
                radius,
            };
            (tubes, Some((lid_left, lid_right)))
        }
    }
}
