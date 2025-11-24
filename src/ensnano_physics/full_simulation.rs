use crate::ensnano_design::{Helices, HelixCollection as _, HelixParameters, Nucl};
use crate::ensnano_interactor::ObjectType;
use crate::ensnano_physics::{
    RapierPhysicsSystem,
    anchors::SpringAnchorsReference,
    helices::{IntermediaryHelix, IntermediaryPair},
    point_from_parts,
};
use ahash::HashMap;
use itertools::Itertools as _;
use rapier3d::prelude::*;

const NUCLEOTIDE_RADIUS: f32 = 0.05;
const PAIR_CAPSULE_RADIUS: f32 = 0.1;

const BASE_LINEAR_DAMPING: f32 = 0.04;
const BASE_ANGULAR_DAMPING: f32 = 0.04;

const STRONG_SPRING_RANGES: [u32; 4] = [1, 2, 4, 8];

const INTERBASE_SPRING_STIFFNESS: f32 = 10000.0;
const INTERBASE_SPRING_DAMPING: f32 = 1000.0;

const CROSSOVER_STIFFNESS: f32 = 100.0;
const CROSSOVER_DAMPING: f32 = 50.0;
const CROSSOVER_SIZE: f32 = 0.64;

// const FREE_NUCLEOTIDE_STIFFNESS: f32 = 100000.0;
// const FREE_NUCLEOTIDE_DAMPING: f32 = 50000.0;
// const FREE_NUCLEOTIDE_DISTANCE: f32 = 0.64;

/// A trait to represent a strategy of how to attach
/// colliders to rigid bodies in the simulation.
/// This is meant to differentiate between
/// full simulation (all bases are simulated),
/// rigid helices (helices are one rigid body),
/// or sliced rigid helices (helices are rigid bodies
/// separated at crossovers)
pub trait SimulationSetup {
    // creates rigid bodes and assigns the provided
    // colliders to them
    fn build_bodies(
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        collider_map: &HashMap<(usize, isize), Vec<ColliderHandle>>,
        intermediary_representation: &HashMap<usize, IntermediaryHelix>,
    );
}

// This is used for full simulations; not used right now, but will be
// with a proper interface.
#[expect(dead_code)]
pub struct FullSimulationSetup;

impl SimulationSetup for FullSimulationSetup {
    fn build_bodies(
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        collider_map: &HashMap<(usize, isize), Vec<ColliderHandle>>,
        intermediary_representation: &HashMap<usize, IntermediaryHelix>,
    ) {
        for (helix_index, helix) in intermediary_representation {
            for pair_position in helix.pairs.keys() {
                let rigid_body = RigidBodyBuilder::dynamic()
                    .linear_damping(BASE_LINEAR_DAMPING)
                    .angular_damping(BASE_ANGULAR_DAMPING);

                let rigid_body_handle = rigid_body_set.insert(rigid_body);

                for collider_handle in collider_map
                    .get(&(*helix_index, *pair_position))
                    .unwrap_or(&vec![])
                {
                    collider_set.set_parent(
                        *collider_handle,
                        Some(rigid_body_handle),
                        rigid_body_set,
                    );
                }
            }
        }
    }
}

pub struct RigidHelicesSetup;

impl SimulationSetup for RigidHelicesSetup {
    fn build_bodies(
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        collider_map: &HashMap<(usize, isize), Vec<ColliderHandle>>,
        intermediary_representation: &HashMap<usize, IntermediaryHelix>,
    ) {
        for (helix_index, helix) in intermediary_representation {
            let rigid_body = RigidBodyBuilder::dynamic()
                .linear_damping(BASE_LINEAR_DAMPING)
                .angular_damping(BASE_ANGULAR_DAMPING);
            let rigid_body_handle = rigid_body_set.insert(rigid_body);
            for pair_position in helix.pairs.keys() {
                for collider_handle in collider_map
                    .get(&(*helix_index, *pair_position))
                    .unwrap_or(&vec![])
                {
                    collider_set.set_parent(
                        *collider_handle,
                        Some(rigid_body_handle),
                        rigid_body_set,
                    );
                }
            }
        }
    }
}

