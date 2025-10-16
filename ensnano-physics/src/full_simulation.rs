use ahash::HashMap;
use ensnano_design::Helices;
use rapier3d::prelude::*;

use crate::{RapierPhysicsSystem, helices::IntermediaryHelix};

pub fn build_full_simulation(
    intermediary_representation: &HashMap<usize, IntermediaryHelix>,
    space_position: &HashMap<u32, [f32; 3]>,
    helices: &Helices,
) -> RapierPhysicsSystem {
    // TODO create objects and store them in the map
    // 1) for each helix, for each level of the helix
    // 2) create either 2 balls and one capsule, or just 1 ball
    // 3) and place the resulting nucleotide colliders in a new map

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
