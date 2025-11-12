/*
ENSnano, a 3d graphical application for DNA nanostructures.
    Copyright (C) 2021  Nicolas Levy <nicolaspierrelevy@gmail.com> and Nicolas Schabanel <nicolas.schabanel@ens-lyon.fr>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use crate::ensnano_organizer::{
    AttributeDisplay, AttributeWidget, ElementKey, OrganizerAttribute,
    OrganizerAttributeDiscriminant, OrganizerElement,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

/// Actual implementation of the OrganizerElement for the LeftPanel.
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
            Self::StrandWithLength(length) => {
                write!(f, "Strands with length {}", length.to_string())
            }
            Self::StrandWithDomainOfLength(length) => match length {
                BoundedLength::Last(_, _) => {
                    write!(f, "Strand with domains of lengths {}", length.to_string())
                }
                _ => write!(f, "Strands with a domain of length {}", length.to_string()),
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

impl ToString for BoundedLength {
    fn to_string(&self) -> String {
        match self {
            Self::Last(ll_min, ll_max) => format!("≥ {ll_min} (max {ll_max})"),
            Self::Long(m) => format!("> {m}"),
            Self::Short => format!("< {SHORT}"),
            Self::Between(n) => format!("= {n}"),
        }
    }
}

impl OrganizerElement for DesignElement {
    type Attribute = DnaAttribute;
    type Key = DesignElementKey;
    type AutoGroup = DnaAutoGroup;

    fn key(&self) -> DesignElementKey {
        match self {
            DesignElement::Grid { id, .. } => DesignElementKey::Grid(*id),
            DesignElement::Strand { id, .. } => DesignElementKey::Strand(*id),
            DesignElement::Helix { id, .. } => DesignElementKey::Helix(*id),
            DesignElement::Nucleotide {
                helix,
                position,
                forward,
            } => DesignElementKey::Nucleotide {
                helix: *helix,
                position: *position,
                forward: *forward,
            },
            DesignElement::CrossOver { xover_id, .. } => DesignElementKey::CrossOver {
                xover_id: *xover_id,
            },
        }
    }

    fn display_name(&self) -> String {
        match self {
            DesignElement::Grid { id, .. } => format!("Grid {}", id),
            DesignElement::Strand { id, .. } => format!("Strand {}", id),
            DesignElement::Helix { id, .. } => format!("Helix {}", id),
            DesignElement::Nucleotide {
                helix,
                position,
                forward,
            } => format!("Nucl {}:{}:{}", helix, position, forward),
            DesignElement::CrossOver {
                helix5prime,
                position5prime,
                forward5prime,
                helix3prime,
                position3prime,
                forward3prime,
                ..
            } => format!(
                "Xover ({}:{}:{}) -> ({}:{}:{})",
                helix5prime,
                position5prime,
                forward5prime,
                helix3prime,
                position3prime,
                forward3prime
            ),
        }
    }

    fn attributes(&self) -> Vec<DnaAttribute> {
        match self {
            DesignElement::Helix {
                group,
                locked_for_simulations: locked,
                ..
            } => vec![
                DnaAttribute::XoverGroup(*group),
                DnaAttribute::LockedForSimulations(*locked),
            ],
            DesignElement::Grid { visible, .. } => vec![DnaAttribute::Visible(*visible)],
            _ => vec![],
        }
    }

    fn min_max_domain_length_if_strand(&self) -> Option<(usize, usize)> {
        match self {
            DesignElement::Strand { domain_lengths, .. } => match (
                domain_lengths.clone().iter().min().copied(),
                domain_lengths.clone().iter().max().copied(),
            ) {
                (Some(n_min), Some(n_max)) => Some((n_min, n_max)),
                _ => None,
            },

            _ => None,
        }
    }

    fn auto_groups(&self, last_domain_length_bounds: (usize, usize)) -> Vec<Self::AutoGroup> {
        match self {
            DesignElement::Strand {
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
                    ))
                }
                ret
            }
            _ => vec![],
        }
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

impl ElementKey for DesignElementKey {
    type Section = DesignElementSection;

    fn name(section: DesignElementSection) -> String {
        match section {
            DesignElementSection::Grid => "Grid".to_owned(),
            DesignElementSection::Helix => "Helix".to_owned(),
            DesignElementSection::Strand => "Strand".to_owned(),
            DesignElementSection::CrossOver => "CrossOver".to_owned(),
            DesignElementSection::Nucleotide => "Nucleotide".to_owned(),
        }
    }

    fn section(&self) -> DesignElementSection {
        match self {
            Self::Strand(_) => DesignElementSection::Strand,
            Self::Helix(_) => DesignElementSection::Helix,
            Self::Nucleotide { .. } => DesignElementSection::Nucleotide,
            Self::CrossOver { .. } => DesignElementSection::CrossOver,
            Self::Grid { .. } => DesignElementSection::Grid,
        }
    }
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

const ALL_DNA_ATTRIBUTE_DISCRIMINANTS: [DnaAttributeDiscriminant; 3] = [
    DnaAttributeDiscriminant::Visible,
    DnaAttributeDiscriminant::XoverGroup,
    DnaAttributeDiscriminant::LockedForSimulations,
];

impl OrganizerAttributeDiscriminant for DnaAttributeDiscriminant {
    fn all_discriminants() -> &'static [Self] {
        &ALL_DNA_ATTRIBUTE_DISCRIMINANTS
    }
}

impl OrganizerAttribute for DnaAttribute {
    type Discriminant = DnaAttributeDiscriminant;

    fn discriminant(&self) -> DnaAttributeDiscriminant {
        match self {
            DnaAttribute::Visible(_) => DnaAttributeDiscriminant::Visible,
            DnaAttribute::XoverGroup(_) => DnaAttributeDiscriminant::XoverGroup,
            DnaAttribute::LockedForSimulations(_) => DnaAttributeDiscriminant::LockedForSimulations,
        }
    }

    fn widget(&self) -> AttributeWidget<DnaAttribute> {
        match self {
            DnaAttribute::Visible(b) => AttributeWidget::new(DnaAttribute::Visible(!b)),
            DnaAttribute::LockedForSimulations(b) => {
                AttributeWidget::new(DnaAttribute::LockedForSimulations(!b))
            }
            DnaAttribute::XoverGroup(None) => {
                AttributeWidget::new(DnaAttribute::XoverGroup(Some(false)))
            }
            DnaAttribute::XoverGroup(Some(b)) => AttributeWidget::new(if *b {
                DnaAttribute::XoverGroup(None)
            } else {
                DnaAttribute::XoverGroup(Some(true))
            }),
        }
    }

    fn char_repr(&self) -> AttributeDisplay {
        match self {
            DnaAttribute::Visible(b) => AttributeDisplay::Icon(if *b {
                icondata::BsEyeFill
            } else {
                icondata::BsEyeSlash
            }),
            DnaAttribute::XoverGroup(group) => match group {
                None => AttributeDisplay::Text("\u{2205}".to_owned()),
                Some(false) => AttributeDisplay::Text("G".to_owned()),
                Some(true) => AttributeDisplay::Text("R".to_owned()),
            },
            DnaAttribute::LockedForSimulations(b) => AttributeDisplay::Icon(if *b {
                icondata::BsLock
            } else {
                icondata::BsUnlock
            }),
        }
    }
}
