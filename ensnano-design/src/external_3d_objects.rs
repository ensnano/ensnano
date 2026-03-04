use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Component, Path, PathBuf},
    sync::Arc,
};
use ultraviolet::{Rotor3, Vec3};

const DEFAULT_OPACITY: f32 = 1.0;
const DEFAULT_COLOR: u32 = 0xdb5530; // orange/red

fn show() -> bool {
    true
}

/// An external object to be drawn in the scene.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct External3DObject {
    opacity: f32,
    color: u32,
    position: Vec3,
    orientation: Rotor3,
    source_file: String,
    #[serde(default = "show")]
    visibility: bool,
}

pub struct External3DObjectDescriptor {
    pub object_path: PathBuf,
    pub design_path: PathBuf,
}

impl External3DObject {
    pub fn get_path_to_source_file<P: AsRef<Path>>(&self, design_path: P) -> PathBuf {
        RelativePathBuf::from(&self.source_file).to_path(design_path)
    }

    pub fn is_visible(&self) -> bool {
        self.visibility
    }

    pub fn set_visible(&mut self, value: bool) {
        self.visibility = value;
    }

    pub fn new(desc: External3DObjectDescriptor) -> Option<Self> {
        if let Some(rel_path) = diff_paths(&desc.object_path, &desc.design_path)
            .and_then(|rel_path| RelativePathBuf::from_path(rel_path).ok())
        {
            Some(Self {
                opacity: DEFAULT_OPACITY,
                color: DEFAULT_COLOR,
                position: Vec3::zero(),
                orientation: Rotor3::identity(),
                source_file: rel_path.to_string(),
                visibility: true,
            })
        } else {
            log::error!(
                "Could not compute path diff between {:?} and {:?}",
                desc.object_path.to_string_lossy(),
                desc.design_path.to_string_lossy()
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

    pub fn toggle_visibility(&mut self) {
        for (_, object) in Arc::<_>::make_mut(&mut self.0).iter_mut() {
            object.set_visible(!object.is_visible());
        }
    }
}

// Shamelessly copied from the pathdiff crate (MIT license)
// https://github.com/Manishearth/pathdiff/blob/bf1ea6a5e528f6f2/src/lib.rs#L43
fn diff_paths(path: impl AsRef<Path>, base: impl AsRef<Path>) -> Option<PathBuf> {
    let path = path.as_ref();
    let base = base.as_ref();

    if path.is_absolute() != base.is_absolute() {
        return path.is_absolute().then(|| PathBuf::from(path));
    }

    let mut ita = path.components();
    let mut itb = base.components();
    let mut comps: Vec<Component> = vec![];

    // ./foo and foo are the same
    if ita.clone().next() == Some(Component::CurDir) {
        ita.next();
    }
    if itb.clone().next() == Some(Component::CurDir) {
        itb.next();
    }

    loop {
        match (ita.next(), itb.next()) {
            (None, None) => break,
            (Some(a), None) => {
                comps.push(a);
                comps.extend(ita.by_ref());
                break;
            }
            (None, _) => comps.push(Component::ParentDir),
            (Some(a), Some(b)) if comps.is_empty() && a == b => (),
            (Some(a), Some(Component::CurDir)) => comps.push(a),
            (Some(_), Some(Component::ParentDir)) => return None,
            (Some(a), Some(_)) => {
                comps.push(Component::ParentDir);
                for _ in itb {
                    comps.push(Component::ParentDir);
                }
                comps.push(a);
                comps.extend(ita.by_ref());
                break;
            }
        }
    }

    Some(comps.iter().map(|c| c.as_os_str()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute() {
        fn abs(path: &str) -> String {
            // Absolute paths look different on Windows vs Unix.
            if cfg!(windows) {
                format!("C:\\{path}")
            } else {
                format!("/{path}")
            }
        }

        assert_diff_paths(&abs("foo"), &abs("bar"), Some("../foo"));
        assert_diff_paths(&abs("foo"), "bar", Some(&abs("foo")));
        assert_diff_paths("foo", &abs("bar"), None);
        assert_diff_paths("foo", "bar", Some("../foo"));
    }

    #[test]
    fn identity() {
        assert_diff_paths(".", ".", Some(""));
        assert_diff_paths("../foo", "../foo", Some(""));
        assert_diff_paths("./foo", "./foo", Some(""));
        assert_diff_paths("/foo", "/foo", Some(""));
        assert_diff_paths("foo", "foo", Some(""));
        assert_diff_paths("./foo", "foo", Some(""));
        assert_diff_paths("././foo", "foo", Some(""));
        assert_diff_paths("foo", "./foo", Some(""));
        assert_diff_paths("foo/foo", "./foo/foo", Some(""));

        assert_diff_paths("../foo/bar/baz", "../foo/bar/baz", Some(""));
        assert_diff_paths("foo/bar/baz", "foo/bar/baz", Some(""));
    }

    #[test]
    fn subset() {
        assert_diff_paths("foo", "fo", Some("../foo"));
        assert_diff_paths("./././fo", "foo", Some("../fo"));
    }

    #[test]
    fn empty() {
        assert_diff_paths("", "", Some(""));
        assert_diff_paths("foo", "", Some("foo"));
        assert_diff_paths("", "foo", Some(".."));
    }

    #[test]
    fn relative() {
        assert_diff_paths("../foo", "../bar", Some("../foo"));
        assert_diff_paths("../foo", "../foo/bar/baz", Some("../.."));
        assert_diff_paths("../foo/bar/baz", "../foo", Some("bar/baz"));
        assert_diff_paths("../foo", "bar", Some("../../foo"));
        assert_diff_paths("foo", "../bar", None);

        assert_diff_paths("foo/bar/baz", "foo", Some("bar/baz"));
        assert_diff_paths("foo/bar/baz", "foo/bar", Some("baz"));
        assert_diff_paths("foo/bar/baz", "foo/bar/baz", Some(""));
        assert_diff_paths("foo/bar/baz", "foo/bar/baz/", Some(""));

        assert_diff_paths("foo/bar/baz/", "foo", Some("bar/baz"));
        assert_diff_paths("foo/bar/baz/", "foo/bar", Some("baz"));
        assert_diff_paths("foo/bar/baz/", "foo/bar/baz", Some(""));
        assert_diff_paths("foo/bar/baz/", "foo/bar/baz/", Some(""));

        assert_diff_paths("foo/bar/baz", "foo/", Some("bar/baz"));
        assert_diff_paths("foo/bar/baz", "foo/bar/", Some("baz"));
        assert_diff_paths("foo/bar/baz", "foo/bar/baz", Some(""));
    }

    #[test]
    fn current_directory() {
        assert_diff_paths(".", "foo", Some("../."));
        assert_diff_paths("foo", ".", Some("foo"));
        assert_diff_paths("/foo", "/.", Some("foo"));

        assert_diff_paths("./foo/bar/baz", "foo", Some("bar/baz"));
        assert_diff_paths("foo/bar/baz", "./foo", Some("bar/baz"));
        assert_diff_paths("./foo/bar/baz", "./foo", Some("bar/baz"));
    }

    fn assert_diff_paths(path: &str, base: &str, expected: Option<&str>) {
        assert_eq!(diff_paths(path, base), expected.map(Into::into));
    }
}
