use crate::nucl::Nucl;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// An iterator over all positions of a domain.
pub struct DomainIter {
    start: isize,
    end: isize,
    forward: bool,
}

impl Iterator for DomainIter {
    type Item = isize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            None
        } else if self.forward {
            let s = self.start;
            self.start += 1;
            Some(s)
        } else {
            let s = self.end;
            self.end -= 1;
            Some(s - 1)
        }
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct HelixInterval {
    /// Index of the helix in the array of helices. Indices start at
    /// 0.
    pub helix: usize,
    /// Position of the leftmost base of this domain along the helix
    /// (this might be the first or last base of the domain, depending
    /// on the `orientation` parameter below).
    pub start: isize,
    /// Position of the first base after the forwardmost base of the
    /// domain, along the helix. Domains must always be such that
    /// `domain.start < domain.end`.
    pub end: isize,
    /// If true, the "5' to 3'" direction of this domain runs in the
    /// same direction as the helix, i.e. "to the forward" along the
    /// axis of the helix. Else, the 5' to 3' runs to the left along
    /// the axis.
    pub forward: bool,
    /// In addition to the strand-level sequence, individual domains
    /// may have sequences too. The precedence has to be defined by
    /// the user of this library.
    pub sequence: Option<Cow<'static, str>>,
}

impl HelixInterval {
    pub fn prime5(&self) -> Nucl {
        if self.forward {
            Nucl {
                helix: self.helix,
                position: self.start,
                forward: true,
            }
        } else {
            Nucl {
                helix: self.helix,
                position: self.end - 1,
                forward: false,
            }
        }
    }

    pub fn prime3(&self) -> Nucl {
        if self.forward {
            Nucl {
                helix: self.helix,
                position: self.end - 1,
                forward: true,
            }
        } else {
            Nucl {
                helix: self.helix,
                position: self.start,
                forward: false,
            }
        }
    }

    pub fn iter(&self) -> DomainIter {
        DomainIter {
            start: self.start,
            end: self.end,
            forward: self.forward,
        }
    }
}

impl std::fmt::Display for HelixInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.forward {
            write!(f, "[H{}: {} -> {}]", self.helix, self.start, self.end - 1)
        } else {
            write!(f, "[H{}: {} <- {}]", self.helix, self.start, self.end - 1)
        }
    }
}

impl std::fmt::Debug for HelixInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self}")
    }
}