pub fn build_simulation<S: SimulationSetup>(
    intermediary_representation: &HashMap<usize, IntermediaryHelix>,
    object_type: &HashMap<u32, ObjectType>,
    nucleotide: &HashMap<u32, Nucl>,
    space_position: &HashMap<u32, [f32; 3]>,
    helices: &Helices,
    global_parameters: &HelixParameters,
) -> RapierPhysicsSystem {
    let mut rigid_body_set: RigidBodySet = Default::default();
    let mut collider_set: ColliderSet = Default::default();
    let mut impulse_joint_set: ImpulseJointSet = Default::default();

    let mut nucleotide_body_map: HashMap<u32, ColliderHandle> = Default::default();

    // Stores the colliders per helix and per position in the helix
    let mut collider_map: HashMap<(usize, isize), Vec<ColliderHandle>> = Default::default();

    // Create objects and store them in the map
    // 1) for each helix, for each level of the helix
    // 2) create either 2 balls and one capsule, or just 1 ball
    // 3) and place the resulting nucleotide colliders in a new map

    build_colliders(
        &mut rigid_body_set,
        &mut collider_set,
        &mut nucleotide_body_map,
        &mut collider_map,
        intermediary_representation,
        space_position,
    );

    S::build_bodies(
        &mut rigid_body_set,
        &mut collider_set,
        &collider_map,
        intermediary_representation,
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
        &collider_set,
        &mut impulse_joint_set,
        global_parameters,
    );

    // TODO add free nucleotide springs
    // 1) add code in the intermediary representation X
    // to detect single threaded ranges
    // 2) add simple springs on those single threaded ranges

    // add crossover springs
    add_crossover_springs(
        object_type,
        nucleotide,
        &nucleotide_body_map,
        &collider_set,
        &mut impulse_joint_set,
    );

    // we return the physics system
    RapierPhysicsSystem {
        rigid_body_set,
        collider_set,
        impulse_joint_set,
        nucleotide_body_map,
        ..Default::default()
    }
}

