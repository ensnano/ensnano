use crate::grid::{GridDescriptor, GridId};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default, Hash,
)]
/// Identifier of a free grid
pub struct FreeGridId(pub usize);

impl FreeGridId {
    pub fn to_grid_id(self) -> GridId {
        GridId::FreeGrid(self.0)
    }

    pub fn try_from_grid_id(grid_id: GridId) -> Option<Self> {
        if let GridId::FreeGrid(id) = grid_id {
            Some(Self(id))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
/// Collection of free grids descriptor
pub struct FreeGrids(pub(super) Arc<BTreeMap<FreeGridId, Arc<GridDescriptor>>>);

impl FreeGrids {
    pub fn make_mut(&mut self) -> FreeGridsMut<'_> {
        FreeGridsMut {
            new_map: BTreeMap::clone(&self.0),
            source: self,
        }
    }

    pub fn from_vec(vec: Vec<GridDescriptor>) -> Self {
        Self(Arc::new(
            vec.into_iter()
                .enumerate()
                .map(|(id, grid)| (FreeGridId(id), Arc::new(grid)))
                .collect(),
        ))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub fn get_from_g_id(&self, key: &GridId) -> Option<&GridDescriptor> {
        let free_id = FreeGridId::try_from_grid_id(*key)?;
        self.0.get(&free_id).map(AsRef::as_ref)
    }

    pub fn get(&self, grid_id: &FreeGridId) -> Option<&GridDescriptor> {
        self.0.get(grid_id).map(AsRef::as_ref)
    }

    pub fn values(&self) -> impl Iterator<Item = &GridDescriptor> {
        self.0.values().map(AsRef::as_ref)
    }
}

pub struct FreeGridsMut<'a> {
    source: &'a mut FreeGrids,
    new_map: BTreeMap<FreeGridId, Arc<GridDescriptor>>,
}

impl FreeGridsMut<'_> {
    pub fn push(&mut self, desc: GridDescriptor) -> GridId {
        let new_key = self
            .new_map
            .keys()
            .max()
            .map(|m| FreeGridId(m.0 + 1))
            .unwrap_or_default();
        self.new_map.insert(new_key, Arc::new(desc));
        GridId::FreeGrid(new_key.0)
    }

    pub fn get_mut(&mut self, g_id: &FreeGridId) -> Option<&mut GridDescriptor> {
        self.new_map.get_mut(g_id).map(Arc::make_mut)
    }

    pub fn get_mut_g_id(&mut self, g_id: &GridId) -> Option<&mut GridDescriptor> {
        let free_id = FreeGridId::try_from_grid_id(*g_id)?;
        self.get_mut(&free_id)
    }

    pub fn remove(&mut self, g_id: &GridId) -> Option<Arc<GridDescriptor>> {
        let free_id = FreeGridId::try_from_grid_id(*g_id)?;
        self.new_map.remove(&free_id)
    }
}

impl Drop for FreeGridsMut<'_> {
    fn drop(&mut self) {
        *self.source = FreeGrids(Arc::new(std::mem::take(&mut self.new_map)));
    }
}
