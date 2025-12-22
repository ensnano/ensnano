use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CheckXoversParameter {
    #[default]
    None,
    Checked,
    Unchecked,
    Both,
}

impl std::fmt::Display for CheckXoversParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Checked => write!(f, "Checked"),
            Self::Unchecked => write!(f, "Unchecked"),
            Self::Both => write!(f, "Both"),
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
