use ensnano_design::{
    bezier_plane::{BezierPathId, BezierVertexId},
    curves::bezier::BezierControlPoint,
    design_element::DesignElementKey,
    domains::Domain,
    grid::GridId,
    nucl::Nucl,
    phantom_element::PhantomElement,
};
use std::collections::BTreeSet;

use crate::app_state::design_interactor::DesignInteractor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Selection {
    Nucleotide(u32, Nucl),
    Bond(u32, Nucl, Nucl),
    Xover(u32, usize),
    Design(u32),
    Strand(u32, u32),
    Helix {
        design_id: u32,
        helix_id: usize,
        segment_id: usize,
    },
    Grid(u32, GridId),
    Phantom(PhantomElement),
    BezierControlPoint {
        helix_id: usize,
        bezier_control: BezierControlPoint,
    },
    BezierVertex(BezierVertexId),
    Nothing,
}

/// The object that is focused in the 3D scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CenterOfSelection {
    Nucleotide(u32, Nucl),
    Bond(u32, Nucl, Nucl),
    HelixGridPosition {
        design: u32,
        grid_id: GridId,
        x: isize,
        y: isize,
    },
    BezierControlPoint {
        helix_id: usize,
        bezier_control: BezierControlPoint,
    },
    BezierVertex {
        path_id: BezierPathId,
        vertex_id: usize,
    },
}

impl Selection {
    pub fn get_design(&self) -> Option<u32> {
        match self {
            Self::Helix { design_id, .. }
            | Self::Phantom(PhantomElement { design_id, .. })
            | Self::Design(design_id)
            | Self::Bond(design_id, _, _)
            | Self::Strand(design_id, _)
            | Self::Nucleotide(design_id, _)
            | Self::Grid(design_id, _)
            | Self::Xover(design_id, _) => Some(*design_id),
            Self::BezierControlPoint { .. } | Self::BezierVertex(_) => Some(0),
            Self::Nothing => None,
        }
    }

    pub fn info(&self) -> String {
        format!("{self:?}")
    }

    fn get_helices_containing_self(&self, reader: &DesignInteractor) -> Option<Vec<usize>> {
        match self {
            Self::Helix { helix_id, .. } => Some(vec![(*helix_id)]),
            Self::Nucleotide(_, nucl) => Some(vec![nucl.helix]),
            Self::Phantom(pe) => Some(vec![pe.to_nucl().helix]),
            Self::Strand(_, s_id) => {
                let strand = reader.get_strand_with_id(*s_id as usize)?;
                Some(strand.domains.iter().filter_map(Domain::helix).collect())
            }
            Self::Xover(_, xover_id) => {
                let (n1, n2) = reader.get_xover_with_id(*xover_id)?;
                Some(vec![n1.helix, n2.helix])
            }
            Self::Bond(_, n1, n2) => Some(vec![n1.helix, n2.helix]),
            Self::Nothing => Some(vec![]),
            Self::Design(_)
            | Self::Grid(_, _)
            | Self::BezierControlPoint { .. }
            | Self::BezierVertex(_) => None,
        }
    }

    fn get_grids_containing_self(&self, reader: &DesignInteractor) -> Option<Vec<GridId>> {
        if let Self::Grid(_, g_id) = self {
            Some(vec![*g_id])
        } else {
            let helices = self.get_helices_containing_self(reader)?;
            Some(
                helices
                    .iter()
                    .filter_map(|h| reader.get_helix_grid(*h))
                    .collect(),
            )
        }
    }
}

pub fn extract_nucls_and_xover_ends(
    selection: &[Selection],
    reader: &DesignInteractor,
) -> Vec<Nucl> {
    let mut ret = Vec::with_capacity(2 * selection.len());
    for s in selection {
        match s {
            Selection::Nucleotide(_, n) => ret.push(*n),
            Selection::Bond(_, n1, n2) => {
                ret.push(*n1);
                ret.push(*n2);
            }
            Selection::Xover(_, xover_id) => {
                if let Some((n1, n2)) = reader.get_xover_with_id(*xover_id) {
                    ret.push(n1);
                    ret.push(n2);
                } else {
                    log::error!("No xover with id {xover_id}");
                }
            }
            Selection::Strand(_, s_id) => {
                if let Some(ends) = reader.get_domain_ends(*s_id as usize) {
                    ret.extend(ends);
                } else {
                    log::error!("No strand with id {s_id}");
                }
            }
            _ => (),
        }
    }
    ret.dedup();
    ret
}

pub fn extract_strands_from_selection(selection: &[Selection]) -> Vec<usize> {
    selection.iter().filter_map(extract_one_strand).collect()
}

