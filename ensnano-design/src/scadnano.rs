use crate::{
    Design,
    domains::{Domain, helix_interval::HelixInterval, sanitize_domains},
    ensnano_version,
    grid::{GridDescriptor, GridId, GridTypeDescr, HelixGridPosition, grid_collection::FreeGrids},
    helices::{Helices, Helix},
    parameters::HelixParameters,
    strands::{Strand, Strands, read_junctions},
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
};
use ultraviolet::{Isometry2, Rotor2, Rotor3, Vec2, Vec3};

#[derive(Serialize, Deserialize)]
pub struct ScadnanoDesign {
    pub version: String,
    #[serde(default = "default_grid")]
    pub grid: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub groups: Option<HashMap<String, ScadnanoGroup>>,
    pub helices: Vec<ScadnanoHelix>,
    pub strands: Vec<ScadnanoStrand>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub modifications_in_design: Option<HashMap<String, ScadnanoModification>>,
}

fn default_grid() -> String {
    String::from("square")
}

impl ScadnanoDesign {
    pub fn default_grid_descriptor(&self) -> Result<GridDescriptor, ScadnanoImportError> {
        let grid_type = match self.grid.as_str() {
            "square" => Ok(GridTypeDescr::Square { twist: None }),
            "honeycomb" => Ok(GridTypeDescr::Honeycomb { twist: None }),
            grid_type => {
                println!("Unsupported grid type: {grid_type}");
                Err(ScadnanoImportError::UnsupportedGridType(
                    grid_type.to_owned(),
                ))
            }
        }?;
        Ok(GridDescriptor {
            position: Vec3::zero(),
            orientation: Rotor3::identity(),
            helix_parameters: Some(HelixParameters::GEARY_2014_DNA),
            grid_type,
            invisible: false,
            bezier_vertex: None,
        })
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct ScadnanoGroup {
    pub position: Vec3,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub pitch: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    yaw: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    roll: Option<f32>,
    grid: String,
}

impl ScadnanoGroup {
    pub fn to_grid_desc(&self) -> Result<GridDescriptor, ScadnanoImportError> {
        let grid_type = match self.grid.as_str() {
            "square" => Ok(GridTypeDescr::Square { twist: None }),
            "honeycomb" => Ok(GridTypeDescr::Honeycomb { twist: None }),
            grid_type => {
                println!("Unsupported grid type: {grid_type}");
                Err(ScadnanoImportError::UnsupportedGridType(
                    grid_type.to_owned(),
                ))
            }
        }?;
        let orientation = Rotor3::from_euler_angles(
            self.roll.unwrap_or_default().to_radians(),
            self.pitch.unwrap_or_default().to_radians(),
            self.yaw.unwrap_or_default().to_radians(),
        );
        Ok(GridDescriptor {
            grid_type,
            orientation,
            helix_parameters: Some(HelixParameters::GEARY_2014_DNA),
            position: self.position,
            invisible: false,
            bezier_vertex: None,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct ScadnanoHelix {
    #[serde(default)]
    pub max_offset: usize,
    pub grid_position: Vec<isize>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub group: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ScadnanoStrand {
    #[serde(default)]
    pub is_scaffold: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sequence: Option<String>,
    pub color: String,
    pub domains: Vec<ScadnanoDomain>,
    #[serde(
        rename = "5prime_modification",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub prime5_modification: Option<String>,
    #[serde(
        rename = "3prime_modification",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub prime3_modification: Option<String>,
    #[serde(default)]
    pub circular: bool,
}

impl ScadnanoStrand {
    pub fn color(&self) -> Result<u32, ScadnanoImportError> {
        let color_str = &self.color[1..];
        let ret = u32::from_str_radix(color_str, 16);
        if let Ok(ret) = ret {
            Ok(ret)
        } else {
            Err(ScadnanoImportError::InvalidColor(color_str.to_owned()))
        }
    }

    pub fn read_deletions(&self, deletions: &mut BTreeMap<usize, BTreeSet<isize>>) {
        for d in &self.domains {
            d.read_deletions(deletions);
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScadnanoDomain {
    Loopout {
        loopout: usize,
    },
    HelixDomain {
        helix: usize,
        start: isize,
        end: isize,
        forward: bool,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        insertions: Option<Vec<Vec<isize>>>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        deletions: Option<Vec<isize>>,
    },
}

impl ScadnanoDomain {
    fn read_deletions(&self, deletions_map: &mut BTreeMap<usize, BTreeSet<isize>>) {
        match self {
            Self::Loopout { .. } => (),
            Self::HelixDomain {
                deletions, helix, ..
            } => {
                if let Some(vec) = deletions {
                    let entry = deletions_map.entry(*helix).or_default();
                    for d in vec {
                        entry.insert(*d);
                    }
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ScadnanoModification {
    pub display_text: String,
    pub idt_text: String,
    pub location: String,
}

#[derive(Debug)]
pub enum ScadnanoImportError {
    UnsupportedGridType(String),
    InvalidColor(String),
    MissingField(String),
}

#[derive(Default)]
pub(crate) struct ScadnanoInsertionsDeletions {
    count: BTreeMap<usize, BTreeMap<isize, isize>>,
}

impl ScadnanoInsertionsDeletions {
    pub(crate) fn read_domain(&mut self, domain: &ScadnanoDomain) {
        match domain {
            ScadnanoDomain::Loopout { .. } => (),
            ScadnanoDomain::HelixDomain {
                deletions,
                helix,
                insertions,
                ..
            } => {
                if let Some(vec) = deletions {
                    let entry = self.count.entry(*helix).or_default();
                    for d in vec {
                        let count_entry = entry.entry(*d).or_default();
                        *count_entry -= 1;
                    }
                }
                if let Some(vec) = insertions {
                    let entry = self.count.entry(*helix).or_default();
                    for insertion in vec {
                        let position = insertion[0];
                        let count = insertion[1];
                        let count_entry = entry.entry(position).or_default();
                        *count_entry += count;
                    }
                }
            }
        }
    }

    pub(crate) fn adjust(&self, position: isize, helix: usize) -> isize {
        let mut ret = position;
        if let Some(counts) = self.count.get(&helix) {
            for (_, c) in counts.iter().take_while(|(y, _)| **y <= position) {
                ret += *c / 2;
            }
        }
        ret
    }
}

impl Domain {
    fn from_scadnano(
        scad: &ScadnanoDomain,
        insertion_deletions: &ScadnanoInsertionsDeletions,
    ) -> Vec<Self> {
        match scad {
            ScadnanoDomain::HelixDomain {
                helix,
                start,
                end,
                forward,
                ..// TODO read insertion and deletion
            } => {
                let start = insertion_deletions.adjust(*start, *helix);
                let end = insertion_deletions.adjust(*end, *helix);

                vec![Self::HelixDomain(HelixInterval {
                    helix: *helix,
                    start,
                    end,
                    forward: *forward,
                    sequence: None,
                })]
            }
            ScadnanoDomain::Loopout{ loopout: n } => vec![Self::new_insertion(*n)]
        }
    }
}

impl Strand {
    fn from_scadnano(
        scad: &ScadnanoStrand,
        insertion_deletions: &ScadnanoInsertionsDeletions,
    ) -> Result<Self, ScadnanoImportError> {
        let color = scad.color()?;
        let domains: Vec<Domain> = scad
            .domains
            .iter()
            .flat_map(|s| Domain::from_scadnano(s, insertion_deletions))
            .collect();
        let sequence = scad.sequence.as_ref().map(|seq| Cow::Owned(seq.clone()));
        let cyclic = scad.circular;
        let sane_domains = sanitize_domains(&domains, cyclic);
        let junctions = read_junctions(&sane_domains, cyclic);
        Ok(Self {
            domains: sane_domains,
            color,
            is_cyclic: cyclic,
            junctions,
            sequence,
            ..Default::default()
        })
    }
}

impl Helix {
    fn from_scadnano(
        scad: &ScadnanoHelix,
        group_map: &BTreeMap<String, usize>,
        groups: &[ScadnanoGroup],
        helix_per_group: &mut Vec<usize>,
    ) -> Result<Self, ScadnanoImportError> {
        let group_id = scad
            .group
            .clone()
            .unwrap_or_else(|| String::from("default_group"));
        let Some(grid_id) = group_map.get(&group_id) else {
            return Err(ScadnanoImportError::MissingField(format!(
                "group {group_id}",
            )));
        };
        let Some(x) = scad.grid_position.first().copied() else {
            return Err(ScadnanoImportError::MissingField(String::from("x")));
        };
        let Some(y) = scad.grid_position.get(1).copied() else {
            return Err(ScadnanoImportError::MissingField(String::from("y")));
        };
        let Some(group) = groups.get(*grid_id) else {
            return Err(ScadnanoImportError::MissingField(format!(
                "group {grid_id}",
            )));
        };

        println!("helices per group {group_map:?}");
        println!("helices per group {helix_per_group:?}");
        let Some(nb_helices) = helix_per_group.get_mut(*grid_id) else {
            return Err(ScadnanoImportError::MissingField(format!(
                "helix_per_group {grid_id}",
            )));
        };
        let rotation = Rotor2::from_angle(group.pitch.unwrap_or_default().to_radians());
        let isometry2d = Isometry2 {
            translation: (5. * *nb_helices as f32 - 1.) * Vec2::unit_y().rotated_by(rotation)
                + 5. * Vec2::new(group.position.x, group.position.y),
            rotation,
        };
        *nb_helices += 1;

        Ok(Self {
            position: Vec3::zero(),
            orientation: Rotor3::identity(),
            helix_parameters: None,
            grid_position: Some(HelixGridPosition {
                grid: GridId::FreeGrid(*grid_id),
                x,
                y,
                axis_pos: 0,
                roll: 0f32,
            }),
            visible: true,
            roll: 0f32,
            isometry2d: Some(isometry2d),
            additional_isometries: Vec::new(),
            symmetry: Vec2::one(),
            locked_for_simulations: false,
            curve: None,
            instantiated_curve: None,
            instantiated_descriptor: None,
            delta_bases_per_turn: 0.,
            initial_nt_index: 0,
            support_helix: None,
            path_id: None,
        })
    }
}

impl Design {
    pub fn from_scadnano(scad: &ScadnanoDesign) -> Result<Self, ScadnanoImportError> {
        let mut grids = Vec::new();
        let mut group_map = BTreeMap::new();
        let default_grid = scad.default_grid_descriptor()?;
        let mut insertion_deletions = ScadnanoInsertionsDeletions::default();
        group_map.insert(String::from("default_group"), 0usize);
        grids.push(default_grid);
        let mut helices_per_group = vec![0];
        let mut groups: Vec<ScadnanoGroup> = vec![Default::default()];
        if let Some(scad_groups) = &scad.groups {
            for (name, g) in scad_groups {
                let group = g.to_grid_desc()?;
                groups.push(g.clone());
                group_map.insert(name.clone(), grids.len());
                grids.push(group);
                helices_per_group.push(0);
            }
        }
        for s in &scad.strands {
            for d in &s.domains {
                insertion_deletions.read_domain(d);
            }
        }
        let mut helices = BTreeMap::new();
        for (i, h) in scad.helices.iter().enumerate() {
            let helix = Helix::from_scadnano(h, &group_map, &groups, &mut helices_per_group)?;
            helices.insert(i, Arc::new(helix));
        }
        let mut strands = BTreeMap::new();
        for (i, s) in scad.strands.iter().enumerate() {
            let strand = Strand::from_scadnano(s, &insertion_deletions)?;
            strands.insert(i, strand);
        }
        println!("grids {grids:?}");
        println!("helices {helices:?}");
        Ok(Self {
            free_grids: FreeGrids::from_vec(grids),
            helices: Helices(Arc::new(helices)),
            strands: Strands(strands),
            small_spheres: Default::default(),
            scaffold_id: None, //TODO determine this value
            scaffold_sequence: None,
            scaffold_shift: None,
            groups: Default::default(),
            no_phantoms: Default::default(),
            helix_parameters: Some(HelixParameters::DEFAULT),
            anchors: Default::default(),
            organizer_tree: None,
            ensnano_version: ensnano_version(),
            group_attributes: Default::default(),
            cameras: Default::default(),
            ..Default::default()
        })
    }
}
