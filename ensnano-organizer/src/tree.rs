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

use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;

use serde::Deserialize;
#[derive(Clone, Debug, Serialize)]
pub enum OrganizerTree<K> {
    Leaf(K),
    Node {
        name: String,
        #[serde(alias = "childrens")]
        children: Vec<OrganizerTree<K>>,
        expanded: bool,
        #[serde(default)]
        id: Option<GroupId>,
    },
}

impl<K: PartialEq> OrganizerTree<K> {
    pub fn get_names_of_groups_having(&self, element: &K) -> Vec<String> {
        let mut ret = Vec::new();
        match self {
            Self::Leaf(_) => (),
            Self::Node {
                name, children, id, ..
            } => {
                let rename: String = self.get_name_copy_with_id();
                for c in children {
                    match c {
                        Self::Leaf(k) if k == element => ret.push(rename.clone()),
                        Self::Leaf(_) => (),
                        node => {
                            let extention = node.get_names_of_groups_having(element);
                            ret.extend(extention)
                        }
                    }
                }
            }
        }
        ret.dedup();
        ret
    }

    // return the array of the names of all the groups in the tree
    pub fn get_names_of_all_groups(&self) -> Vec<String> {
        let mut ret = Vec::new();
        match self {
            Self::Leaf(_) => (),
            Self::Node {
                name, children, id, ..
            } => {
                let _ = ret.push(self.get_name_copy_with_id());
                for c in children {
                    let extention = c.get_names_of_all_groups();
                    ret.extend(extention);
                }
            }
        }
        ret.dedup();
        ret
    }

    pub fn get_names_of_all_groups_without_id(&self) -> Vec<String> {
        let mut ret = Vec::new();
        match self {
            Self::Leaf(_) => (),
            Self::Node {
                name, children, id, ..
            } => {
                if let Some(name) = self.get_name_copy() {
                    ret.push(name);
                }
                for c in children {
                    let extention = c.get_names_of_all_groups_without_id();
                    ret.extend(extention);
                }
            }
        }
        ret.dedup();
        ret
    }

    pub fn get_name_copy(&self) -> Option<String> {
        match self {
            Self::Leaf(_) => None,
            Self::Node {
                name,..
            } => Some(name.clone())
        }
    }

    pub fn get_name_copy_with_id(&self) -> String {
        match self {
            Self::Leaf(_) => "".to_string(),
            Self::Node {
                name, children, id, ..
            } => {
                if let Some(GroupId(x)) = id {
                    format!("{name}_{:0X}", x & 0xFFFF).to_string()
                } else {
                    name.clone()
                }
            }
        }
    }
}

/// Hashmap
impl<K: Eq + Hash + Copy> OrganizerTree<K> {
    pub fn get_hashmap_to_all_groupnames_with_prefix(
        &self,
        prefix: &str,
    ) -> HashMap<K, Vec<&str>, RandomState> {
        let mut hashmap = HashMap::new();

        match self {
            Self::Leaf(_) => (),
            Self::Node { name, children, .. } => {
                let trimmed_name = name.trim();
                let has_prefix = trimmed_name.starts_with(prefix);
                for c in children {
                    match c {
                        Self::Leaf(e) => {
                            let mut e_names: Vec<&str> = hashmap
                                .get(e)
                                .map(|x: &Vec<&str>| x.clone())
                                .unwrap_or(Vec::new());
                            if has_prefix {
                                e_names.push(trimmed_name.clone());
                            }
                            hashmap.insert(*e, e_names);
                        }
                        _ => {
                            let c_hashmap = c.get_hashmap_to_all_groupnames_with_prefix(prefix);
                            for (e, e_names) in c_hashmap {
                                let mut new_e_names: Vec<&str> = hashmap
                                    .get(&e)
                                    .map(|x: &Vec<&str>| x.clone())
                                    .unwrap_or(Vec::new());
                                new_e_names.extend(e_names);
                                if has_prefix {
                                    new_e_names.push(trimmed_name.clone());
                                }
                                hashmap.insert(e, new_e_names);
                            }
                        }
                    }
                }
            }
        }
        return hashmap;
    }
}

// For compatibility reasons, we need to implement Deserialize ourselved for OrganizerTree.
// We want to be able to accept both the old format (pre 0.3.0) and the current format.

#[derive(Clone, Debug, Serialize, Deserialize)]
enum OldOrganizerTree<K> {
    Leaf(K),
    Node(String, Vec<OrganizerTree<K>>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum NewOrganizerTree<K> {
    Leaf(K),
    Node {
        name: String,
        #[serde(alias = "childrens")]
        children: Vec<OrganizerTree<K>>,
        expanded: bool,
        #[serde(default)]
        id: Option<GroupId>,
    },
}

impl<K> OldOrganizerTree<K> {
    fn to_new(self) -> OrganizerTree<K> {
        match self {
            Self::Leaf(k) => OrganizerTree::Leaf(k),
            Self::Node(name, children) => OrganizerTree::Node {
                name,
                children,
                expanded: false,
                id: None,
            },
        }
    }
}

impl<K> NewOrganizerTree<K> {
    fn to_real(self) -> OrganizerTree<K> {
        match self {
            Self::Leaf(k) => OrganizerTree::Leaf(k),
            Self::Node {
                name,
                children,
                expanded,
                id,
            } => OrganizerTree::Node {
                name,
                children,
                expanded,
                id,
            },
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum NewOrOld<K> {
    New(NewOrganizerTree<K>),
    Old(OldOrganizerTree<K>),
}

impl<'de, K: Deserialize<'de>> Deserialize<'de> for OrganizerTree<K> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match NewOrOld::deserialize(deserializer) {
            Ok(NewOrOld::New(new_tree)) => Ok(new_tree.to_real()),
            Ok(NewOrOld::Old(old_tree)) => Ok(old_tree.to_new()),
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// The identifier of a group.
///
/// Used to map groups to group attributes.
pub struct GroupId(u64);

use rand::distributions::{Distribution, Standard};
use rand::Rng;

impl Distribution<GroupId> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> GroupId {
        let id: u64 = rng.gen();
        GroupId(id)
    }
}
