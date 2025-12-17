pub mod helix_interval;

use self::helix_interval::HelixInterval;
use crate::{
    helices::{Helices, HelixCollection as _},
    insertions::InstantiatedInsertion,
    nucl::{Nucl, VirtualNucl},
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, sync::Arc};

/// A domain can be either an interval of nucleotides on an helix, or an "Insertion" that is a set
/// of nucleotides that are not on an helix and form an independent loop.
#[derive(Clone, Serialize, Deserialize)]
pub enum Domain {
    /// An interval of nucleotides on an helix
    HelixDomain(HelixInterval),
    /// A set of nucleotides not on an helix.
    Insertion {
        nb_nucl: usize,
        #[serde(
            skip,
            default,
            alias = "instanciation", // cspell: disable-line
        )]
        instantiation: Option<Arc<InstantiatedInsertion>>,
        #[serde(default)]
        sequence: Option<Cow<'static, str>>,
        #[serde(default)]
        attached_to_prime3: bool,
    },
}

impl Domain {
    pub fn length(&self) -> usize {
        match self {
            Self::Insertion { nb_nucl, .. } => *nb_nucl,
            Self::HelixDomain(interval) => (interval.end - interval.start).max(0) as usize,
        }
    }

    pub fn other_end(&self, nucl: Nucl) -> Option<isize> {
        match self {
            Self::Insertion { .. } => None,
            Self::HelixDomain(interval) => {
                if interval.helix == nucl.helix && nucl.forward == interval.forward {
                    if interval.start == nucl.position {
                        Some(interval.end - 1)
                    } else if interval.end - 1 == nucl.position {
                        Some(interval.start)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    pub fn prime5_end(&self) -> Option<Nucl> {
        match self {
            Self::Insertion { .. } => None,
            Self::HelixDomain(interval) => {
                let position = if interval.forward {
                    interval.start
                } else {
                    interval.end - 1
                };
                Some(Nucl {
                    helix: interval.helix,
                    position,
                    forward: interval.forward,
                })
            }
        }
    }

    pub fn prime3_end(&self) -> Option<Nucl> {
        match self {
            Self::Insertion { .. } => None,
            Self::HelixDomain(interval) => {
                let position = if interval.forward {
                    interval.end - 1
                } else {
                    interval.start
                };
                Some(Nucl {
                    helix: interval.helix,
                    position,
                    forward: interval.forward,
                })
            }
        }
    }

    pub fn has_nucl(&self, nucl: &Nucl) -> Option<usize> {
        match self {
            Self::Insertion { .. } => None,
            Self::HelixDomain(HelixInterval {
                forward,
                start,
                end,
                helix,
                ..
            }) => {
                if *helix == nucl.helix && *forward == nucl.forward {
                    if nucl.position >= *start && nucl.position < *end {
                        if *forward {
                            Some((nucl.position - *start) as usize)
                        } else {
                            Some((*end - 1 - nucl.position) as usize)
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    pub fn has_virtual_nucl(&self, nucl: &VirtualNucl, helices: &Helices) -> Option<usize> {
        match self {
            Self::Insertion { .. } => None,
            Self::HelixDomain(HelixInterval {
                forward,
                start,
                end,
                helix,
                ..
            }) => {
                let shift = helices.get(helix).map_or(0, |h| h.initial_nt_index);
                let helix = helices
                    .get(helix)
                    .and_then(|h| h.support_helix)
                    .unwrap_or(*helix);
                let start = start + shift;
                let end = end + shift;
                if helix == nucl.0.helix && *forward == nucl.0.forward {
                    if nucl.0.position >= start && nucl.0.position < end {
                        if *forward {
                            Some((nucl.0.position - start) as usize)
                        } else {
                            Some((end - 1 - nucl.0.position) as usize)
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Split self at position `n`, putting `n` on the 5' prime half of the split
    pub fn split(&self, n: usize) -> Option<(Self, Self)> {
        match self {
            Self::Insertion { .. } => None,
            Self::HelixDomain(HelixInterval {
                forward,
                start,
                end,
                helix,
                sequence,
            }) => {
                if (*end - 1 - *start) as usize >= n {
                    let seq_prim5;
                    let seq_prim3;
                    if let Some(seq) = sequence {
                        let seq = seq.clone().into_owned();
                        let chars = seq.chars();
                        seq_prim5 = Some(Cow::Owned(chars.clone().take(n).collect()));
                        seq_prim3 = Some(Cow::Owned(chars.clone().skip(n).collect()));
                    } else {
                        seq_prim3 = None;
                        seq_prim5 = None;
                    }
                    let dom_left;
                    let dom_right;
                    if *forward {
                        dom_left = Self::HelixDomain(HelixInterval {
                            forward: *forward,
                            start: *start,
                            end: *start + n as isize + 1,
                            helix: *helix,
                            sequence: seq_prim5,
                        });
                        dom_right = Self::HelixDomain(HelixInterval {
                            forward: *forward,
                            start: *start + n as isize + 1,
                            end: *end,
                            helix: *helix,
                            sequence: seq_prim3,
                        });
                    } else {
                        dom_right = Self::HelixDomain(HelixInterval {
                            forward: *forward,
                            start: *end - 1 - n as isize,
                            end: *end,
                            helix: *helix,
                            sequence: seq_prim3,
                        });
                        dom_left = Self::HelixDomain(HelixInterval {
                            forward: *forward,
                            start: *start,
                            end: *end - 1 - n as isize,
                            helix: *helix,
                            sequence: seq_prim5,
                        });
                    }
                    if *forward {
                        Some((dom_left, dom_right))
                    } else {
                        Some((dom_right, dom_left))
                    }
                } else {
                    None
                }
            }
        }
    }

    pub fn helix(&self) -> Option<usize> {
        match self {
            Self::HelixDomain(domain) => Some(domain.helix),
            Self::Insertion { .. } => None,
        }
    }

    pub fn half_helix(&self) -> Option<(usize, bool)> {
        match self {
            Self::HelixDomain(domain) => Some((domain.helix, domain.forward)),
            Self::Insertion { .. } => None,
        }
    }

    pub fn merge(&mut self, other: &Self) {
        let old_self = self.clone();
        match (self, other) {
            (Self::HelixDomain(dom1), Self::HelixDomain(dom2)) if dom1.helix == dom2.helix => {
                let start = dom1.start.min(dom2.start);
                let end = dom1.end.max(dom2.end);
                dom1.start = start;
                dom1.end = end;
            }
            (
                Self::Insertion {
                    nb_nucl: n1,
                    sequence,
                    ..
                },
                Self::Insertion {
                    nb_nucl: n2,
                    sequence: s2,
                    ..
                },
            ) => {
                let s1 = sequence
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default();
                let s2 = s2.as_ref().map(ToString::to_string).unwrap_or_default();
                *n1 += *n2;
                *sequence = Some(Cow::Owned(format!("{s1}{s2}")));
            }
            _ => println!("Warning attempt to merge unmergeable domains {old_self:?}, {other:?}",),
        }
    }

    #[expect(clippy::suspicious_operation_groupings)]
    pub fn can_merge(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::HelixDomain(dom1), Self::HelixDomain(dom2)) => {
                if dom1.forward {
                    dom1.helix == dom2.helix
                        && dom1.end == dom2.start
                        && dom1.forward == dom2.forward
                } else {
                    dom1.helix == dom2.helix
                        && dom1.start == dom2.end
                        && dom1.forward == dom2.forward
                }
            }
            (Self::Insertion { .. }, Self::Insertion { .. }) => true,
            _ => false,
        }
    }

    #[expect(clippy::suspicious_operation_groupings)]
    pub fn intersect(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::HelixDomain(dom1), Self::HelixDomain(dom2)) => {
                dom1.helix == dom2.helix
                    && dom1.start < dom2.end
                    && dom2.start < dom1.end
                    && dom1.forward == dom2.forward
            }
            _ => false,
        }
    }

    pub fn new_insertion(nb_nucl: usize) -> Self {
        Self::Insertion {
            nb_nucl,
            instantiation: None,
            sequence: None,
            attached_to_prime3: false,
        }
    }

    pub fn new_prime5_insertion(nb_nucl: usize) -> Self {
        Self::Insertion {
            nb_nucl,
            instantiation: None,
            sequence: None,
            attached_to_prime3: true,
        }
    }

    pub fn is_neighbor(&self, other: &Self) -> bool {
        if let (
            Self::HelixDomain(HelixInterval {
                start: my_start, ..
            }),
            Self::HelixDomain(HelixInterval {
                start: other_start, ..
            }),
        ) = (self, other)
        {
            let my_helix = self.half_helix();

            my_helix.is_some()
                && my_helix == other.half_helix()
                && (*my_start == 0 || *other_start == 0)
        } else {
            false
        }
    }
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Insertion { nb_nucl, .. } => write!(f, "[@{nb_nucl}]"),
            Self::HelixDomain(dom) => write!(f, "{dom}"),
        }
    }
}

impl std::fmt::Debug for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

struct InsertionAccumulator {
    attached_to_prime3: bool,
    length: usize,
    sequence: String,
}

/// Return a list of domains that validate the condition SaneDomains
///
/// SaneDomains: There must always be a Domain::HelixDomain between two Domain::Insertion. If the
/// strand is cyclic, this include the first and the last domain.
pub fn sanitize_domains(domains: &[Domain], cyclic: bool) -> Vec<Domain> {
    let mut ret = Vec::with_capacity(domains.len());
    let mut current_insertion: Option<InsertionAccumulator> = None;
    for d in domains {
        match d {
            Domain::HelixDomain(_) => {
                if let Some(acc) = current_insertion.take() {
                    ret.push(Domain::Insertion {
                        nb_nucl: acc.length,
                        sequence: Some(acc.sequence.into()),
                        instantiation: None,
                        attached_to_prime3: acc.attached_to_prime3,
                    });
                }
                ret.push(d.clone());
            }
            Domain::Insertion {
                nb_nucl: m,
                sequence,
                attached_to_prime3,
                ..
            } => {
                if let Some(acc) = current_insertion.as_mut() {
                    acc.length += m;
                    if let Some(seq) = sequence {
                        acc.sequence.push_str(seq);
                    }
                } else {
                    current_insertion = Some(InsertionAccumulator {
                        length: *m,
                        attached_to_prime3: *attached_to_prime3,
                        sequence: sequence
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_default(),
                    });
                }
            }
        }
    }

    if let Some(mut acc) = current_insertion {
        if cyclic {
            if let Domain::Insertion { nb_nucl, .. } = ret[0].clone() {
                ret.remove(0);
                acc.length += nb_nucl;
            }
            ret.push(Domain::new_insertion(acc.length));
        } else if acc.attached_to_prime3 {
            ret.push(Domain::new_prime5_insertion(acc.length));
        } else {
            ret.push(Domain::new_insertion(acc.length));
        }
    } else if cyclic
        && let Domain::Insertion {
            nb_nucl,
            attached_to_prime3,
            ..
        } = ret[0].clone()
    {
        ret.remove(0);
        if attached_to_prime3 {
            ret.push(Domain::new_prime5_insertion(nb_nucl));
        } else {
            ret.push(Domain::new_insertion(nb_nucl));
        }
    }
    ret
}
