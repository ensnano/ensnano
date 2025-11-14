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

use crate::ensnano_design::{Axis, Design, Domain, Nucl, OwnedAxis};
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct StrandBuilder {
    /// The nucleotide that can move
    pub moving_end: Nucl,
    /// The initial position of the moving end
    pub initial_position: isize,
    /// Axis of the support helix on which the domain lies
    pub axis: OwnedAxis,
    /// The identifier of the domain being edited
    identifier: DomainIdentifier,
    /// The fixed_end of the domain being edited, `None` if the domain is new and can go in both
    /// direction
    fixed_end: Option<isize>,
    /// The eventual other strand being modified by the current modification
    neighbor_strand: Option<NeighborDescriptor>,
    /// The direction in which the end of neighbor_strand can go, starting from its initial
    /// position
    neighbor_direction: Option<EditDirection>,
    /// The minimum position to which the edited domain can go. It corresponds to the eventual
    /// minimum position of the neighbor_strand or to the other end of the domain being edited
    min_pos: Option<isize>,
    /// The maximum position to which the edited domain can go. It corresponds to the eventual
    /// maximum position of the neighbor_strand, or to the other end of the domain being edited
    max_pos: Option<isize>,
    /// An eventual neighbor that was detached during the movement
    detached_neighbor: Option<NeighborDescriptor>,
}

impl StrandBuilder {
    /// Create a strand that will build a new strand. This means that the initial position
    /// corresponds to a phantom nucleotide
    /// # Argument
    ///
    /// * identifier: The identifier of the domain that will be created
    ///
    /// * nucl: The fixed end of the domain that will be created
    ///
    /// * axis: The axis of the helix on which the domain will be created
    ///
    /// * neighbor: An eventual existing neighbor of the strand being created
    pub fn init_empty(
        identifier: DomainIdentifier,
        nucl: Nucl,
        axis: OwnedAxis,
        neighbor: Option<NeighborDescriptor>,
    ) -> Self {
        let mut neighbor_strand = None;
        let mut neighbor_direction = None;
        let mut min_pos = None;
        let mut max_pos = None;
        if let Some(desc) = neighbor {
            neighbor_strand = Some(desc);
            neighbor_direction = if desc.initial_moving_end < nucl.position {
                min_pos = Some(desc.fixed_end + 1);
                Some(EditDirection::Negative)
            } else {
                max_pos = Some(desc.fixed_end - 1);
                Some(EditDirection::Positive)
            };
        }

        Self {
            initial_position: nucl.position,
            moving_end: nucl,
            identifier,
            axis,
            fixed_end: None,
            neighbor_strand,
            neighbor_direction,
            min_pos,
            max_pos,
            detached_neighbor: None,
        }
    }

    /// Create a strand that will edit an existing domain. This means that the initial position
    /// corresponds to an end of an existing domain
    /// # Argument
    ///
    /// * identifier: The identifier of the domain that will be created
    ///
    /// * nucl: The moving end of the domain that will be created
    ///
    /// * axis: The axis of the helix on which the domain will be created
    ///
    /// * other_end: The position of the fixed end of the domain that will be edited
    ///
    /// * neighbor: An eventual existing neighbor of the strand being created
    pub fn init_existing(
        identifier: DomainIdentifier,
        nucl: Nucl,
        axis: OwnedAxis,
        other_end: Option<isize>,
        neighbor: Option<NeighborDescriptor>,
        stick: bool,
    ) -> Self {
        let mut min_pos = None;
        let mut max_pos = None;
        let initial_position = nucl.position;
        if let Some(other_end) = other_end {
            if initial_position < other_end {
                max_pos = Some(other_end);
            } else {
                min_pos = Some(other_end);
            }
        }
        let neighbor_strand;
        let neighbor_direction;
        if let Some(desc) = neighbor {
            neighbor_strand = Some(desc);
            neighbor_direction = if stick {
                Some(EditDirection::Both)
            } else if desc.moving_end > initial_position {
                Some(EditDirection::Positive)
            } else {
                Some(EditDirection::Negative)
            };
            if desc.initial_moving_end > initial_position {
                max_pos = max_pos.or(Some(desc.fixed_end - 1));
            } else {
                min_pos = min_pos.or(Some(desc.fixed_end + 1));
            }
        } else {
            neighbor_strand = None;
            neighbor_direction = None;
        }
        let ret = Self {
            moving_end: nucl,
            initial_position,
            axis,
            identifier,
            fixed_end: other_end,
            neighbor_strand,
            neighbor_direction,
            max_pos,
            min_pos,
            detached_neighbor: None,
        };
        log::info!("builder {:?}", ret);
        ret
    }

