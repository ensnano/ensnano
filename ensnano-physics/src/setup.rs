//! This module creates the actual contents of the Rapier simulation.

use crate::{
    anchors::SpringAnchorsReference,
    helices::{IntermediaryHelix, IntermediaryPair},
    parameters::RapierParameters,
    simulation::RapierPhysicsSystem,
    vec_to_vector,
};
use ahash::HashMap;
use ensnano_design::{
    design_element::DesignElement,
    helices::{Helices, NuclCollection},
    nucl::Nucl,
    parameters::HelixParameters,
};
use rapier3d::{
    na::{Const, OVector},
    prelude::*,
};

const NUCLEOTIDE_RADIUS: f32 = 0.32;

/// A trait to represent a strategy of how to attach
/// colliders to rigid bodies in the simulation.
/// This is meant to differentiate between
/// full simulation (all bases are simulated),
/// rigid helices (helices are one rigid body),
/// or sliced rigid helices (helices are rigid bodies
/// separated at crossovers).
pub(crate) trait SimulationSetup {
    // creates rigid bodes and assigns the provided
    // colliders to them
    fn build_bodies(
        &self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        collider_map: &HashMap<(usize, isize), Vec<ColliderHandle>>,
        intermediary_representation: &HashMap<usize, IntermediaryHelix>,
        rapier_parameters: &RapierParameters,
    );
}

/// This is used for full simulations. This is the default setting and
/// should be preferred over other ones if performances allow.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct FullSimulationSetup;

