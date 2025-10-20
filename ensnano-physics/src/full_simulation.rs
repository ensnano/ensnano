use ahash::HashMap;
use ensnano_design::{Helices, HelixCollection, HelixParameters};
use rapier3d::{na::Point3, prelude::*};

use crate::{
    RapierPhysicsSystem,
    anchors::SpringAnchorsReference,
    helices::{IntermediaryHelix, IntermediaryPair},
    point_from_parts, vec_to_vector,
};

const NUCLEOTIDE_RADIUS: f32 = 0.05;
const PAIR_CAPSULE_RADIUS: f32 = 0.1;

const BASE_LINEAR_DAMPING: f32 = 0.04;
const BASE_ANGULAR_DAMPING: f32 = 0.04;

const STRONG_SPRING_RANGES: [u32; 4] = [1, 2, 4, 8];

const INTERBASE_SPRING_STIFFNESS: f32 = 10000.0;
const INTERBASE_SPRING_DAMPING: f32 = 1000.0;

pub fn build_full_simulation(
    intermediary_representation: &HashMap<usize, IntermediaryHelix>,
    space_position: &HashMap<u32, [f32; 3]>,
    helices: &Helices,
    global_parameters: &HelixParameters,
) -> RapierPhysicsSystem {
    let mut rigid_body_set: RigidBodySet = Default::default();
    let mut collider_set: ColliderSet = Default::default();
    let mut impulse_joint_set: ImpulseJointSet = Default::default();

    let mut nucleotide_body_map: HashMap<u32, ColliderHandle> = Default::default();

    // Create objects and store them in the map
    // 1) for each helix, for each level of the helix
    // 2) create either 2 balls and one capsule, or just 1 ball
    // 3) and place the resulting nucleotide colliders in a new map

    build_bodies(
        &mut rigid_body_set,
        &mut collider_set,
        &mut nucleotide_body_map,
        intermediary_representation,
        space_position,
    );

    // create springs from double helix portions;
    // 1) for each helix, for each window size
    // create a reference of that size for that helix
    // iterate windows and place "anchors" according to the reference
    // actually create the springs with those anchors

    build_strong_springs(
        intermediary_representation,
        &nucleotide_body_map,
        helices,
        &rigid_body_set,
        &collider_set,
        &mut impulse_joint_set,
        global_parameters,
    );

    // TODO add free nucleotide springs
    // 1) add code in the intermediary representation X
    // to detect single threaded ranges
    // 2) add simple springs on those single threaded ranges

    // we return the physics system
    RapierPhysicsSystem {
        rigid_body_set,
        collider_set,
        impulse_joint_set,
        nucleotide_body_map,
        ..Default::default()
    }
}