    /// Detach the neighbor, this function must be called when the moving end goes to a position
    /// where the moving end of the neighbor cannot follow it.
    fn detach_neighbor(&mut self) {
        self.neighbor_direction = None;
        self.detached_neighbor = self.neighbor_strand.take();
    }

    /// Attach a new neighbor. This function must be called when the moving end goes to a position
    /// where it is next to an existing domain
    fn attach_neighbor(&mut self, descriptor: &NeighborDescriptor) -> bool {
        // To prevent attaching to self or attaching to the same neighbor or attaching to a
        // neighbor in the wrong direction
        if self.identifier.is_same_domain_than(&descriptor.identifier)
            || self.neighbor_strand.is_some()
            || descriptor.moving_end > self.max_pos.unwrap_or(descriptor.moving_end)
            || descriptor.moving_end < self.min_pos.unwrap_or(descriptor.moving_end)
        {
            return false;
        }
        self.neighbor_direction = if self.moving_end.position < descriptor.initial_moving_end {
            Some(EditDirection::Positive)
        } else {
            Some(EditDirection::Negative)
        };
        self.neighbor_strand = Some(*descriptor);
        true
    }

    /// Increase the position of the moving end by one, and update the neighbor in consequences.
    fn incr_position(&mut self, design: &Design, ignored_domains: &[DomainIdentifier]) {
        // Eventually detach from neighbor
        if let Some(desc) = self.neighbor_strand.as_mut() {
            if desc.initial_moving_end == self.moving_end.position - 1
                && self.neighbor_direction == Some(EditDirection::Negative)
            {
                self.detach_neighbor();
            } else {
                desc.moving_end += 1;
            }
        }
        self.moving_end.position += 1;
        let desc = design
            .get_neighbor_nucl(self.moving_end.right())
            .filter(|neighbor| {
                !ignored_domains
                    .iter()
                    .any(|d| d.is_same_domain_than(&neighbor.identifier))
            })
            .filter(|neighbor| neighbor.identifier.start != self.identifier.start);
        if let Some(ref desc) = desc
            && self.attach_neighbor(desc)
        {
            self.max_pos = self.max_pos.or(Some(desc.fixed_end - 1));
        }
    }

    /// Decrease the position of the moving end by one, and update the neighbor in consequences.
    fn decr_position(&mut self, design: &Design, ignored_domains: &[DomainIdentifier]) {
        // Update neighbor and eventually detach from it
        if let Some(desc) = self.neighbor_strand.as_mut() {
            if desc.initial_moving_end == self.moving_end.position + 1
                && self.neighbor_direction == Some(EditDirection::Positive)
            {
                self.detach_neighbor();
            } else {
                desc.moving_end -= 1;
            }
        }
        self.moving_end.position -= 1;
        let desc = design
            .get_neighbor_nucl(self.moving_end.left())
            .filter(|neighbor| {
                !ignored_domains
                    .iter()
                    .any(|d| d.is_same_domain_than(&neighbor.identifier))
            })
            .filter(|neighbor| neighbor.identifier.start != self.identifier.start);
        if let Some(ref desc) = desc
            && self.attach_neighbor(desc)
        {
            self.min_pos = self.min_pos.or(Some(desc.fixed_end + 1));
        }
    }

    /// Move the moving end to an objective position. If this position cannot be reached by the
    /// moving end, it will go as far as possible.
    pub fn move_to(
        &mut self,
        objective: isize,
        design: &mut Design,
        ignored_domains: &[DomainIdentifier],
    ) {
        log::info!("self {:?}", self);
        log::info!("move to {}", objective);
        let mut need_update = true;
        match objective.cmp(&self.moving_end.position) {
            Ordering::Greater => {
                while self.moving_end.position < objective.min(self.max_pos.unwrap_or(objective)) {
                    self.incr_position(design, ignored_domains);
                    need_update = true;
                }
            }
            Ordering::Less => {
                while self.moving_end.position > objective.max(self.min_pos.unwrap_or(objective)) {
                    self.decr_position(design, ignored_domains);
                    need_update = true;
                }
            }
            _ => (),
        }
        if need_update {
            self.update(design);
        }
    }

