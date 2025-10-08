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

use {
    ensnano_design::{
        Design, Domain, Helix, HelixInterval, Nucl, Strand,
        grid::{Grid, GridType},
    },
    ensnano_interactor::consts::SCAFFOLD_COLOR,
    serde::{Deserialize, Serialize},
    std::{
        collections::{BTreeMap, HashMap, HashSet},
        sync::Arc,
    },
    std::{fs::File, path::Path},
    ultraviolet::{Rotor3, Vec3},
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(super) struct Cadnano {
    name: String,
    vstrands: Vec<VStrand>,
}

impl Cadnano {
    pub fn from_file<P: AsRef<Path>>(file: P) -> Result<Self, CadnanoError> {
        let f = File::open(file)?;
        Ok(serde_json::from_reader(&f)?)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct VStrand {
    pub col: isize,

    // Position of insertions
    #[serde(rename = "loop")]
    pub loop_: Vec<isize>,
    pub num: isize,
    pub row: isize,
    pub scaf: Vec<(isize, isize, isize, isize)>,
    #[serde(rename = "scafLoop")]
    pub scaf_loop: Vec<isize>,
    pub skip: Vec<isize>,
    // Each element is a corner (helix number, position, helix number, position).
    pub stap: Vec<(isize, isize, isize, isize)>,
    #[serde(rename = "stapLoop")]
    pub stap_loop: Vec<isize>,
    pub stap_colors: Vec<(isize, isize)>,
}

#[derive(Debug)]
pub(super) enum CadnanoError {
    IO(#[allow(unused)] std::io::Error),
    Json(#[allow(unused)] serde_json::Error),
}

impl std::convert::From<std::io::Error> for CadnanoError {
    fn from(e: std::io::Error) -> Self {
        CadnanoError::IO(e)
    }
}

impl std::convert::From<serde_json::Error> for CadnanoError {
    fn from(e: serde_json::Error) -> Self {
        CadnanoError::Json(e)
    }
}

const NO_HELIX: usize = std::usize::MAX;

pub(super) trait FromCadnano: Sized {
    fn from_cadnano(nano: Cadnano) -> Self;
}

impl FromCadnano for Design {
    /// Create a design from a cadnano file
    fn from_cadnano(nano: Cadnano) -> Self {
        let vstrands = nano.vstrands;
        let mut seen: HashSet<(usize, usize, bool)> = HashSet::new();
        let mut design = Design::new();
        let mut nb_strand = 0;
        let mut colors = BTreeMap::new();

        let mut num_to_helix: HashMap<isize, usize> = HashMap::new();

        let mut helices = BTreeMap::new();
        let honeycomb = vstrands[0].scaf.len() % 21 == 0;
        let grid = Grid::new(
            Vec3::zero(),
            Rotor3::identity(),
            Default::default(),
            if honeycomb {
                GridType::honeycomb(None)
            } else {
                GridType::square(None)
            },
        );

        for (i, v) in vstrands.iter().enumerate() {
            num_to_helix.insert(v.num, i);
            let position = grid.position_helix(v.col, v.row);
            let helix = Helix::new(position, Rotor3::identity());
            helices.insert(i, Arc::new(helix));
            for (j, color) in v.stap_colors.iter() {
                colors.insert((i, *j as usize), *color as usize);
            }
        }
        num_to_helix.insert(-1, NO_HELIX);

        for scaf in vec![false, true] {
            for i in 0..vstrands.len() {
                let v = &vstrands[i];
                for j in 0..v.stap.len() {
                    let result = if scaf { v.scaf[j] } else { v.stap[j] };
                    if seen.insert((i, j, scaf)) && result != (-1, -1, -1, -1) {
                        println!("{}, {}, {}", scaf, i, j);
                        let end_5 = find_5_end(i, j, &vstrands, &num_to_helix, scaf);
                        println!("end: {:?}", end_5);
                        let strand =
                            make_strand(end_5, &vstrands, &num_to_helix, &mut seen, scaf, &colors);
                        design.strands.insert(nb_strand, strand);
                        nb_strand += 1;
                    }
                }
            }
        }
        println!("color {:?}", colors);
        design._set_helices(helices);
        design
    }
}

fn find_5_end(
    i: usize,
    j: usize,
    vstrands: &Vec<VStrand>,
    num_to_helix: &HashMap<isize, usize>,
    scaf: bool,
) -> (usize, usize, bool) {
    let (mut a, mut b, mut c, mut d) = (i, j, 0, 0);
    let mut cyclic = false;
    while a != NO_HELIX {
        let result = if scaf {
            vstrands[a].scaf[b]
        } else {
            vstrands[a].stap[b]
        };
        c = a;
        d = b;
        a = num_to_helix[&result.0];
        b = result.1 as usize;

        if a == i && b == j {
            cyclic = true;
            a = NO_HELIX;
        }
    }
    (c, d, cyclic)
}

fn make_strand(
    end_5: (usize, usize, bool),
    vstrands: &Vec<VStrand>,
    num_to_helix: &HashMap<isize, usize>,
    seen: &mut HashSet<(usize, usize, bool)>,
    scaf: bool,
    colors: &BTreeMap<(usize, usize), usize>,
) -> Strand {
    println!("making strand {:?}", end_5);
    let cyclic = end_5.2;
    let (mut i, mut j) = (end_5.0, end_5.1);
    let mut ret = Strand {
        domains: Vec::new(),
        sequence: None,
        junctions: Vec::new(),
        is_cyclic: cyclic,
        color: SCAFFOLD_COLOR,
        name: None,
    };

    let mut insertions = Vec::new();

    let mut current_dom = 0;
    while current_dom == 0 || i != end_5.0 || j != end_5.1 {
        let current_helix = i;
        let current_5 = j;
        let mut current_3 = j;
        let mut once = false;
        let mut insertions_on_dom = Vec::new();
        while i == current_helix && (i != end_5.0 || j != end_5.1 || !once) {
            once = true;
            current_3 = j;
            println!("nucl {}, {}", i, j);
            if let Some(color) = colors.get(&(i, j)).filter(|_| !scaf) {
                ret.color = *color as u32;
            }
            seen.insert((i, j, scaf));
            let result = if scaf {
                vstrands[i].scaf[j]
            } else {
                vstrands[i].stap[j]
            };
            let insertion_size = vstrands[i].loop_[j];
            if vstrands[i].loop_[j] > 0 {
                insertions_on_dom.push((j, insertion_size));
            }
            println!("result {:?}", result);
            i = num_to_helix[&result.2];
            j = result.3 as usize;
        }
        println!("ready to build domain");
        let forward = current_3 >= current_5;
        let start = if forward {
            subtract_skips(current_5, current_helix, vstrands)
        } else {
            subtract_skips(current_3, current_helix, vstrands)
        };
        let end = if forward {
            subtract_skips(current_3, current_helix, vstrands)
        } else {
            subtract_skips(current_5, current_helix, vstrands)
        };
        for (j, n) in insertions_on_dom {
            insertions.push((
                Nucl {
                    helix: current_helix,
                    position: subtract_skips(j, current_helix, vstrands),
                    forward,
                },
                n,
            ));
        }

        println!("pushing {} {} {} {}", current_helix, start, end, forward);
        ret.domains.push(Domain::HelixDomain(HelixInterval {
            helix: current_helix,
            start,
            end: end + 1,
            forward,
            sequence: None,
        }));
        if i == NO_HELIX {
            break;
        }
        current_dom += 1;
    }
    if cyclic {
        if let Domain::HelixDomain(dom0) = &ret.domains[0] {
            if let Domain::HelixDomain(last_dom) = &ret.domains[ret.domains.len() - 1] {
                if last_dom.helix != dom0.helix {
                    let helix = dom0.helix;
                    let start = dom0.start;
                    let end = dom0.start + 1;
                    let forward = dom0.forward;
                    ret.domains.push(Domain::HelixDomain(HelixInterval {
                        helix,
                        start,
                        end,
                        forward,
                        sequence: None,
                    }));
                } else {
                    let len = ret.domains.len();
                    let forward = dom0.forward;
                    let start = dom0.start;
                    let end = dom0.end;
                    if let Domain::HelixDomain(last_dom) = &mut ret.domains[len - 1] {
                        if forward {
                            last_dom.end = start + 1;
                        } else {
                            last_dom.start = end - 1;
                        }
                    }
                }
            }
        }
    }
    for (nucl, n) in insertions.iter() {
        ret.add_insertion_at_nucl(nucl, *n as usize);
    }
    ret
}

fn subtract_skips(nucl: usize, helix: usize, vstrands: &Vec<VStrand>) -> isize {
    let skips: isize = (0..(nucl + 1))
        .map(|n| vstrands[helix].skip[n as usize])
        .sum();
    nucl as isize + skips
}