/// Builds the rigid bodies and colliders necessary for the simulation.
/// Fills nucleotide body map by indicating the colliders that correspond
/// to the nucleotides.
fn build_bodies(
    rigid_body_set: &mut RigidBodySet,
    collider_set: &mut ColliderSet,
    nucleotide_body_map: &mut HashMap<u32, ColliderHandle>,
    intermediary_representation: &HashMap<usize, IntermediaryHelix>,
    space_position: &HashMap<u32, [f32; 3]>,
) {
    for helix in intermediary_representation.values() {
        for pair in helix.pairs.values() {
            match pair {
                crate::helices::IntermediaryPair::Pair(i, n, j, _) => {
                    let i_p = space_position
                        .get(i)
                        .expect("Couldn't get position of nucl");
                    let j_p = space_position
                        .get(j)
                        .expect("Couldn't get position of nucl");

                    let i_collider = ColliderBuilder::ball(NUCLEOTIDE_RADIUS)
                        .position(Isometry::translation(i_p[0], i_p[1], i_p[2]));
                    let j_collider = ColliderBuilder::ball(NUCLEOTIDE_RADIUS)
                        .position(Isometry::translation(j_p[0], j_p[1], j_p[2]));

                    let capsule = ColliderBuilder::capsule_from_endpoints(
                        point_from_parts(&i_p),
                        point_from_parts(&j_p),
                        PAIR_CAPSULE_RADIUS,
                    );

                    let rigid_body = RigidBodyBuilder::dynamic()
                        .linear_damping(BASE_LINEAR_DAMPING)
                        .angular_damping(BASE_ANGULAR_DAMPING);

                    let rigid_body_handle = rigid_body_set.insert(rigid_body);

                    let i_collider_handle = collider_set.insert_with_parent(
                        i_collider,
                        rigid_body_handle,
                        rigid_body_set,
                    );
                    let j_collider_handle = collider_set.insert_with_parent(
                        j_collider,
                        rigid_body_handle,
                        rigid_body_set,
                    );

                    collider_set.insert_with_parent(capsule, rigid_body_handle, rigid_body_set);

                    nucleotide_body_map.insert(*i, i_collider_handle);
                    nucleotide_body_map.insert(*j, j_collider_handle);
                }
                crate::helices::IntermediaryPair::OnlyForward(id, _)
                | crate::helices::IntermediaryPair::OnlyBackward(id, _) => {
                    let position = space_position
                        .get(id)
                        .expect("Couldn't get position of nucl");
                    let rigid_body = RigidBodyBuilder::dynamic()
                        .linear_damping(BASE_LINEAR_DAMPING)
                        .angular_damping(BASE_ANGULAR_DAMPING);
                    let collider = ColliderBuilder::ball(NUCLEOTIDE_RADIUS)
                        .position(Isometry::translation(position[0], position[1], position[2]));

                    let rigid_body_handle = rigid_body_set.insert(rigid_body);
                    let collider_handle = collider_set.insert_with_parent(
                        collider,
                        rigid_body_handle,
                        rigid_body_set,
                    );

                    nucleotide_body_map.insert(*id, collider_handle);
                }
            }
        }
    }
}

fn up_vector(
    down_pair: &IntermediaryPair,
    up_pair: &IntermediaryPair,
    nucleotide_body_map: &HashMap<u32, ColliderHandle>,
    collider_set: &ColliderSet,
) -> Vector<Real> {
    let IntermediaryPair::Pair(up_i, _, up_j, _) = up_pair else {
        panic!("Incoherent double ranges");
    };
    let IntermediaryPair::Pair(down_i, _, down_j, _) = down_pair else {
        panic!("Incoherent double ranges");
    };

    let up_i = collider_set.get(nucleotide_body_map[&up_i]).unwrap();
    let up_j = collider_set.get(nucleotide_body_map[&up_j]).unwrap();
    let down_i = collider_set.get(nucleotide_body_map[&down_i]).unwrap();
    let down_j = collider_set.get(nucleotide_body_map[&down_j]).unwrap();

    let up_center = (up_i.translation() + up_j.translation()) / 2.0;
    let down_center = (down_i.translation() + down_j.translation()) / 2.0;

    (up_center - down_center).normalize()
}