/// Builds the rigid bodies necessary for the simulation.
/// Fills nucleotide body map and  by indicating the colliders that correspond
/// to the nucleotides.
fn build_colliders(
    rigid_body_set: &mut RigidBodySet,
    collider_set: &mut ColliderSet,
    nucleotide_body_map: &mut HashMap<u32, ColliderHandle>,
    collider_map: &mut HashMap<(usize, isize), Vec<ColliderHandle>>,
    intermediary_representation: &HashMap<usize, IntermediaryHelix>,
    space_position: &HashMap<u32, [f32; 3]>,
) {
    // This dummy body is here to "hold" the collider before
    // they get reassigned to a proper body in a later step.
    // Pacôme : I tried doing it with only "insert", which
    // doesn't link to a body, but then it wouldn't work.
    // Looking at the rapier source it looks like insert does
    // a lot less work than insert_with_parent, and since
    // doing it this way works...
    let dummy_body = rigid_body_set.insert(RigidBodyBuilder::dynamic());

    for helix in intermediary_representation.values() {
        for pair in helix.pairs.values() {
            match pair {
                IntermediaryPair::Pair(i, n, j) => {
                    let i_p = space_position
                        .get(i)
                        .expect("Couldn't get position of nucl");
                    let j_p = space_position
                        .get(j)
                        .expect("Couldn't get position of nucl");

                    let i_collider = ColliderBuilder::ball(NUCLEOTIDE_RADIUS)
                        .position(Isometry::translation(i_p[0], i_p[1], i_p[2]))
                        .active_collision_types(ActiveCollisionTypes::empty())
                        .collision_groups(InteractionGroups::new(Group::GROUP_2, Group::empty()));
                    let j_collider = ColliderBuilder::ball(NUCLEOTIDE_RADIUS)
                        .position(Isometry::translation(j_p[0], j_p[1], j_p[2]))
                        .active_collision_types(ActiveCollisionTypes::empty())
                        .collision_groups(InteractionGroups::new(Group::GROUP_2, Group::empty()));

                    let capsule = ColliderBuilder::capsule_from_endpoints(
                        point_from_parts(i_p),
                        point_from_parts(j_p),
                        PAIR_CAPSULE_RADIUS,
                    );

                    let i_collider_handle =
                        collider_set.insert_with_parent(i_collider, dummy_body, rigid_body_set);
                    let j_collider_handle =
                        collider_set.insert_with_parent(j_collider, dummy_body, rigid_body_set);

                    let capsule_handle =
                        collider_set.insert_with_parent(capsule, dummy_body, rigid_body_set);

                    nucleotide_body_map.insert(*i, i_collider_handle);
                    nucleotide_body_map.insert(*j, j_collider_handle);

                    collider_map.insert(
                        (n.helix, n.position),
                        vec![i_collider_handle, j_collider_handle, capsule_handle],
                    );
                }
                IntermediaryPair::OnlyForward(id, n) | IntermediaryPair::OnlyBackward(id, n) => {
                    let position = space_position
                        .get(id)
                        .expect("Couldn't get position of nucl");

                    let collider = ColliderBuilder::ball(NUCLEOTIDE_RADIUS)
                        .position(Isometry::translation(position[0], position[1], position[2]));

                    let collider_handle =
                        collider_set.insert_with_parent(collider, dummy_body, rigid_body_set);

                    nucleotide_body_map.insert(*id, collider_handle);
                    collider_map.insert((n.helix, n.position), vec![collider_handle]);
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
    let IntermediaryPair::Pair(up_i, _, up_j) = up_pair else {
        panic!("Incoherent double ranges");
    };
    let IntermediaryPair::Pair(down_i, _, down_j) = down_pair else {
        panic!("Incoherent double ranges");
    };

    let up_i = collider_set.get(nucleotide_body_map[up_i]).unwrap();
    let up_j = collider_set.get(nucleotide_body_map[up_j]).unwrap();
    let down_i = collider_set.get(nucleotide_body_map[down_i]).unwrap();
    let down_j = collider_set.get(nucleotide_body_map[down_j]).unwrap();

    let up_center = (up_i.translation() + up_j.translation()) / 2.0;
    let down_center = (down_i.translation() + down_j.translation()) / 2.0;

    (up_center - down_center).normalize()
}

fn build_strong_springs(
    intermediary_representation: &HashMap<usize, IntermediaryHelix>,
    nucleotide_body_map: &HashMap<u32, ColliderHandle>,
    helices: &Helices,
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

            for (&a, &b) in range.iter().tuple_windows() {
                let down_pair = intermediary.pairs[&a];
                let up_pair = intermediary.pairs[&b];

                let result = up_vector(&down_pair, &up_pair, nucleotide_body_map, collider_set);

                up_vectors.insert(a, result);
                // we always insert a copy up
                // so that the last pair also gets
                // an up vector
                up_vectors.insert(a + 1, result);
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
                    let down_up = &up_vectors[down_index];

                    let IntermediaryPair::Pair(down_i, _, down_j) = down_pair else {
                        panic!("Incoherent double ranges");
                    };

                    let down_forward = collider_set.get(nucleotide_body_map[&down_i]).unwrap();
                    let down_backward = collider_set.get(nucleotide_body_map[&down_j]).unwrap();

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
                    let up_up = &up_vectors[up_index];

                    let IntermediaryPair::Pair(up_i, _, up_j) = up_pair else {
                        panic!("Incoherent double ranges");
                    };

                    let up_forward = collider_set.get(nucleotide_body_map[&up_i]).unwrap();
                    let up_backward = collider_set.get(nucleotide_body_map[&up_j]).unwrap();

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

                    // we don't attach rigid bodies to themselves
                    if up_body_handle == down_body_handle {
                        continue;
                    }

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

pub fn add_crossover_springs(
    object_type: &HashMap<u32, ObjectType>,
    nucleotide: &HashMap<u32, Nucl>,
    nucleotide_body_map: &HashMap<u32, ColliderHandle>,
    collider_set: &ColliderSet,
    impulse_joint_set: &mut ImpulseJointSet,
) {
    let mut bonds: Vec<(u32, u32)> = Default::default();

    for ty in object_type.values() {
        match ty {
            ObjectType::Bond(a, b) | ObjectType::SlicedBond(_, a, b, _) => {
                if nucleotide[a].helix == nucleotide[b].helix {
                    continue;
                }
                bonds.push((*a, *b));
            }
            _ => {}
        }
    }

    // for each bond, a spring
    for (a, b) in bonds {
        let a = collider_set
            .get(nucleotide_body_map[&a])
            .expect("Error fetching nucleotide body");
        let b = collider_set
            .get(nucleotide_body_map[&b])
            .expect("Error fetching nucleotide body");

        impulse_joint_set.insert(
            a.parent().expect("Collider without parent"),
            b.parent().expect("Collider without parent"),
            SpringJointBuilder::new(CROSSOVER_SIZE, CROSSOVER_STIFFNESS, CROSSOVER_DAMPING)
                .local_anchor1(a.position_wrt_parent().unwrap().translation.vector.into())
                .local_anchor2(b.position_wrt_parent().unwrap().translation.vector.into())
                .build(),
            true,
        );
    }
}
