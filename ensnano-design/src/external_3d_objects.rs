use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use ultraviolet::{Rotor3, Vec3};

const DEFAULT_OPACITY: f32 = 1.0;
const DEFAULT_COLOR: u32 = 0xdb5530; // orange/red

/// An external object to be drawn in the scene
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct External3DObject {
    opacity: f32,
    color: u32,
    position: Vec3,
    orientation: Rotor3,
    source_file: String,
}

pub struct External3DObjectDescriptor<P1: AsRef<Path>, P2: AsRef<Path>> {
    pub object_path: P1,
    pub design_path: P2,
}

impl External3DObject {
    pub fn get_path_to_source_file<P: AsRef<Path>>(&self, design_path: P) -> PathBuf {
        RelativePathBuf::from(&self.source_file).to_path(design_path)
    }

    pub fn new<P1: AsRef<Path>, P2: AsRef<Path>>(
        desc: External3DObjectDescriptor<P1, P2>,
    ) -> Option<Self> {
        if let Some(rel_path) = pathdiff::diff_paths(&desc.object_path, &desc.design_path)
            .and_then(|rel_path| RelativePathBuf::from_path(rel_path).ok())
        {
            Some(Self {
                opacity: DEFAULT_OPACITY,
                color: DEFAULT_COLOR,
                position: Vec3::zero(),
                orientation: Rotor3::identity(),
                source_file: rel_path.to_string(),
            })
        } else {
            log::error!(
                "Could not compute path diff between {:?} and {:?}",
                desc.object_path.as_ref().to_string_lossy(),
                desc.design_path.as_ref().to_string_lossy()
            );
            None
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct External3DObjectId(pub usize);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct External3DObjects(Arc<HashMap<External3DObjectId, External3DObject>>);

#[derive(Debug, Copy, Clone)]
pub struct External3DObjectsStamp(*const HashMap<External3DObjectId, External3DObject>);

impl External3DObjects {
    pub fn iter(&self) -> impl Iterator<Item = (&External3DObjectId, &External3DObject)> {
        self.0.iter()
    }

    pub fn was_updated(
        &self,
        old_stamp: Option<External3DObjectsStamp>,
    ) -> Option<External3DObjectsStamp> {
        let new = Some(External3DObjectsStamp(Arc::as_ptr(&self.0)));
        new.filter(|p| {
            if let Some(old) = old_stamp {
                p.0 != old.0
            } else {
                true
            }
        })
    }

    pub fn add_object(&mut self, object: External3DObject) {
        let key = self
            .0
            .keys()
            .min_by_key(|k| k.0)
            .map_or(External3DObjectId(0), |k| External3DObjectId(k.0 + 1));
        Arc::make_mut(&mut self.0).insert(key, object);
    }
}
