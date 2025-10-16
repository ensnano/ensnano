use std::ops::Range;

use ahash::HashMap;
use ensnano_design::{HelixParameters, Nucl};
use rapier3d::prelude::*;

/// Holds the intermediary representation
/// of a nucleotide pair.
/// Handles the case of free nucleotides;
#[derive(Clone, Debug)]
pub(crate) enum IntermediaryPair {
    OnlyForward(u32, Nucl),
    OnlyBackward(u32, Nucl),
    // always forward, backward
    Pair(u32, Nucl, u32, Nucl),
}

impl IntermediaryPair {
    pub fn is_a_pair(&self) -> bool {
        match self {
            IntermediaryPair::Pair(..) => true,
            _ => false,
        }
    }

    pub fn is_only_forward(&self) -> bool {
        match self {
            IntermediaryPair::OnlyForward(..) => true,
            _ => false,
        }
    }

    pub fn is_only_backward(&self) -> bool {
        match self {
            IntermediaryPair::OnlyBackward(..) => true,
            _ => false,
        }
    }
}

/// Holds the intermediary representation
/// of an helix.
/// Contains a copy of its parameters, a map
/// of pairs, and a vector of ranges of indices
/// that correspond to the continous ranges
/// of double pairs within the helix. This
/// information is computed so that the structure
/// of those continuous segments can be better
/// ensured by the physical simulation.
pub(crate) struct IntermediaryHelix {
    pub parameters: HelixParameters,
    pub helices: HashMap<isize, IntermediaryPair>,
    pub double_ranges: Vec<Range<isize>>,
    pub single_ranges: Vec<Range<isize>>,
}

pub fn build_helices(
    parameters: HelixParameters,
    nucleotide: &HashMap<u32, Nucl>,
) -> HashMap<usize, IntermediaryHelix> {
    let mut result = HashMap::default();

    for (&id, &nucl) in nucleotide {
        let mut helix = result
            .entry(nucl.helix)
            .or_insert(IntermediaryHelix::new(parameters))
            .push_nucleotide(id, nucl);
    }

    result.values_mut().for_each(|helix| helix.compute_ranges());

    result
}

impl IntermediaryHelix {
    pub fn new(parameters: HelixParameters) -> Self {
        Self {
            parameters,
            helices: Default::default(),
            double_ranges: Default::default(),
            single_ranges: Default::default(),
        }
    }

    pub fn compute_ranges(&mut self) {
        self.double_ranges = self.compute_ranges_only(IntermediaryPair::is_a_pair);
        self.single_ranges = self.compute_ranges_only(IntermediaryPair::is_only_forward);
        self.single_ranges
            .extend(self.compute_ranges_only(IntermediaryPair::is_only_backward));

        self.single_ranges.sort_by(|r, s| r.start.cmp(&s.start));
    }

    fn compute_ranges_only<F: Fn(&IntermediaryPair) -> bool>(
        &self,
        predicate: F,
    ) -> Vec<Range<isize>> {
        let mut result = vec![];

        let mut values = self.helices.iter().collect::<Vec<_>>();
        values.sort_unstable_by(|p, q| p.0.cmp(q.0));

        let mut current_range: Option<Range<isize>> = None;

        for (position, pair) in values {
            // we add the position to the current range, or create a new one
            if predicate(pair) {
                if let Some(ref mut range) = current_range {
                    if range.end == *position {
                        range.end += 1;
                    } else {
                        result.push(range.clone());
                        current_range = Some(*position..position + 1);
                    }
                } else {
                    current_range = Some(*position..position + 1);
                }
            } else {
                if let Some(range) = current_range {
                    result.push(range);
                    current_range = None;
                }
            }
        }

        if let Some(range) = current_range {
            result.push(range);
        }

        result
    }

    pub fn push_nucleotide(&mut self, id: u32, nucl: Nucl) -> Option<()> {
        if let Some(pair) = self.helices.get_mut(&nucl.position) {
            match pair {
                IntermediaryPair::OnlyForward(i, n) => {
                    if nucl.forward {
                        return None;
                    }

                    *pair = IntermediaryPair::Pair(*i, *n, id, nucl);
                    return Some(());
                }
                IntermediaryPair::OnlyBackward(i, n) => {
                    if !nucl.forward {
                        return None;
                    }

                    *pair = IntermediaryPair::Pair(id, nucl, *i, *n);
                    return Some(());
                }
                _ => {
                    return None;
                }
            }
        }

        if nucl.forward {
            self.helices
                .insert(nucl.position, IntermediaryPair::OnlyForward(id, nucl));
        } else {
            self.helices
                .insert(nucl.position, IntermediaryPair::OnlyBackward(id, nucl));
        }

        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn helix_ranges() {
        let mut helix = IntermediaryHelix::new(HelixParameters::GEARY_2014_DNA_P_STICK);

        helix.push_nucleotide(
            0,
            Nucl {
                helix: 0,
                position: -6,
                forward: true,
            },
        );
        helix.push_nucleotide(
            0,
            Nucl {
                helix: 0,
                position: -5,
                forward: true,
            },
        );
        helix.push_nucleotide(
            0,
            Nucl {
                helix: 0,
                position: -4,
                forward: false,
            },
        );
        helix.push_nucleotide(
            1,
            Nucl {
                helix: 0,
                position: -2,
                forward: false,
            },
        );
        helix.push_nucleotide(
            2,
            Nucl {
                helix: 0,
                position: -2,
                forward: true,
            },
        );
        helix.push_nucleotide(
            3,
            Nucl {
                helix: 0,
                position: 1,
                forward: false,
            },
        );
        helix.push_nucleotide(
            4,
            Nucl {
                helix: 0,
                position: 1,
                forward: true,
            },
        );
        helix.push_nucleotide(
            5,
            Nucl {
                helix: 0,
                position: 2,
                forward: false,
            },
        );
        helix.push_nucleotide(
            6,
            Nucl {
                helix: 0,
                position: 3,
                forward: false,
            },
        );
        helix.push_nucleotide(
            7,
            Nucl {
                helix: 0,
                position: 3,
                forward: true,
            },
        );
        helix.push_nucleotide(
            8,
            Nucl {
                helix: 0,
                position: 4,
                forward: false,
            },
        );
        helix.push_nucleotide(
            9,
            Nucl {
                helix: 0,
                position: 4,
                forward: true,
            },
        );
        helix.push_nucleotide(
            10,
            Nucl {
                helix: 0,
                position: 5,
                forward: false,
            },
        );
        helix.push_nucleotide(
            11,
            Nucl {
                helix: 0,
                position: 5,
                forward: true,
            },
        );
        helix.push_nucleotide(
            12,
            Nucl {
                helix: 0,
                position: 6,
                forward: true,
            },
        );
        helix.push_nucleotide(
            12,
            Nucl {
                helix: 0,
                position: 7,
                forward: false,
            },
        );
        helix.push_nucleotide(
            12,
            Nucl {
                helix: 0,
                position: 7,
                forward: true,
            },
        );

        helix.compute_ranges();

        assert_eq!(helix.double_ranges, vec![-2..-1, 1..2, 3..6, 7..8]);
        assert_eq!(helix.single_ranges, vec![-6..-4, -4..-3, 2..3, 6..7]);
    }
}
