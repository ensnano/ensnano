use crate::organizer::element::{AttributeDisplay, AttributeWidget, OrganizerAttribute};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub enum DesignElement {
    Grid {
        id: usize,
        visible: bool,
    },
    Strand {
        id: usize,
        length: usize,
        domain_lengths: Vec<usize>,
    },
    Helix {
        id: usize,
        group: Option<bool>,
        locked_for_simulations: bool,
    },
    Nucleotide {
        helix: usize,
        position: isize,
        forward: bool,
    },
    CrossOver {
        xover_id: usize,
        helix5prime: usize,
        position5prime: isize,
        forward5prime: bool,
        helix3prime: usize,
        position3prime: isize,
        forward3prime: bool,
    },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum DnaAutoGroup {
    StrandWithLength(BoundedLength),
    StrandWithDomainOfLength(BoundedLength),
}

impl std::fmt::Display for DnaAutoGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StrandWithLength(length) => write!(f, "Strands with length {length}"),
            Self::StrandWithDomainOfLength(length) => match length {
                BoundedLength::Last(_, _) => {
                    write!(f, "Strand with domains of lengths {length}")
                }
                _ => write!(f, "Strands with a domain of length {length}"),
            },
        }
    }
}

const LONG: usize = 100;
const SHORT: usize = 4;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum BoundedLength {
    Short,
    Between(usize),
    Long(usize),
    Last(usize, usize),
}

impl From<(usize, (usize, usize))> for BoundedLength {
    fn from(n_bounds: (usize, (usize, usize))) -> Self {
        let n = n_bounds.0;
        let (last_lengths_min, last_lengths_max) = n_bounds.1;
        if n >= last_lengths_min {
            if last_lengths_min == last_lengths_max {
                Self::Long(last_lengths_min)
            } else {
                Self::Last(last_lengths_min, last_lengths_max)
            }
        } else if n < SHORT {
            Self::Short
        } else {
            Self::Between(n)
        }
    }
}

impl std::fmt::Display for BoundedLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Last(ll_min, ll_max) => write!(f, "≥ {ll_min} (max {ll_max})"),
            Self::Long(m) => write!(f, "> {m}"),
            Self::Short => write!(f, "< {SHORT}"),
            Self::Between(n) => write!(f, "= {n}"),
        }
    }
}

impl DesignElement {
    pub fn key(&self) -> DesignElementKey {
        match self {
            Self::Grid { id, .. } => DesignElementKey::Grid(*id),
            Self::Strand { id, .. } => DesignElementKey::Strand(*id),
            Self::Helix { id, .. } => DesignElementKey::Helix(*id),
            Self::Nucleotide {
                helix,
                position,
                forward,
            } => DesignElementKey::Nucleotide {
                helix: *helix,
                position: *position,
                forward: *forward,
            },
            Self::CrossOver { xover_id, .. } => DesignElementKey::CrossOver {
                xover_id: *xover_id,
            },
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            Self::Grid { id, .. } => format!("Grid {id}"),
            Self::Strand { id, .. } => format!("Strand {id}"),
            Self::Helix { id, .. } => format!("Helix {id}"),
            Self::Nucleotide {
                helix,
                position,
                forward,
            } => format!("Nucl {helix}:{position}:{forward}"),
            Self::CrossOver {
                helix5prime,
                position5prime,
                forward5prime,
                helix3prime,
                position3prime,
                forward3prime,
                ..
            } => format!(
                "Xover ({helix5prime}:{position5prime}:{forward5prime}) -> ({helix3prime}:{position3prime}:{forward3prime})"
            ),
        }
    }

    pub fn attributes(&self) -> Vec<DnaAttribute> {
        match self {
            Self::Helix {
                group,
                locked_for_simulations: locked,
                ..
            } => vec![
                DnaAttribute::XoverGroup(*group),
                DnaAttribute::LockedForSimulations(*locked),
            ],
            Self::Grid { visible, .. } => vec![DnaAttribute::Visible(*visible)],
            _ => vec![],
        }
    }

    pub fn min_max_domain_length_if_strand(&self) -> Option<(usize, usize)> {
        match self {
            Self::Strand { domain_lengths, .. } => match (
                domain_lengths.clone().iter().min().copied(),
                domain_lengths.clone().iter().max().copied(),
            ) {
                (Some(n_min), Some(n_max)) => Some((n_min, n_max)),
                _ => None,
            },

            _ => None,
        }
    }