fn extract_one_strand(selection: &Selection) -> Option<usize> {
    if let Selection::Strand(_, s_id) = selection {
        Some(*s_id as usize)
    } else {
        None
    }
}

pub fn extract_grids(selection: &[Selection]) -> Vec<GridId> {
    selection.iter().filter_map(extract_one_grid).collect()
}

pub fn extract_only_grids(selection: &[Selection]) -> Option<Vec<GridId>> {
    selection.iter().map(extract_one_grid).collect()
}

fn extract_one_grid(selection: &Selection) -> Option<GridId> {
    if let Selection::Grid(_, g_id) = selection {
        Some(*g_id)
    } else {
        None
    }
}

pub fn list_of_strands(selection: &[Selection]) -> Option<(usize, Vec<usize>)> {
    let design_id = selection.first().and_then(Selection::get_design)?;
    let mut strands = BTreeSet::new();
    for s in selection {
        match s {
            Selection::Strand(d_id, s_id) => {
                if *d_id != design_id {
                    return None;
                }
                strands.insert(*s_id as usize);
            }
            _ => return None,
        }
    }
    let strands: Vec<usize> = strands.into_iter().collect();
    Some((design_id as usize, strands))
}

/// Convert a selection of bonds into a list of cross-overs
pub fn list_of_xover_ids(
    selection: &[Selection],
    reader: &DesignInteractor,
) -> Option<(usize, Vec<usize>)> {
    let design_id = selection.first().and_then(Selection::get_design)?;
    let mut xovers = BTreeSet::new();
    for s in selection {
        match s {
            Selection::Bond(d_id, n1, n2) => {
                if *d_id != design_id {
                    return None;
                }
                if let Some(id) = reader.get_xover_id(&(*n1, *n2)) {
                    xovers.insert(id);
                }
            }
            Selection::Xover(d_id, xover_id) => {
                if *d_id != design_id {
                    return None;
                }
                xovers.insert(*xover_id);
            }
            _ => return None,
        }
    }
    Some((design_id as usize, xovers.into_iter().collect()))
}

/// Convert a selection of bonds into a list of cross-overs
pub fn list_of_xover_as_nucl_pairs(
    selection: &[Selection],
    reader: &DesignInteractor,
) -> Option<(usize, Vec<(Nucl, Nucl)>)> {
    let design_id = selection.first().and_then(Selection::get_design)?;
    let mut xovers = BTreeSet::new();
    for s in selection {
        match s {
            Selection::Bond(d_id, n1, n2) => {
                if *d_id != design_id {
                    return None;
                }
                if reader.get_xover_id(&(*n1, *n2)).is_none() {
                    xovers.insert((*n1, *n2));
                }
            }
            Selection::Xover(d_id, xover_id) => {
                if *d_id != design_id {
                    return None;
                }
                if let Some(pair) = reader.get_xover_with_id(*xover_id) {
                    xovers.insert(pair);
                } else {
                    return None;
                }
            }
            // When selecting objects in the 2D view, one often selects strand extremities as well.
            // We do no want this to interfere with the copying of crossovers
            Selection::Nucleotide(_, _) => (),
            _ => return None,
        }
    }
    Some((design_id as usize, xovers.into_iter().collect()))
}

pub fn list_of_helices(selection: &[Selection]) -> Option<(usize, Vec<usize>)> {
    let design_id = selection.first().and_then(Selection::get_design)?;
    let mut helices = BTreeSet::new();
    for s in selection {
        match s {
            Selection::Helix {
                design_id: d_id,
                helix_id,
                ..
            } => {
                if *d_id != design_id {
                    return None;
                }
                helices.insert(*helix_id);
            }
            _ => return None,
        }
    }
    Some((design_id as usize, helices.into_iter().collect()))
}

pub fn list_of_free_grids(selection: &[Selection]) -> Option<Vec<usize>> {
    let mut ret = Vec::new();
    for s in selection {
        match s {
            Selection::Grid(_, GridId::FreeGrid(g_id)) => ret.push(*g_id),
            _ => return None,
        }
    }
    Some(ret)
}

pub fn list_of_bezier_vertices(selection: &[Selection]) -> Option<Vec<BezierVertexId>> {
    selection
        .iter()
        .map(|s| {
            if let Selection::BezierVertex(id) = s {
                Some(*id)
            } else {
                None
            }
        })
        .collect()
}

pub fn extract_helices_with_controls(selection: &[Selection]) -> Vec<usize> {
    let mut ret = Vec::new();
    for s in selection {
        if let Selection::Helix { helix_id, .. } = s {
            ret.push(*helix_id);
        } else if let Selection::BezierControlPoint { helix_id, .. } = s {
            ret.push(*helix_id);
        }
    }
    ret.dedup();
    ret
}