    pub fn try_incr(&mut self, design: &Design, ignored_domains: &[DomainIdentifier]) -> bool {
        if self.moving_end.position < self.max_pos.unwrap_or(isize::MAX) {
            self.incr_position(design, ignored_domains);
            true
        } else {
            false
        }
    }

    pub fn try_decr(&mut self, design: &Design, ignored_domains: &[DomainIdentifier]) -> bool {
        if self.moving_end.position > self.min_pos.unwrap_or(isize::MIN) {
            self.decr_position(design, ignored_domains);
            true
        } else {
            false
        }
    }

    /// Apply the modification on the data
    pub fn update(&mut self, design: &mut Design) {
        Self::update_strand(
            design,
            self.identifier,
            self.moving_end.position,
            self.fixed_end.unwrap_or(self.initial_position),
        );
        if let Some(desc) = self.neighbor_strand {
            Self::update_strand(design, desc.identifier, desc.moving_end, desc.fixed_end);
        }
        if let Some(desc) = self.detached_neighbor.take() {
            Self::update_strand(design, desc.identifier, desc.moving_end, desc.fixed_end);
        }
    }

    fn update_strand(
        design: &mut Design,
        identifier: DomainIdentifier,
        position: isize,
        fixed_position: isize,
    ) {
        log::info!(
            "updating {:?}, position {}, fixed_position {}",
            identifier,
            position,
            fixed_position
        );
        let domain =
            &mut design.strands.get_mut(&identifier.strand).unwrap().domains[identifier.domain];
        if let Domain::HelixDomain(domain) = domain {
            match identifier.start {
                None => {
                    let start = position.min(fixed_position);
                    let end = position.max(fixed_position) + 1;
                    domain.start = start;
                    domain.end = end;
                }
                Some(false) => {
                    domain.end = position + 1;
                }
                Some(true) => {
                    domain.start = position;
                }
            }
        }
    }

    pub fn get_axis(&self) -> Axis<'_> {
        self.axis.borrow()
    }

    pub fn get_strand_id(&self) -> usize {
        self.identifier.strand
    }

    pub fn get_moving_end_position(&self) -> isize {
        self.moving_end.position
    }

    pub fn get_domain_identifier(&self) -> DomainIdentifier {
        self.identifier
    }
}

/// The direction in which a moving end can go
#[derive(Debug, Clone, Copy, PartialEq)]
enum EditDirection {
    /// In both direction
    Both,
    /// Only on position smaller that the initial position
    Negative,
    /// Only on position bigger that the initial position
    Positive,
}

/// Describes a domain being edited
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeighborDescriptor {
    pub identifier: DomainIdentifier,
    pub initial_moving_end: isize,
    pub moving_end: isize,
    pub fixed_end: isize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DomainIdentifier {
    pub strand: usize,
    pub domain: usize,
    pub start: Option<bool>,
}

impl DomainIdentifier {
    pub fn other_end(&self) -> Option<Self> {
        self.start.map(|end| Self {
            strand: self.strand,
            domain: self.domain,
            start: Some(!end),
        })
    }

    pub fn is_same_domain_than(&self, other: &Self) -> bool {
        self.strand == other.strand && self.domain == other.domain
    }
}

pub trait NeighborDescriptorGiver {
    fn get_neighbor_nucl(&self, nucl: Nucl) -> Option<NeighborDescriptor>;
}

impl NeighborDescriptorGiver for Design {
    fn get_neighbor_nucl(&self, nucl: Nucl) -> Option<NeighborDescriptor> {
        for (s_id, s) in self.strands.iter() {
            for (d_id, d) in s.domains.iter().enumerate() {
                if let Some(other) = d.other_end(nucl) {
                    let start = if let Domain::HelixDomain(i) = d {
                        // if the domain has length one, we are not at a specific end
                        (d.length() > 1).then_some(i.start)
                    } else {
                        None
                    };
                    return Some(NeighborDescriptor {
                        identifier: DomainIdentifier {
                            strand: *s_id,
                            domain: d_id,
                            start: start.map(|s| s == nucl.position),
                        },
                        fixed_end: other,
                        initial_moving_end: nucl.position,
                        moving_end: nucl.position,
                    });
                }
            }
        }
        None
    }
}