    pub fn auto_groups(&self, last_domain_length_bounds: (usize, usize)) -> Vec<DnaAutoGroup> {
        match self {
            Self::Strand {
                length,
                domain_lengths,
                ..
            } => {
                let mut ret = vec![DnaAutoGroup::StrandWithLength(
                    (*length, (LONG, LONG)).into(),
                )];
                let mut lengths = domain_lengths.clone();
                lengths.sort();
                lengths.dedup();
                for len in lengths {
                    ret.push(DnaAutoGroup::StrandWithDomainOfLength(
                        (len, last_domain_length_bounds).into(),
                    ));
                }
                ret
            }
            _ => vec![],
        }
    }

    pub fn all_discriminants() -> &'static [DnaAttributeDiscriminant] {
        DnaAttribute::all_discriminants()
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash, Copy)]
pub enum DesignElementKey {
    Grid(usize),
    Strand(usize),
    Helix(usize),
    Nucleotide {
        helix: usize,
        position: isize,
        forward: bool,
    },
    CrossOver {
        xover_id: usize,
    },
}

impl DesignElementKey {
    pub fn name(section: DesignElementSection) -> String {
        match section {
            DesignElementSection::Grid => "Grid".to_owned(),
            DesignElementSection::Helix => "Helix".to_owned(),
            DesignElementSection::Strand => "Strand".to_owned(),
            DesignElementSection::CrossOver => "CrossOver".to_owned(),
            DesignElementSection::Nucleotide => "Nucleotide".to_owned(),
        }
    }

    pub fn section(&self) -> DesignElementSection {
        match self {
            Self::Strand(_) => DesignElementSection::Strand,
            Self::Helix(_) => DesignElementSection::Helix,
            Self::Nucleotide { .. } => DesignElementSection::Nucleotide,
            Self::CrossOver { .. } => DesignElementSection::CrossOver,
            Self::Grid { .. } => DesignElementSection::Grid,
        }
    }
}

/// Default sections of the DesignElement
///
/// NOTE: This enum derives TryFromPrimitive. This allow to get the section from an usize with the
///       method .try_into().
#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(usize)]
pub enum DesignElementSection {
    Grid,
    Helix,
    Strand,
    CrossOver,
    Nucleotide,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DnaAttribute {
    Visible(bool),
    XoverGroup(Option<bool>),
    LockedForSimulations(bool),
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(usize)]
pub enum DnaAttributeDiscriminant {
    Visible,
    XoverGroup,
    LockedForSimulations,
}

impl DnaAttributeDiscriminant {
    pub fn all_discriminants() -> &'static [Self] {
        &[Self::Visible, Self::XoverGroup, Self::LockedForSimulations]
    }
}

impl OrganizerAttribute for DnaAttribute {
    fn discriminant(&self) -> DnaAttributeDiscriminant {
        match self {
            Self::Visible(_) => DnaAttributeDiscriminant::Visible,
            Self::XoverGroup(_) => DnaAttributeDiscriminant::XoverGroup,
            Self::LockedForSimulations(_) => DnaAttributeDiscriminant::LockedForSimulations,
        }
    }

    fn widget(&self) -> AttributeWidget<Self> {
        match self {
            Self::Visible(b) => AttributeWidget::new(Self::Visible(!b)),
            Self::LockedForSimulations(b) => AttributeWidget::new(Self::LockedForSimulations(!b)),
            Self::XoverGroup(None) => AttributeWidget::new(Self::XoverGroup(Some(false))),
            Self::XoverGroup(Some(b)) => AttributeWidget::new(if *b {
                Self::XoverGroup(None)
            } else {
                Self::XoverGroup(Some(true))
            }),
        }
    }

    fn char_repr(&self) -> AttributeDisplay {
        match self {
            Self::Visible(b) => AttributeDisplay::Icon(if *b {
                icondata::BsEyeFill
            } else {
                icondata::BsEyeSlash
            }),
            Self::XoverGroup(group) => match group {
                None => AttributeDisplay::Text("\u{2205}".to_owned()),
                Some(false) => AttributeDisplay::Text("G".to_owned()),
                Some(true) => AttributeDisplay::Text("R".to_owned()),
            },
            Self::LockedForSimulations(b) => AttributeDisplay::Icon(if *b {
                icondata::BsLock
            } else {
                icondata::BsUnlock
            }),
        }
    }
}
