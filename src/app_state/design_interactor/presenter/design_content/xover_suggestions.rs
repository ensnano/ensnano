use ahash::RandomState;
use ensnano_design::{Design, Nucl};
use ensnano_interactor::app_state_parameters::suggestion_parameters::SuggestionParameters;
use std::collections::{BTreeMap, HashMap, HashSet};
use ultraviolet::Vec3;

type CubeMap = HashMap<(isize, isize, isize), Vec<Nucl>, RandomState>;

const LEN_CRIT: f32 = 1.2;

#[derive(Default, Debug, Clone)]
pub(super) struct XoverSuggestions {
    helices_groups: BTreeMap<usize, Vec<Nucl>>,
    helices_cubes: BTreeMap<usize, CubeMap>,
    blue_nucl: Vec<Nucl>,
    red_cubes: CubeMap,
}

impl XoverSuggestions {
    pub(super) fn add_nucl(&mut self, nucl: Nucl, space_pos: Vec3, groups: &BTreeMap<usize, bool>) {
        let cube = space_to_cube(space_pos.x, space_pos.y, space_pos.z);

        self.helices_groups
            .entry(nucl.helix)
            .or_default()
            .push(nucl);
        self.helices_cubes
            .entry(nucl.helix)
            .or_default()
            .entry(cube)
            .or_default()
            .push(nucl);

        match groups.get(&nucl.helix) {
            Some(true) => {
                self.blue_nucl.push(nucl);
            }
            Some(false) => {
                self.red_cubes.entry(cube).or_default().push(nucl);
            }
            None => (),
        }
    }

    /// Return the list of all suggested crossovers
    pub(super) fn get_suggestions(
        &self,
        design: &Design,
        suggestion_parameters: &SuggestionParameters,
    ) -> Vec<(Nucl, Nucl)> {
        let mut ret = vec![];
        if suggestion_parameters.ignore_groups {
            self.get_suggestions_all_helices(&mut ret, design, suggestion_parameters);
        } else {
            self.get_suggestions_groups(&mut ret, design, suggestion_parameters);
        }
        ret.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
        self.trim_suggestion(&ret, design, suggestion_parameters)
    }

    /// Return the list of all suggested crossovers
    fn get_suggestions_groups(
        &self,
        ret: &mut Vec<(Nucl, Nucl, f32)>,
        design: &Design,
        suggestion_parameters: &SuggestionParameters,
    ) {
        for blue_nucl in &self.blue_nucl {
            let neighbor = self
                .get_possible_cross_over_groups(design, blue_nucl, suggestion_parameters)
                .unwrap_or_default();
            for (red_nucl, dist) in neighbor {
                ret.push((*blue_nucl, red_nucl, dist));
            }
        }
    }

    /// Trim a list of crossovers so that each nucleotide appears at most once in the suggestion
    /// list.
    fn trim_suggestion(
        &self,
        suggestion: &[(Nucl, Nucl, f32)],
        design: &Design,
        suggestion_parameters: &SuggestionParameters,
    ) -> Vec<(Nucl, Nucl)> {
        let mut used = HashSet::new();
        let mut ret = vec![];
        for (a, b, _) in suggestion {
            if !used.contains(a) && !used.contains(b) {
                let a_end = design.strands.is_strand_end(a).to_opt();
                let b_end = design.strands.is_strand_end(b).to_opt();
                if !matches!(a_end.zip(b_end), Some((a, b)) if a == b)
                    && (suggestion_parameters.include_xover_ends
                        || (!design.strands.is_true_xover_end(a)
                            && !design.strands.is_true_xover_end(b)))
                {
                    ret.push((*a, *b));
                    used.insert(a);
                    used.insert(b);
                }
            }
        }
        ret
    }

    fn get_suggestions_all_helices(
        &self,
        ret: &mut Vec<(Nucl, Nucl, f32)>,
        design: &Design,
        suggestion_parameters: &SuggestionParameters,
    ) {
        for nucls in self.helices_groups.values() {
            for n in nucls {
                let neighbor = self
                    .get_possible_cross_over_all_helices(design, n, suggestion_parameters)
                    .unwrap_or_default();
                for (red_nucl, dist) in neighbor {
                    ret.push((*n, red_nucl, dist));
                }
            }
        }
    }

