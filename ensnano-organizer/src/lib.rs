//! Tree-like structure for iced.
//!
//! In ENSnano, `ensnano_design` implements the structures defined here, and are instantiated in
//! `ensnano_gui`.

pub mod element;
pub mod hoverable_container;
pub mod keyboard_priority;
pub mod tree;

type TreeId = Vec<usize>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeId<AutoGroupId> {
    Tree(TreeId),
    Section(usize),
    AutoGroup(AutoGroupId),
}

impl<E: std::fmt::Debug> NodeId<E> {
    pub fn push(&mut self, x: usize) {
        if let Self::Tree(v) = self {
            v.push(x);
        } else {
            log::error!("Trying to push on {self:?}");
        }
    }
}
