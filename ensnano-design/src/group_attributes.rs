use serde::{Deserialize, Serialize};
use ultraviolet::{Rotor3, Vec3};

/// The attributes of a group.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupAttribute {
    pub pivot: Option<GroupPivot>,
}

/// The position and orientation of the pivot used to rotate/translate the group
#[derive(Copy, Debug, Clone, Serialize, Deserialize)]
pub struct GroupPivot {
    pub position: Vec3,
    pub orientation: Rotor3,
}
