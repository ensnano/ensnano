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
use ensnano_organizer::{
    AttributeDisplay, AttributeWidget, ElementKey, Icon, OrganizerAttribute,
    OrganizerAttributeRepr, OrganizerElement,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Clone, Debug)]
pub enum DesignElement {
    GridElement {
        id: usize,
        visible: bool,
    },
    StrandElement {
        id: usize,
        length: usize,
        domain_lengths: Vec<usize>,
    },
    HelixElement {
        id: usize,
        group: Option<bool>,
        visible: bool,
        locked_for_simulations: bool,
    },
    NucleotideElement {
        helix: usize,
        position: isize,
        forward: bool,
    },
    CrossOverElement {
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

impl ToString for DnaAutoGroup {
    fn to_string(&self) -> String {
        match self {
            Self::StrandWithLength(length) => format!("Strand with length {}", length.to_string()),
            Self::StrandWithDomainOfLength(length) => {
                format!("Strand with a domain of length {}", length.to_string())
            }
        }
    }
}

const LONG: usize = 100;
const SHORT: usize = 4;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum BoundedLength {
    Short,
    Between(usize),
    Long,
}

impl From<usize> for BoundedLength {
    fn from(n: usize) -> Self {
        if n > LONG {
            Self::Long
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
            Self::Long => format!("> {LONG}"),
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
            DesignElement::GridElement { id, .. } => DesignElementKey::Grid(*id),
            DesignElement::StrandElement { id, .. } => DesignElementKey::Strand(*id),
            DesignElement::HelixElement { id, .. } => DesignElementKey::Helix(*id),
            DesignElement::NucleotideElement {
                helix,
                position,
                forward,
            } => DesignElementKey::Nucleotide {
                helix: *helix,
                position: *position,
                forward: *forward,
            },
            DesignElement::CrossOverElement { xover_id, .. } => DesignElementKey::CrossOver {
                xover_id: *xover_id,
            },
        }
    }

    fn display_name(&self) -> String {
        match self {
            DesignElement::GridElement { id, .. } => format!("Grid {}", id),
            DesignElement::StrandElement { id, .. } => format!("Strand {}", id),
            DesignElement::HelixElement { id, .. } => format!("Helix {}", id),
            DesignElement::NucleotideElement {
                helix,
                position,
                forward,
            } => format!("Nucl {}:{}:{}", helix, position, forward),
            DesignElement::CrossOverElement {
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
            DesignElement::HelixElement {
                group,
                locked_for_simulations: locked,
                ..
            } => vec![
                DnaAttribute::XoverGroup(*group),
                DnaAttribute::LockedForSimulations(*locked),
            ],
            DesignElement::GridElement { visible, .. } => vec![DnaAttribute::Visible(*visible)],
            _ => vec![],
        }
    }

    fn auto_groups(&self) -> Vec<Self::AutoGroup> {
        match self {
            DesignElement::StrandElement {
                length,
                domain_lengths,
                ..
            } => {
                let mut ret = vec![DnaAutoGroup::StrandWithLength((*length).into())];
                let mut lengths = domain_lengths.clone();
                lengths.sort();
                lengths.dedup();
                for len in lengths {
                    ret.push(DnaAutoGroup::StrandWithDomainOfLength((len).into()))
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
pub enum DnaAttributeRepr {
    Visible,
    XoverGroup,
    LockedForSimulations,
}

const ALL_DNA_ATTRIBUTE_REPR: [DnaAttributeRepr; 3] = [
    DnaAttributeRepr::Visible,
    DnaAttributeRepr::XoverGroup,
    DnaAttributeRepr::LockedForSimulations,
];

impl OrganizerAttributeRepr for DnaAttributeRepr {
    fn all_repr() -> &'static [Self] {
        &ALL_DNA_ATTRIBUTE_REPR
    }
}

impl OrganizerAttribute for DnaAttribute {
    type Repr = DnaAttributeRepr;

    fn repr(&self) -> DnaAttributeRepr {
        match self {
            DnaAttribute::Visible(_) => DnaAttributeRepr::Visible,
            DnaAttribute::XoverGroup(_) => DnaAttributeRepr::XoverGroup,
            DnaAttribute::LockedForSimulations(_) => DnaAttributeRepr::LockedForSimulations,
        }
    }

    fn widget(&self) -> AttributeWidget<DnaAttribute> {
        match self {
            DnaAttribute::Visible(b) => AttributeWidget::FlipButton {
                value_if_pressed: DnaAttribute::Visible(!b),
            },
            DnaAttribute::LockedForSimulations(b) => AttributeWidget::FlipButton {
                value_if_pressed: DnaAttribute::LockedForSimulations(!b),
            },
            DnaAttribute::XoverGroup(None) => AttributeWidget::FlipButton {
                value_if_pressed: DnaAttribute::XoverGroup(Some(false)),
            },
            DnaAttribute::XoverGroup(Some(b)) => AttributeWidget::FlipButton {
                value_if_pressed: if *b {
                    DnaAttribute::XoverGroup(None)
                } else {
                    DnaAttribute::XoverGroup(Some(true))
                },
            },
        }
    }

    fn char_repr(&self) -> AttributeDisplay {
        match self {
            DnaAttribute::Visible(b) => {
                let c = if *b {
                    Icon::EyeFill.into()
                } else {
                    Icon::EyeSlash.into()
                };
                AttributeDisplay::Icon(c)
            }
            DnaAttribute::XoverGroup(group) => match group {
                None => AttributeDisplay::Text("\u{2205}".to_owned()),
                Some(false) => AttributeDisplay::Text("G".to_owned()),
                Some(true) => AttributeDisplay::Text("R".to_owned()),
            },
            DnaAttribute::LockedForSimulations(b) => {
                let c = if *b {
                    Icon::Lock.into()
                } else {
                    Icon::Unlock.into()
                };
                AttributeDisplay::Icon(c)
            }
        }
    }
}

impl std::fmt::Display for DnaAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.char_repr() {
                AttributeDisplay::Icon(c) => format!("{}", c),
                AttributeDisplay::Text(s) => s,
            }
        )
    }
}
