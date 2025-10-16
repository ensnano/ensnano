use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckXoversParameter {
    None,
    Checked,
    Unchecked,
    Both,
}

impl Default for CheckXoversParameter {
    fn default() -> Self {
        Self::None
    }
}

impl ToString for CheckXoversParameter {
    fn to_string(&self) -> String {
        match self {
            Self::None => String::from("None"),
            Self::Checked => String::from("Checked"),
            Self::Unchecked => String::from("Unchecked"),
            Self::Both => String::from("Both"),
        }
    }
}

impl CheckXoversParameter {
    pub const ALL: &'static [Self] = &[Self::None, Self::Checked, Self::Unchecked, Self::Both];

    pub fn wants_checked(&self) -> bool {
        match self {
            Self::Checked | Self::Both => true,
            Self::None | Self::Unchecked => false,
        }
    }

    pub fn wants_unchecked(&self) -> bool {
        match self {
            Self::Unchecked | Self::Both => true,
            Self::None | Self::Checked => false,
        }
    }
}