pub fn extract_control_points(selection: &[Selection]) -> Vec<(usize, BezierControlPoint)> {
    let mut ret = Vec::new();
    for s in selection {
        if let Selection::BezierControlPoint {
            helix_id,
            bezier_control,
        } = s
        {
            ret.push((*helix_id, *bezier_control));
        }
    }
    ret.dedup();
    ret
}

pub fn set_of_helices_containing_selection(
    selection: &[Selection],
    reader: &DesignInteractor,
) -> Option<Vec<usize>> {
    let mut ret = Vec::new();
    for s in selection {
        let helices = s.get_helices_containing_self(reader)?;
        ret.extend_from_slice(helices.as_slice());
    }
    ret.sort();
    ret.dedup();
    Some(ret)
}

pub fn set_of_grids_containing_selection(
    selection: &[Selection],
    reader: &DesignInteractor,
) -> Option<Vec<GridId>> {
    let mut ret = Vec::new();
    for s in selection {
        let grids = s.get_grids_containing_self(reader)?;
        ret.extend_from_slice(grids.as_slice());
    }
    ret.sort();
    ret.dedup();
    Some(ret)
}

/// Return true iff the selection is only made of helices that are not attached to a grid
pub fn all_helices_no_grid(selection: &[Selection], reader: &DesignInteractor) -> bool {
    let design_id = selection.first().and_then(Selection::get_design);
    let mut nb_helices = 0;
    if design_id.is_none() {
        return false;
    }
    let design_id = design_id.unwrap();

    for s in selection {
        match s {
            Selection::Helix {
                design_id: d_id,
                helix_id,
                ..
            } => {
                if *d_id != design_id {
                    return false;
                }
                if reader.get_grid_position_of_helix(*helix_id).is_some() {
                    return false;
                }
                nb_helices += 1;
            }
            s if s.get_design() == Some(design_id) => (),
            _ => return false,
        }
    }
    nb_helices >= 4
}

/// Extract all the elements of the form Selection::Nucl(_) from a slice of selection
pub fn extract_nucls_from_selection(selection: &[Selection]) -> Vec<Nucl> {
    let mut ret = vec![];
    for s in selection {
        if let Selection::Nucleotide(_, nucl) = s {
            ret.push(*nucl);
        }
    }
    ret
}

pub trait DesignElementKeySelection: Sized {
    fn from_selection(selection: &Selection, d_id: u32) -> Option<Self>;
    fn to_selection(&self, d_id: u32) -> Selection;
}

impl DesignElementKeySelection for DesignElementKey {
    fn from_selection(selection: &Selection, d_id: u32) -> Option<Self> {
        if selection.get_design() != Some(d_id) {
            return None;
        }

        match selection {
            Selection::Grid(_, GridId::FreeGrid(g_id)) => Some(Self::Grid(*g_id)),
            Selection::Helix { helix_id, .. } => Some(Self::Helix(*helix_id)),
            Selection::Strand(_, s_id) => Some(Self::Strand(*s_id as usize)),
            Selection::Nucleotide(_, nucl) => Some(Self::Nucleotide {
                helix: nucl.helix,
                position: nucl.position,
                forward: nucl.forward,
            }),
            Selection::Xover(_, xover_id) => Some(Self::CrossOver {
                xover_id: *xover_id,
            }),
            Selection::Phantom(pe) => {
                if pe.bond {
                    None
                } else {
                    let nucl = pe.to_nucl();
                    Some(Self::Nucleotide {
                        helix: nucl.helix,
                        position: nucl.position,
                        forward: nucl.forward,
                    })
                }
            }
            // TODO: make DesignElement out of these
            Selection::Grid(_, _)
            | Selection::Design(_)
            | Selection::Bond(_, _, _)
            | Selection::Nothing
            | Selection::BezierControlPoint { .. }
            | Selection::BezierVertex(_) => None,
        }
    }

    fn to_selection(&self, d_id: u32) -> Selection {
        match self {
            Self::Nucleotide {
                helix,
                position,
                forward,
            } => Selection::Nucleotide(
                d_id,
                Nucl {
                    helix: *helix,
                    position: *position,
                    forward: *forward,
                },
            ),
            Self::CrossOver { xover_id } => Selection::Xover(d_id, *xover_id),
            Self::Helix(h_id) => Selection::Helix {
                design_id: d_id,
                helix_id: *h_id,
                segment_id: 0,
            },
            Self::Strand(s_id) => Selection::Strand(d_id, *s_id as u32),
            Self::Grid(g_id) => Selection::Grid(d_id, GridId::FreeGrid(*g_id)),
        }
    }
}
