//! Encoding of phantom element identifier.
//! The identifier is an integer of the form helix_id * max_pos_id + pos_id.
//!
//! helix_id is the identifier of the helix and pos_id is of the form
//! position * nb_kind + element_kid
//! where element_kind is
//! 0 for forward nucl
//! 1 for backward nucl
//! 2 for forward bond
//! 3 for backward bond.
//!
//! and position is a number between -PHANTOM_RANGE and PHANTOM_RANGE that is made positive by
//! adding PHANTOM_RANGE to it.

use crate::nucl::Nucl;

pub const PHANTOM_RANGE: i32 = 1000;

/// Generate the identifier of a phantom nucleotide.
pub fn phantom_helix_encoder_nucl(
    design_id: u32,
    helix_id: u32,
    position: i32,
    forward: bool,
) -> u32 {
    let pos_id = (position + PHANTOM_RANGE) as u32 * 4 + !forward as u32;
    let max_pos_id = (2 * PHANTOM_RANGE) as u32 * 4 + 3;
    let helix = helix_id * max_pos_id;
    assert!(helix <= 0xFF_FF_FF);
    (helix + pos_id) | (design_id << 24)
}

/// Generate the identifier of a phantom bond.
pub fn phantom_helix_encoder_bond(
    design_id: u32,
    helix_id: u32,
    position: i32,
    forward: bool,
) -> u32 {
    let pos_id = (position + PHANTOM_RANGE) as u32 * 4 + if forward { 2 } else { 3 };
    let max_pos_id = (2 * PHANTOM_RANGE) as u32 * 4 + 3;
    let helix = helix_id * max_pos_id;
    assert!(helix <= 0xFF_FF_FF);
    (helix + pos_id) | (design_id << 24)
}

pub fn phantom_helix_decoder(id: u32) -> PhantomElement {
    let max_pos_id = (2 * PHANTOM_RANGE) as u32 * 4 + 3;
    let design_id = id >> 24;
    let reminder = id & 0xFF_FF_FF;
    let helix_id = reminder / max_pos_id;
    let reminder = reminder % max_pos_id;
    let bond = reminder & 0b10 > 0;
    let forward = reminder.is_multiple_of(2);
    let nucl_id = reminder / 4;
    let position = nucl_id as i32 - PHANTOM_RANGE;
    PhantomElement {
        design_id,
        helix_id,
        position,
        bond,
        forward,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhantomElement {
    pub design_id: u32,
    pub helix_id: u32,
    pub position: i32,
    pub bond: bool,
    pub forward: bool,
}

impl PhantomElement {
    pub fn to_nucl(self) -> Nucl {
        Nucl {
            helix: self.helix_id as usize,
            position: self.position as isize,
            forward: self.forward,
        }
    }
}