impl SimulationSetup for FullSimulationSetup {
    fn build_bodies(
        &self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        collider_map: &HashMap<(usize, isize), Vec<ColliderHandle>>,
        intermediary_representation: &HashMap<usize, IntermediaryHelix>,
        rapier_parameters: &RapierParameters,
    ) {
        for (helix_index, helix) in intermediary_representation {
            for pair_position in helix.pairs.keys() {
                let rigid_body = RigidBodyBuilder::dynamic()
                    .linear_damping(rapier_parameters.linear_damping)
                    .angular_damping(rapier_parameters.angular_damping);

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

/// Builds the entire simulation, taking a generic parameter setup
/// that indicates how to regroup the colliders in rigid bodies.
pub(crate) fn build_simulation<S: SimulationSetup>(
    setup: S,
    intermediary_representation: &HashMap<usize, IntermediaryHelix>,
    nucl_collection: &NuclCollection,
    elements: &Vec<DesignElement>,
    space_position: &HashMap<u32, [f32; 3]>,
    helices: &Helices,
    global_parameters: &HelixParameters,
    rapier_parameters: &RapierParameters,
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

    setup.build_bodies(
        &mut rigid_body_set,
        &mut collider_set,
        &collider_map,
        intermediary_representation,
        rapier_parameters,
    );

    prevent_bodies_from_sleeping(&mut rigid_body_set);

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
        rapier_parameters,
    );

    // add free nucleotide springs
    build_free_springs(
        intermediary_representation,
        &nucleotide_body_map,
        &collider_set,
        &mut impulse_joint_set,
        rapier_parameters,
    );

    // add crossover springs
    let crossovers = add_crossover_springs(
        elements,
        nucl_collection,
        &nucleotide_body_map,
        &collider_set,
        &mut impulse_joint_set,
        rapier_parameters,
    );

    // we return the physics system
    RapierPhysicsSystem {
        rigid_body_set,
        collider_set,
        impulse_joint_set,
        crossovers,
        nucleotide_body_map,
        ..Default::default()
    }
}

/// Simulation performance is not important here, so
/// we make sure the rigid bodies never go to sleep when
/// they are slow enough.
fn prevent_bodies_from_sleeping(bodies: &mut RigidBodySet) {
    for (_, body) in bodies.iter_mut() {
        *body.activation_mut() = RigidBodyActivation::cannot_sleep();
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

    for (helix_id, helix) in intermediary_representation {
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
                        .collision_groups(InteractionGroups::new(
                            Group::GROUP_1 | Group::GROUP_2,
                            Group::GROUP_1 | Group::GROUP_2,
                            InteractionTestMode::And,
                        ))
                        // we indicate the helix of the nucleotide in user data
                        .user_data(*helix_id as u128);
                    let j_collider = ColliderBuilder::ball(NUCLEOTIDE_RADIUS)
                        .position(Isometry::translation(j_p[0], j_p[1], j_p[2]))
                        .collision_groups(InteractionGroups::new(
                            Group::GROUP_1 | Group::GROUP_2,
                            Group::GROUP_1 | Group::GROUP_2,
                            InteractionTestMode::And,
                        ))
                        // we indicate the helix of the nucleotide in user data
                        .user_data(*helix_id as u128);

                    // let capsule = ColliderBuilder::capsule_from_endpoints(
                    //     point_from_parts(i_p),
                    //     point_from_parts(j_p),
                    //     PAIR_CAPSULE_RADIUS,
                    // );

                    let i_collider_handle =
                        collider_set.insert_with_parent(i_collider, dummy_body, rigid_body_set);
                    let j_collider_handle =
                        collider_set.insert_with_parent(j_collider, dummy_body, rigid_body_set);

                    // let capsule_handle =
                    //     collider_set.insert_with_parent(capsule, dummy_body, rigid_body_set);

                    nucleotide_body_map.insert(*i, i_collider_handle);
                    nucleotide_body_map.insert(*j, j_collider_handle);

                    collider_map.insert(
                        (n.helix, n.position),
                        // vec![i_collider_handle, j_collider_handle, capsule_handle],
                        vec![i_collider_handle, j_collider_handle],
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

/// Makes the so called strong springs, which connect pairs of nucleotides
/// along helices.
fn build_strong_springs(
    intermediary_representation: &HashMap<usize, IntermediaryHelix>,
    nucleotide_body_map: &HashMap<u32, ColliderHandle>,
    helices: &Helices,
    collider_set: &ColliderSet,
    impulse_joint_set: &mut ImpulseJointSet,
    global_parameters: &HelixParameters,
    rapier_parameters: &RapierParameters,
) {
    let distance = 1;

    for (id, intermediary) in intermediary_representation {
        let helix = helices
            .get(id)
            .expect("Couldn't find an helix in spring creation");
        let reference = SpringAnchorsReference::new(
            helix,
            distance,
            global_parameters,
            rapier_parameters.ignore_local_parameters,
        );

        let mut last_index = None;

        for range in &intermediary.double_ranges {
            // one long ranges have no springs inside
            if range.start == range.end {
                continue;
            }

            let up_vectors = range
                .clone()
                .map(|k| {
                    (
                        k,
                        vec_to_vector(helix.normal_at_pos(k + helix.initial_nt_index, true)),
                    )
                })
                .collect::<HashMap<_, _>>();

            // we use a vec to use Slice::window
            let range = range.clone().collect::<Vec<_>>();

            // we make windows on the double helices range
            // add springs to each end of the window based on
            // the reference's anchors, using the up vectors
            // computed earlier

            for window in range.windows(2) {
                let down_index = window.first().unwrap();
                let up_index = window.last().unwrap();

                match last_index {
                    Some(index) => {
                        if *up_index > index {
                            last_index = Some(*up_index);
                        }
                    }
                    None => last_index = Some(*up_index),
                }

                insert_strong_spring(
                    &reference,
                    nucleotide_body_map,
                    impulse_joint_set,
                    collider_set,
                    rapier_parameters,
                    &up_vectors,
                    intermediary,
                    down_index,
                    up_index,
                );
            }
        }

        if let Some(index) = last_index
            && intermediary.is_cyclic
        {
            let up_vectors = [0, index]
                .into_iter()
                .map(|k| {
                    (
                        k,
                        vec_to_vector(helix.normal_at_pos(k + helix.initial_nt_index, true)),
                    )
                })
                .collect::<HashMap<_, _>>();

            insert_strong_spring(
                &reference,
                nucleotide_body_map,
                impulse_joint_set,
                collider_set,
                rapier_parameters,
                &up_vectors,
                intermediary,
                &index,
                &0,
            );
        }
    }
}

/// Helper method to simplify the previous one.
fn insert_strong_spring(
    reference: &SpringAnchorsReference,
    nucleotide_body_map: &HashMap<u32, ColliderHandle>,
    impulse_joint_set: &mut ImpulseJointSet,
    collider_set: &ColliderSet,
    rapier_parameters: &RapierParameters,
    up_vectors: &HashMap<isize, OVector<f32, Const<3>>>,
    intermediary: &IntermediaryHelix,
    down_index: &isize,
    up_index: &isize,
) {
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
    let (down_forward, down_backward, down_left, down_right) = reference.get_up_spring_anchors(
        *down_forward.translation(),
        *down_backward.translation(),
        *down_up,
    );

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
    let (up_forward, up_backward, up_left, up_right) = reference.get_down_spring_anchors(
        *up_forward.translation(),
        *up_backward.translation(),
        *up_up,
    );

    // we don't attach rigid bodies to themselves
    if up_body_handle == down_body_handle {
        return;
    }

    // forward spring
    impulse_joint_set.insert(
        down_body_handle,
        up_body_handle,
        SpringJointBuilder::new(
            0.0,
            rapier_parameters.interbase_spring_stiffness,
            rapier_parameters.interbase_spring_damping,
        )
        .local_anchor1(down_forward)
        .local_anchor2(up_forward)
        .spring_model(MotorModel::ForceBased)
        .build(),
        true,
    );

    // backward spring
    impulse_joint_set.insert(
        down_body_handle,
        up_body_handle,
        SpringJointBuilder::new(
            0.0,
            rapier_parameters.interbase_spring_stiffness,
            rapier_parameters.interbase_spring_damping,
        )
        .local_anchor1(down_backward)
        .local_anchor2(up_backward)
        .spring_model(MotorModel::ForceBased)
        .build(),
        true,
    );

    // left spring
    impulse_joint_set.insert(
        down_body_handle,
        up_body_handle,
        SpringJointBuilder::new(
            0.0,
            rapier_parameters.interbase_spring_stiffness,
            rapier_parameters.interbase_spring_damping,
        )
        .local_anchor1(down_left)
        .local_anchor2(up_left)
        .spring_model(MotorModel::ForceBased)
        .build(),
        true,
    );

    // right spring
    impulse_joint_set.insert(
        down_body_handle,
        up_body_handle,
        SpringJointBuilder::new(
            0.0,
            rapier_parameters.interbase_spring_stiffness,
            rapier_parameters.interbase_spring_damping,
        )
        .local_anchor1(down_right)
        .local_anchor2(up_right)
        .spring_model(MotorModel::ForceBased)
        .build(),
        true,
    );
}

/// Free springs connect free nucleotides to other objects.
fn build_free_springs(
    intermediary_representation: &HashMap<usize, IntermediaryHelix>,
    nucleotide_body_map: &HashMap<u32, ColliderHandle>,
    collider_set: &ColliderSet,
    impulse_joint_set: &mut ImpulseJointSet,
    rapier_parameters: &RapierParameters,
) {
    for intermediary in intermediary_representation.values() {
        for range in &intermediary.single_ranges {
            // we extend the range to connect to the external bits
            let extended_range = range.start - 1..range.end + 1;

            for down in extended_range {
                let up = down + 1;

                // we take both nucleotides
                // -> with the extension, we could be
                // off range; just continue in this case
                let Some(down_pair) = intermediary.pairs.get(&down) else {
                    continue;
                };
                let Some(up_pair) = intermediary.pairs.get(&up) else {
                    continue;
                };

                // we find the corresponding colliders
                // -> one of them could be a double
                let Some((i, _, j, _)) = down_pair.match_single(up_pair) else {
                    continue;
                };

                let up_collider = nucleotide_body_map[&i];
                let down_collider = nucleotide_body_map[&j];

                let Some(up_body_handle) = collider_set
                    .get(up_collider)
                    .expect("Couldn't find collider")
                    .parent()
                else {
                    continue;
                };
                let Some(down_body_handle) = collider_set
                    .get(down_collider)
                    .expect("Couldn't find collider")
                    .parent()
                else {
                    continue;
                };

                let down_offset = collider_set
                    .get(down_collider)
                    .expect("Couldn't find collider")
                    .position();
                let up_offset = collider_set
                    .get(up_collider)
                    .expect("Couldn't find collider")
                    .position();

                // we don't attach rigid bodies to themselves
                if up_body_handle == down_body_handle {
                    continue;
                }

                // we compute the offsets from the position of
                // the nucleotide colliders

                // free nucleotide spring
                impulse_joint_set.insert(
                    down_body_handle,
                    up_body_handle,
                    SpringJointBuilder::new(
                        rapier_parameters.free_nucleotide_rest_length,
                        rapier_parameters.free_nucleotide_stiffness,
                        rapier_parameters.free_nucleotide_damping,
                    )
                    .local_anchor1(down_offset.translation.vector.into())
                    .local_anchor2(up_offset.translation.vector.into())
                    .build(),
                    true,
                );
            }

            // here we create the entropic spring

            if rapier_parameters.entropic_spring_strength <= 0.0 {
                continue;
            }

            let Some(down_pair) = intermediary.pairs.get(&range.start) else {
                continue;
            };
            let Some(up_pair) = intermediary.pairs.get(&(range.end - 1)) else {
                continue;
            };

            // we find the corresponding colliders
            // -> one of them could be a double
            let Some((i, _, j, _)) = down_pair.match_single(up_pair) else {
                continue;
            };

            let up_collider = nucleotide_body_map[&i];
            let down_collider = nucleotide_body_map[&j];

            let Some(up_body_handle) = collider_set
                .get(up_collider)
                .expect("Couldn't find collider")
                .parent()
            else {
                continue;
            };
            let Some(down_body_handle) = collider_set
                .get(down_collider)
                .expect("Couldn't find collider")
                .parent()
            else {
                continue;
            };

            let down_offset = collider_set
                .get(down_collider)
                .expect("Couldn't find collider")
                .position();
            let up_offset = collider_set
                .get(up_collider)
                .expect("Couldn't find collider")
                .position();

            // we don't attach rigid bodies to themselves
            if up_body_handle == down_body_handle {
                continue;
            }

            // We model an entropic spring with stiffness the inverse of the
            // square root of the length of the chain.
            let len = range.end - range.start;
            let factor = 1.0 / (len as f32).sqrt();

            let strength = rapier_parameters.entropic_spring_strength;

            // free nucleotide spring
            impulse_joint_set.insert(
                down_body_handle,
                up_body_handle,
                SpringJointBuilder::new(
                    // entropic springs have rest length 0
                    0.0,
                    factor * strength,
                    rapier_parameters.entropic_spring_damping,
                )
                .local_anchor1(down_offset.translation.vector.into())
                .local_anchor2(up_offset.translation.vector.into())
                .build(),
                true,
            );
        }
    }
}

/// Crossover springs are simple springs, but with different parameters.
pub(crate) fn add_crossover_springs(
    elements: &Vec<DesignElement>,
    nucl_collection: &NuclCollection,
    nucleotide_body_map: &HashMap<u32, ColliderHandle>,
    collider_set: &ColliderSet,
    impulse_joint_set: &mut ImpulseJointSet,
    rapier_parameters: &RapierParameters,
) -> Vec<(ColliderHandle, ColliderHandle)> {
    let mut bonds: Vec<(u32, u32)> = Default::default();

    for element in elements {
        if let DesignElement::CrossOver {
            helix5prime,
            position5prime,
            forward5prime,
            helix3prime,
            position3prime,
            forward3prime,
            ..
        } = element
        {
            if helix5prime == helix3prime {
                continue;
            }

            let Some(&a) = nucl_collection.identifier.get(&Nucl {
                helix: *helix5prime,
                position: *position5prime,
                forward: *forward5prime,
            }) else {
                continue;
            };

            let Some(&b) = nucl_collection.identifier.get(&Nucl {
                helix: *helix3prime,
                position: *position3prime,
                forward: *forward3prime,
            }) else {
                continue;
            };

            bonds.push((a, b));
        }
    }

    let mut result = vec![];

    // for each bond, a spring
    for (a, b) in bonds {
        // this spring might be between "virtual" nucleotides which
        // are not real.
        if !nucleotide_body_map.contains_key(&a) || !nucleotide_body_map.contains_key(&b) {
            continue;
        }

        result.push((nucleotide_body_map[&a], nucleotide_body_map[&b]));
        let a = collider_set
            .get(nucleotide_body_map[&a])
            .expect("Error fetching nucleotide body");
        let b = collider_set
            .get(nucleotide_body_map[&b])
            .expect("Error fetching nucleotide body");

        impulse_joint_set.insert(
            a.parent().expect("Collider without parent"),
            b.parent().expect("Collider without parent"),
            SpringJointBuilder::new(
                rapier_parameters.crossover_rest_length,
                rapier_parameters.crossover_stiffness,
                rapier_parameters.crossover_damping,
            )
            .local_anchor1(a.position_wrt_parent().unwrap().translation.vector.into())
            .local_anchor2(b.position_wrt_parent().unwrap().translation.vector.into())
            .build(),
            true,
        );
    }

    result
}
