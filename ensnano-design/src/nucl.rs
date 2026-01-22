use crate::helices::Helices;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct Nucl {
    pub helix: usize,
    pub position: isize,
    pub forward: bool,
}

impl PartialOrd for Nucl {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Nucl {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.helix != other.helix {
            self.helix.cmp(&other.helix)
        } else if self.forward != other.forward {
            self.forward.cmp(&other.forward)
        } else if self.forward {
            self.position.cmp(&other.position)
        } else {
            self.position.cmp(&other.position).reverse()
        }
    }
}

impl Nucl {
    pub fn new(helix: usize, position: isize, forward: bool) -> Self {
        Self {
            helix,
            position,
            forward,
        }
    }

    #[must_use]
    pub fn left(&self) -> Self {
        Self {
            position: self.position - 1,
            ..*self
        }
    }

    #[must_use]
    pub fn right(&self) -> Self {
        Self {
            position: self.position + 1,
            ..*self
        }
    }

    #[must_use]
    pub fn prime3(&self) -> Self {
        Self {
            position: if self.forward {
                self.position + 1
            } else {
                self.position - 1
            },
            ..*self
        }
    }

    #[must_use]
    pub fn prime5(&self) -> Self {
        Self {
            position: if self.forward {
                self.position - 1
            } else {
                self.position + 1
            },
            ..*self
        }
    }

    #[must_use]
    pub fn compl(&self) -> Self {
        Self {
            forward: !self.forward,
            ..*self
        }
    }

    pub fn is_neighbor(&self, other: &Self) -> bool {
        self.helix == other.helix
            && self.forward == other.forward
            && (self.position - other.position).abs() == 1
    }

    pub fn map_to_virtual_nucl(nucl: Self, helices: &Helices) -> Option<VirtualNucl> {
        let h = helices.get(&nucl.helix)?;
        let support_helix_id = h
            .support_helix
            .or(Some(nucl.helix))
            .filter(|h_id| helices.contains_key(h_id))?;
        Some(VirtualNucl(Self {
            helix: support_helix_id,
            position: nucl.position + h.initial_nt_index,
            forward: nucl.forward,
        }))
    }
}

impl std::fmt::Display for Nucl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.helix, self.position, self.forward)
    }
}

/// The virtual position of a nucleotide.
///
/// Two nucleotides on different helices with the same support helix will be mapped
/// to the same `VirtualNucl` if they are at the same position on that support helix
#[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub struct VirtualNucl(pub(crate) Nucl);

impl VirtualNucl {
    #[must_use]
    pub fn compl(&self) -> Self {
        Self(self.0.compl())
    }
}
