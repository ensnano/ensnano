use ahash::HashMap;
use ensnano_design::{
    Nucl,
    elements::DesignElement,
    helices::{Helices, HelixCollection},
};
use ensnano_interactor::ObjectType;
use std::ops::Range;

/// Holds the intermediary representation
/// of a nucleotide pair.
/// Handles the case of free nucleotides;
#[derive(Copy, Clone, Debug)]
pub(crate) enum IntermediaryPair {
    OnlyForward(u32, Nucl),
    OnlyBackward(u32, Nucl),
    // always forward, backward
    // the second nucleotide is redundant
    Pair(u32, Nucl, u32),
}

impl IntermediaryPair {
    pub(crate) fn is_a_pair(&self) -> bool {
        matches!(self, Self::Pair(..))
    }

    pub(crate) fn is_only_forward(&self) -> bool {
        matches!(self, Self::OnlyForward(..))
    }

    pub(crate) fn is_only_backward(&self) -> bool {
        matches!(self, Self::OnlyBackward(..))
    }

    /// Matches two pairs to find the way to link them.
    /// If both are double, or they mismatch, None is returned instead.
    pub(crate) fn match_single(&self, other: &Self) -> Option<(u32, Nucl, u32, Nucl)> {
        match (self, other) {
            (Self::OnlyForward(i, n) | Self::Pair(i, n, _), Self::OnlyForward(j, m))
            | (Self::OnlyBackward(i, n) | Self::Pair(_, n, i), Self::OnlyBackward(j, m))
            | (Self::OnlyForward(i, n), Self::Pair(j, m, _))
            | (Self::OnlyBackward(i, n), Self::Pair(_, m, j)) => Some((*i, *n, *j, *m)),
            _ => None,
        }
    }
}

/// Holds the intermediary representation
/// of an helix.
/// Contains a copy of its parameters, a map
/// of pairs, and a vector of ranges of indices
/// that correspond to the continuous ranges
/// of double pairs within the helix. This
/// information is computed so that the structure
/// of those continuous segments can be better
/// ensured by the physical simulation.
#[derive(Default)]
pub(crate) struct IntermediaryHelix {
    pub pairs: HashMap<isize, IntermediaryPair>,
    pub double_ranges: Vec<Range<isize>>,
    pub single_ranges: Vec<Range<isize>>,
    // a cut at 3 means between levels 2 and 3
    pub crossover_cuts: Vec<isize>,
}

pub(crate) fn build_helices(
    helices: &Helices,
    elements: &Vec<DesignElement>,
    nucleotide: &HashMap<u32, Nucl>,
) -> HashMap<usize, IntermediaryHelix> {
    let mut result = HashMap::<usize, IntermediaryHelix>::default();

    for (&id, &nucl) in nucleotide {
        result
            .entry(nucl.helix)
            .or_default()
            .push_nucleotide(id, nucl);
    }

    // we derive cut points from the bonds
    for element in elements {
        // ObjectType::Bond(b, c) => {
        //     if nucleotide[b].helix == nucleotide[c].helix {
        //         continue;
        //     }
        //     let b = nucleotide[b];
        //     let c = nucleotide[c];

        //     result
        //         .get_mut(&b.helix)
        //         .expect("Nucleotide with incorrect helix")
        //         .crossover_cuts
        //         .extend([b.position, b.position + 1]);
        //     result
        //         .get_mut(&c.helix)
        //         .expect("Nucleotide with incorrect helix")
        //         .crossover_cuts
        //         .extend([c.position, c.position + 1]);
        // }
        // ObjectType::SlicedBond(a, b, c, d) => {
        //     if nucleotide[b].helix == nucleotide[c].helix {
        //         continue;
        //     }
        //     let a = nucleotide[a];
        //     let b = nucleotide[b];
        //     let c = nucleotide[c];
        //     let d = nucleotide[d];

        //     let b_cut = if a.position < b.position {
        //         b.position
        //     } else {
        //         b.position + 1
        //     };

        //     let c_cut = if d.position < c.position {
        //         c.position
        //     } else {
        //         c.position + 1
        //     };

        //     result
        //         .get_mut(&b.helix)
        //         .expect("Nucleotide with incorrect helix")
        //         .crossover_cuts
        //         .push(b_cut);
        //     result
        //         .get_mut(&c.helix)
        //         .expect("Nucleotide with incorrect helix")
        //         .crossover_cuts
        //         .push(c_cut);
        // }
        // ObjectType::Nucleotide(_) => todo!(),
        // ObjectType::HelixCylinder(_, _) => todo!(),
        // ObjectType::ColoredHelixCylinder(_, _, items) => todo!(),
        match element {
            DesignElement::CrossOver {
                helix5prime,
                position5prime,
                helix3prime,
                position3prime,
                ..
            } => {
                if helix5prime == helix3prime {
                    continue;
                }

                result
                    .get_mut(helix5prime)
                    .expect("Nucleotide with incorrect helix")
                    .crossover_cuts
                    .extend([*position5prime, position5prime + 1]);
                result
                    .get_mut(helix3prime)
                    .expect("Nucleotide with incorrect helix")
                    .crossover_cuts
                    .extend([*position3prime, position3prime + 1]);
            }
            _ => {}
        }
    }

    for helix in result.values_mut() {
        helix.compute_ranges();
    }

    for helix in result.values_mut() {
        helix.crossover_cuts.sort_unstable();
    }

    result
}

impl IntermediaryHelix {
    pub(crate) fn compute_ranges(&mut self) {
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

        let mut values = self.pairs.iter().collect::<Vec<_>>();
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
            } else if let Some(range) = current_range {
                result.push(range);
                current_range = None;
            }
        }

        if let Some(range) = current_range {
            result.push(range);
        }

        result
    }

    pub(crate) fn push_nucleotide(&mut self, id: u32, nucl: Nucl) -> Option<()> {
        if let Some(pair) = self.pairs.get_mut(&nucl.position) {
            match pair {
                IntermediaryPair::OnlyForward(i, n) => {
                    if nucl.forward {
                        return None;
                    }

                    *pair = IntermediaryPair::Pair(*i, *n, id);
                    return Some(());
                }
                IntermediaryPair::OnlyBackward(i, _) => {
                    if !nucl.forward {
                        return None;
                    }

                    *pair = IntermediaryPair::Pair(id, nucl, *i);
                    return Some(());
                }
                IntermediaryPair::Pair(..) => {
                    return None;
                }
            }
        }

        if nucl.forward {
            self.pairs
                .insert(nucl.position, IntermediaryPair::OnlyForward(id, nucl));
        } else {
            self.pairs
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
        let mut helix = IntermediaryHelix::default();

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
