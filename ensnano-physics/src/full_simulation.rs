use ahash::HashMap;
use ensnano_design::{Helices, HelixCollection, HelixParameters};
use rapier3d::{na::Point3, prelude::*};

use crate::{
    RapierPhysicsSystem, anchors::SpringAnchorsReference, helices::IntermediaryHelix,
    point_from_parts, vec_to_vector,
};

const NUCLEOTIDE_RADIUS: f32 = 0.1;
const PAIR_CAPSULE_RADIUS: f32 = 0.15;

const BASE_LINEAR_DAMPING: f32 = 0.04;
const BASE_ANGULAR_DAMPING: f32 = 0.04;

const STRONG_SPRING_RANGES: [u32; 3] = [2, 3, 5];

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

    // TODO create springs from double helix portions;
    // 1) for each helix, for each window size
    // create a reference of that size for that helix
    // iterate windows and place "anchors" according to the reference
    // actually create the springs with those anchors

    // TODO add free nucleotide springs
    // 1) add code in the intermediary representation X
    // to detect single threaded ranges
    // 2) add simple springs on those single threaded ranges

    todo!()
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
                crate::helices::IntermediaryPair::Pair(i, _, j, _) => {
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
        // todo : extract the "up" vector for each pair in a double
        // helix range
        // -> top pairs in each range use the inverse of the down direction
        // instead
        // (we could do an average here between up and -down when both exist)
        for range in STRONG_SPRING_RANGES {
            let reference = SpringAnchorsReference::new(helix, range, global_parameters);

            // todo : make windows on the double helices range
            // add springs to each end of the window based on
            // the reference's anchors, using the up vectors
            // computed earlier
        }
    }
}