fn build_strong_springs(
    intermediary_representation: &HashMap<usize, IntermediaryHelix>,
    nucleotide_body_map: &HashMap<u32, ColliderHandle>,
    helices: &Helices,
    rigid_body_set: &RigidBodySet,
    collider_set: &ColliderSet,
    impulse_joint_set: &mut ImpulseJointSet,
    global_parameters: &HelixParameters,
) {
    for (id, intermediary) in intermediary_representation {
        let helix = helices
            .get(id)
            .expect("Couldn't find an helix in spring creation");

        for range in &intermediary.double_ranges {
            // one long ranges have no springs inside
            if range.start == range.end {
                continue;
            }

            // we use a vec to use Slice::window
            let range = range.clone().collect::<Vec<_>>();

            let mut up_vectors = HashMap::default();

            // we extract the "up" vector for each pair in a double
            // helix range
            // -> top pairs in each range use the inverse of the down direction
            // instead
            // (we could do an average here between up and -down when both exist)

            for window in range.windows(2) {
                let down_pair = intermediary.pairs[&window[0]];
                let up_pair = intermediary.pairs[&window[1]];

                let result = up_vector(&down_pair, &up_pair, nucleotide_body_map, collider_set);

                up_vectors.insert(window[0], result);
                // we always insert a copy up
                // so that the last pair also gets
                // an up vector
                up_vectors.insert(window[0] + 1, result);
            }

            for distance in STRONG_SPRING_RANGES {
                let reference = SpringAnchorsReference::new(helix, distance, global_parameters);

                // we make windows on the double helices range
                // add springs to each end of the window based on
                // the reference's anchors, using the up vectors
                // computed earlier

                for window in range.windows(distance as usize + 1) {
                    let down_index = window.first().unwrap();

                    let down_pair = intermediary.pairs[down_index];
                    let down_up = up_vectors.get(down_index).unwrap();

                    let IntermediaryPair::Pair(down_i, _, down_j, _) = down_pair else {
                        panic!("Incoherent double ranges");
                    };

                    let down_forward = collider_set.get(nucleotide_body_map[&down_i]).unwrap();
                    let down_backward = collider_set.get(nucleotide_body_map[&down_j]).unwrap();

                    let down_center =
                        (down_forward.translation() + down_backward.translation()) / 2.0;

                    let down_body_handle = collider_set
                        .get(nucleotide_body_map[&down_i])
                        .unwrap()
                        .parent()
                        .unwrap();

                    // down's up spring anchors
                    let (down_forward, down_backward, down_left, down_right) = reference
                        .get_up_spring_anchors(
                            *down_forward.translation(),
                            *down_backward.translation(),
                            *down_up,
                        );

                    let up_index = window.last().unwrap();

                    let up_pair = intermediary.pairs[up_index];
                    let up_up = up_vectors.get(up_index).unwrap();

                    let IntermediaryPair::Pair(up_i, _, up_j, _) = up_pair else {
                        panic!("Incoherent double ranges");
                    };

                    let up_forward = collider_set.get(nucleotide_body_map[&up_i]).unwrap();
                    let up_backward = collider_set.get(nucleotide_body_map[&up_j]).unwrap();

                    let up_center = (up_forward.translation() + up_backward.translation()) / 2.0;

                    let up_body_handle = collider_set
                        .get(nucleotide_body_map[&up_i])
                        .unwrap()
                        .parent()
                        .unwrap();

                    // up's down spring anchors
                    let (up_forward, up_backward, up_left, up_right) = reference
                        .get_down_spring_anchors(
                            *up_forward.translation(),
                            *up_backward.translation(),
                            *up_up,
                        );

                    // forward spring
                    impulse_joint_set.insert(
                        down_body_handle,
                        up_body_handle,
                        SpringJointBuilder::new(
                            0.0,
                            INTERBASE_SPRING_STIFFNESS,
                            INTERBASE_SPRING_DAMPING,
                        )
                        .local_anchor1(down_forward)
                        .local_anchor2(up_forward)
                        .build(),
                        true,
                    );

                    // backward spring
                    impulse_joint_set.insert(
                        down_body_handle,
                        up_body_handle,
                        SpringJointBuilder::new(
                            0.0,
                            INTERBASE_SPRING_STIFFNESS,
                            INTERBASE_SPRING_DAMPING,
                        )
                        .local_anchor1(down_backward)
                        .local_anchor2(up_backward)
                        .build(),
                        true,
                    );

                    // left spring
                    impulse_joint_set.insert(
                        down_body_handle,
                        up_body_handle,
                        SpringJointBuilder::new(
                            0.0,
                            INTERBASE_SPRING_STIFFNESS,
                            INTERBASE_SPRING_DAMPING,
                        )
                        .local_anchor1(down_left)
                        .local_anchor2(up_left)
                        .build(),
                        true,
                    );

                    // right spring
                    impulse_joint_set.insert(
                        down_body_handle,
                        up_body_handle,
                        SpringJointBuilder::new(
                            0.0,
                            INTERBASE_SPRING_STIFFNESS,
                            INTERBASE_SPRING_DAMPING,
                        )
                        .local_anchor1(down_right)
                        .local_anchor2(up_right)
                        .build(),
                        true,
                    );
                }
            }
        }
    }
}
