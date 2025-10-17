use ensnano_design::Nucl;
use ensnano_interactor::ObjectType;
use ahash::HashMap;
use rapier3d::{na::Vector3, prelude::*};

const NUCLEOTIDE_RADIUS: f32 = 0.1;

const BASE_LINEAR_DAMPING: f32 = 0.04;
const BASE_ANGULAR_DAMPING: f32 = 0.04;

const INTERBASE_SPRING_STIFFNESS: f32 = 10000.0;
const INTERBASE_SPRING_DAMPING: f32 = 1000.0;

const CROSSOVER_STIFFNESS: f32 = 100000.0;
const CROSSOVER_DAMPING: f32 = 50000.0;
const CROSSOVER_SIZE: f32 = 0.64;

const FREE_NUCLEOTIDE_STIFFNESS: f32 = 100000.0;
const FREE_NUCLEOTIDE_DAMPING: f32 = 50000.0;
const FREE_NUCLEOTIDE_DISTANCE: f32 = 0.64;

#[derive(Copy, Clone, Debug)]
pub enum BaseOrNucleotide {
    // always backward, forward
    Base((u32, Nucl), (u32, Nucl)),
    ForwardNucleotide((u32, Nucl)),
    BackwardNucleotide((u32, Nucl)),
}

impl BaseOrNucleotide {
    pub fn into_rigid_body(
        self,
        space_position: &HashMap<u32, [f32; 3]>,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
        nucleotide_body_map: &mut HashMap<u32, ColliderHandle>,
    ) -> (
        RigidBodyHandle,
        Option<ColliderHandle>,
        Option<ColliderHandle>,
    ) {
        match self {
            BaseOrNucleotide::Base((k1, _), (k2, _)) => {
                let p1 = space_position
                    .get(&k1)
                    .expect("Couldn't get position of nucl");
                let p2 = space_position
                    .get(&k2)
                    .expect("Couldn't get position of nucl");

                let p1 = vector![p1[0], p1[1], p1[2]];
                let p2 = vector![p2[0], p2[1], p2[2]];
                let center = (p1 + p2) / 2.0;
                let p1 = p1 - center;
                let p2 = p2 - center;

                let rigid_body = RigidBodyBuilder::dynamic()
                    .translation(center.into())
                    .linear_damping(BASE_LINEAR_DAMPING)
                    .angular_damping(BASE_ANGULAR_DAMPING);

                let collider1 = ColliderBuilder::ball(NUCLEOTIDE_RADIUS)
                    .translation(p1.into())
                    .build();
                let collider2 = ColliderBuilder::ball(NUCLEOTIDE_RADIUS)
                    .translation(p2.into())
                    .build();

                let rigid_body_handle = rigid_body_set.insert(rigid_body);
                let collider_handle1 =
                    collider_set.insert_with_parent(collider1, rigid_body_handle, rigid_body_set);
                let collider_handle2 =
                    collider_set.insert_with_parent(collider2, rigid_body_handle, rigid_body_set);

                nucleotide_body_map.insert(k1, collider_handle1);
                nucleotide_body_map.insert(k2, collider_handle2);

                (
                    rigid_body_handle,
                    Some(collider_handle1),
                    Some(collider_handle2),
                )
            }
            BaseOrNucleotide::BackwardNucleotide((k1, _)) => {
                let position = space_position
                    .get(&k1)
                    .expect("Couldn't get position of nucl");
                let rigid_body = RigidBodyBuilder::dynamic()
                    .linear_damping(BASE_LINEAR_DAMPING)
                    .angular_damping(BASE_ANGULAR_DAMPING);
                let collider = ColliderBuilder::ball(NUCLEOTIDE_RADIUS)
                    .position(Isometry::translation(position[0], position[1], position[2]));

                let rigid_body_handle = rigid_body_set.insert(rigid_body);
                let collider_handle =
                    collider_set.insert_with_parent(collider, rigid_body_handle, rigid_body_set);

                nucleotide_body_map.insert(k1, collider_handle);

                (rigid_body_handle, Some(collider_handle), None)
            }
            BaseOrNucleotide::ForwardNucleotide((k2, _)) => {
                let position = space_position
                    .get(&k2)
                    .expect("Couldn't get position of nucl");
                let rigid_body = RigidBodyBuilder::dynamic()
                    .linear_damping(BASE_LINEAR_DAMPING)
                    .angular_damping(BASE_ANGULAR_DAMPING);
                let collider = ColliderBuilder::ball(NUCLEOTIDE_RADIUS)
                    .position(Isometry::translation(position[0], position[1], position[2]));

                let rigid_body_handle = rigid_body_set.insert(rigid_body);
                let collider_handle =
                    collider_set.insert_with_parent(collider, rigid_body_handle, rigid_body_set);

                nucleotide_body_map.insert(k2, collider_handle);

                (rigid_body_handle, None, Some(collider_handle))
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

    for (_, ty) in object_type.iter() {
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

pub fn generate_springs(
    from: (
        RigidBodyHandle,
        Option<ColliderHandle>,
        Option<ColliderHandle>,
    ),
    to: (
        RigidBodyHandle,
        Option<ColliderHandle>,
        Option<ColliderHandle>,
    ),
    rigid_body_set: &mut RigidBodySet,
    collider_set: &mut ColliderSet,
    impulse_joint_set: &mut ImpulseJointSet,
) {
    if from.1.is_some() && from.2.is_some() && to.1.is_some() && to.2.is_some() {
        // This is the case of two base pair, which implies strong springs

        link_with_springs(
            from.1.unwrap(),
            from.2.unwrap(),
            to.1.unwrap(),
            to.2.unwrap(),
            rigid_body_set,
            collider_set,
            impulse_joint_set,
        );

        return;
    }

    if from.1.is_some() && to.1.is_some() {
        // Here we only connect the backward nucleotides with a single spring

        generate_single_spring(
            (from.0, from.1.unwrap()),
            (to.0, to.1.unwrap()),
            collider_set,
            impulse_joint_set,
        );

        return;
    }

    if from.2.is_some() && to.2.is_some() {
        // Here we only connect the forward nucleotides with a single spring

        generate_single_spring(
            (from.0, from.2.unwrap()),
            (to.0, to.2.unwrap()),
            collider_set,
            impulse_joint_set,
        );

        return;
    }

    // in this case, we can't connect anything, so we don't add anything
}

// Tries to stick two rigidbody handles in the same place; for this, it
// supposes that the rigid bodies are at 0, 0, 0.
fn link_with_springs(
    left_a: ColliderHandle,
    right_a: ColliderHandle,
    left_b: ColliderHandle,
    right_b: ColliderHandle,
    rigid_body_set: &mut RigidBodySet,
    collider_set: &mut ColliderSet,
    impulse_joint_set: &mut ImpulseJointSet,
) {
    let a_handle = collider_set.get(left_a).unwrap().parent().unwrap();
    let pos_a = rigid_body_set.get(a_handle).unwrap().translation();
    let b_handle = collider_set.get(left_b).unwrap().parent().unwrap();
    let pos_b = rigid_body_set.get(b_handle).unwrap().translation();

    let left_a = collider_set.get(left_a).unwrap();
    let right_a = collider_set.get(right_a).unwrap();
    let left_b = collider_set.get(left_b).unwrap();
    let right_b = collider_set.get(right_b).unwrap();

    let mid_a = (left_a.translation() + right_a.translation()) / 2.0;
    let mid_b = (left_b.translation() + right_b.translation()) / 2.0;
    let mid_left = (left_a.translation() + left_b.translation()) / 2.0;
    let mid_right = (right_a.translation() + right_b.translation()) / 2.0;

    // create_spring(vector![-size, 0.0, 0.0], a, b, impulse_joint_set);
    // create_spring(vector![size, 0.0, 0.0], a, b, impulse_joint_set);
    // create_spring(vector![0.0, 0.0, -size], a, b, impulse_joint_set);
    // create_spring(vector![0.0, 0.0, size], a, b, impulse_joint_set);

    create_spring(*pos_a, *pos_b, mid_a, a_handle, b_handle, impulse_joint_set);
    create_spring(*pos_a, *pos_b, mid_b, a_handle, b_handle, impulse_joint_set);
    create_spring(
        *pos_a,
        *pos_b,
        mid_left,
        a_handle,
        b_handle,
        impulse_joint_set,
    );
    create_spring(
        *pos_a,
        *pos_b,
        mid_right,
        a_handle,
        b_handle,
        impulse_joint_set,
    );
}

fn create_spring(
    pos_a: Vector3<f32>,
    pos_b: Vector3<f32>,
    spring_point: Vector3<f32>,
    a: RigidBodyHandle,
    b: RigidBodyHandle,
    impulse_joint_set: &mut ImpulseJointSet,
) {
    impulse_joint_set.insert(
        a,
        b,
        SpringJointBuilder::new(0.0, INTERBASE_SPRING_STIFFNESS, INTERBASE_SPRING_DAMPING)
            .local_anchor1((spring_point - pos_a).into())
            .local_anchor2((spring_point - pos_b).into())
            .build(),
        true,
    );
}

fn generate_single_spring(
    from: (RigidBodyHandle, ColliderHandle),
    to: (RigidBodyHandle, ColliderHandle),
    collider_set: &mut ColliderSet,
    impulse_joint_set: &mut ImpulseJointSet,
) {
    let anchor1 = collider_set
        .get(from.1)
        .unwrap()
        .position()
        .translation
        .vector;
    let anchor2 = collider_set
        .get(to.1)
        .unwrap()
        .position()
        .translation
        .vector;

    impulse_joint_set.insert(
        from.0,
        to.0,
        SpringJointBuilder::new(
            FREE_NUCLEOTIDE_DISTANCE,
            FREE_NUCLEOTIDE_STIFFNESS,
            FREE_NUCLEOTIDE_DAMPING,
        )
        .local_anchor1(anchor1.into())
        .local_anchor2(anchor2.into())
        .build(),
        true,
    );
}

/// Generates a list of BaseOrNucleotide from the information provided
/// by ensnano
pub fn generate_intermediary_representation(
    nucleotide: &HashMap<u32, Nucl>,
) -> Vec<Vec<BaseOrNucleotide>> {
    let mut result = vec![];
    let mut current_helix = vec![];

    let mut nucleotide_list = nucleotide.clone().into_iter().collect::<Vec<_>>();

    nucleotide_list.sort_by(|(_, n), (_, m)| {
        if n.helix == m.helix {
            if n.position == m.position {
                // we have a pair, we put the not forward first
                n.forward.cmp(&m.forward)
            } else {
                n.position.cmp(&m.position)
            }
        } else {
            n.helix.cmp(&m.helix)
        }
    });

    let mut i = 0;
    while i < nucleotide_list.len() {
        let j = if i + 1 < nucleotide_list.len() {
            i + 1
        } else {
            // in this case, we only have one nucleotide left, so it is alone.
            if nucleotide_list[i].1.forward {
                current_helix.push(BaseOrNucleotide::ForwardNucleotide(
                    nucleotide_list[i].clone(),
                ));
            } else {
                current_helix.push(BaseOrNucleotide::BackwardNucleotide(
                    nucleotide_list[i].clone(),
                ));
            }
            break;
        };

        // the next 2 nucleotides form a pair
        if nucleotide_list[i].1.helix == nucleotide_list[j].1.helix
            && nucleotide_list[i].1.position == nucleotide_list[j].1.position
        {
            current_helix.push(BaseOrNucleotide::Base(
                nucleotide_list[i].clone(),
                nucleotide_list[j].clone(),
            ));

            i += 2;
            continue;
        }

        // the next nucleotide is alone
        if nucleotide_list[i].1.forward {
            current_helix.push(BaseOrNucleotide::ForwardNucleotide(
                nucleotide_list[i].clone(),
            ));
        } else {
            current_helix.push(BaseOrNucleotide::BackwardNucleotide(
                nucleotide_list[i].clone(),
            ));
        }

        // we check if the next nucleotide is connected;
        // if not, we start a new helix

        if nucleotide_list[i].1.helix != nucleotide_list[j].1.helix {
            result.push(current_helix);
            current_helix = vec![];
        }

        i += 1;
    }

    // we push the final helix
    result.push(current_helix);

    result
}