    fn get_possible_cross_over_all_helices(
        &self,
        design: &Design,
        nucl: &Nucl,
        suggestion_parameters: &SuggestionParameters,
    ) -> Option<Vec<(Nucl, f32)>> {
        let mut ret = Vec::new();
        let positions = design.get_nucl_position(*nucl)?;
        let cube0 = space_to_cube(positions[0], positions[1], positions[2]);
        for i in &[-1, 0, 1] {
            for j in &[-1, 0, 1] {
                for k in &[-1, 0, 1] {
                    let cube = (cube0.0 + i, cube0.1 + j, cube0.2 + k);

                    for (_, cubes) in self.helices_cubes.iter().filter(|(h, _)| **h > nucl.helix) {
                        if let Some(v) = cubes.get(&cube) {
                            for red_nucl in v {
                                if red_nucl.helix != nucl.helix
                                    && let Some(red_position) = design.get_nucl_position(*red_nucl)
                                {
                                    let dist = (0..3)
                                        .map(|i| (positions[i], red_position[i]))
                                        .map(|(x, y)| (x - y) * (x - y))
                                        .sum::<f32>()
                                        .sqrt();
                                    if dist < LEN_CRIT
                                        && (suggestion_parameters.include_scaffold
                                            || design.strands.get_strand_nucl(nucl)
                                                != design.scaffold_id)
                                        && (suggestion_parameters.include_scaffold
                                            || design.strands.get_strand_nucl(red_nucl)
                                                != design.scaffold_id)
                                        && (suggestion_parameters.include_intra_strand
                                            || design.strands.get_strand_nucl(nucl)
                                                != design.strands.get_strand_nucl(red_nucl))
                                    {
                                        ret.push((*red_nucl, dist));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(ret)
    }

    /// Return all the crossovers of length less than `len_crit` involving `nucl`, and their length.
    fn get_possible_cross_over_groups(
        &self,
        design: &Design,
        nucl: &Nucl,
        suggestion_parameters: &SuggestionParameters,
    ) -> Option<Vec<(Nucl, f32)>> {
        let mut ret = Vec::new();
        let positions = design.get_nucl_position(*nucl)?;
        let cube0 = space_to_cube(positions[0], positions[1], positions[2]);

        for i in [-1, 0, 1] {
            for j in [-1, 0, 1] {
                for k in [-1, 0, 1] {
                    let cube = (cube0.0 + i, cube0.1 + j, cube0.2 + k);

                    if let Some(v) = self.red_cubes.get(&cube) {
                        for red_nucl in v {
                            if red_nucl.helix != nucl.helix
                                && let Some(red_position) = design.get_nucl_position(*red_nucl)
                            {
                                let dist = (0..3)
                                    .map(|i| (positions[i], red_position[i]))
                                    .map(|(x, y)| (x - y) * (x - y))
                                    .sum::<f32>()
                                    .sqrt();
                                if dist < LEN_CRIT
                                    && (suggestion_parameters.include_scaffold
                                        || design.strands.get_strand_nucl(nucl)
                                            != design.scaffold_id)
                                    && (suggestion_parameters.include_scaffold
                                        || design.strands.get_strand_nucl(red_nucl)
                                            != design.scaffold_id)
                                    && (suggestion_parameters.include_intra_strand
                                        || design.strands.get_strand_nucl(nucl)
                                            != design.strands.get_strand_nucl(red_nucl))
                                {
                                    ret.push((*red_nucl, dist));
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(ret)
    }
}

fn space_to_cube(x: f32, y: f32, z: f32) -> (isize, isize, isize) {
    let cube_len = 1.2;
    (
        x.div_euclid(cube_len) as isize,
        y.div_euclid(cube_len) as isize,
        z.div_euclid(cube_len) as isize,
    )
}
