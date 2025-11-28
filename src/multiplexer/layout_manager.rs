//! Manage the layout of the window in different areas.
//!
//! The layout is a tree-like structure. A [LayoutNode] can be one of the three following variants:
//!
//! * [LayoutNode::Area] — represents an actual —or drawable— region.
//! * [LayoutNode::VSplit] — represents a region divided vertically between two subregions.
//! * [LayoutNode::HSplit] — represents a region divided horizontally between two subregions.

use ensnano_interactor::graphics::GuiComponentType;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use winit::dpi::PhysicalPosition;

/// Half-width of a “resize region”.
const RESIZE_REGION_WIDTH: f64 = 0.001;

/// Node of a [LayoutTree], representing a region of the window.
///
/// Each variant contains four attributes (`left`, `top`, `right`, `bottom`) that represent the
/// positions of the boundaries of the area, expressed as a fraction between 0.0 and 1.0
///
/// To learn more about recursive data structures in Rust, see [here](https://rust-leipzig.github.io/architecture/2016/12/20/idiomatic-trees-in-rust/)
///
#[derive(Clone, Debug)]
enum LayoutNode {
    /// A leaf of a [LayoutTree]. It represents an area that can be drawn on.
    /// The last attribute is the identifier of the area.
    Area {
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
        identifier: usize,
    },

    /// A Node representing a vertical splitting of an area.
    VSplit {
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
        left_proportion: f64,
        left_child: Rc<RefCell<LayoutNode>>,
        right_child: Rc<RefCell<LayoutNode>>,
        resizable: Option<usize>,
    },

    /// A Node representing a horizontal splitting of an area.
    HSplit {
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
        top_proportion: f64,
        top_child: Rc<RefCell<LayoutNode>>,
        bottom_child: Rc<RefCell<LayoutNode>>,
        resizable: Option<usize>,
    },
}

/// A pointer to a [LayoutNode].
///
type LayoutNodePtr = Rc<RefCell<LayoutNode>>;

/// Data structure representing the partition of the window.
///
/// # Example
///
///     use layout_manager;
///     let mut layout = LayoutTree::new();
///     let (top_bar, content_section) = layout.hsplit(0, 0.05, false);
///     let (left_panel, main_section) = layout.vsplit(content_section, 0.2, true);
///
///
pub struct LayoutTree {
    /// The root node of the LayoutTree.
    root: LayoutNodePtr,
    /// An array mapping area identifier to leaves of the LayoutTree.
    areas: Vec<LayoutNodePtr>,
    /// An array mapping area identifier to ElementType.
    element_types: Vec<GuiComponentType>,
    /// A HashMap mapping element types to area identifier.
    area_identifiers: HashMap<GuiComponentType, usize>,
    /// An array mapping area to their parent node.
    parents: Vec<usize>,
}

impl LayoutTree {
    /// Create a new layout tree with a single area in it.
    ///
    /// Note — The identifier of the area is `0`.
    ///
    pub fn new() -> Self {
        let root = Rc::new(RefCell::new(LayoutNode::Area {
            left: 0.,
            top: 0.,
            right: 1.,
            bottom: 1.,
            identifier: 0,
        }));
        let area = vec![Rc::clone(&root)];
        let element_type = vec![GuiComponentType::Unattributed];
        let area_identifiers = HashMap::new();
        Self {
            root,
            areas: area,
            element_types: element_type,
            area_identifiers,
            parents: vec![0],
        }
    }

    /// Split an area vertically in two.
    ///
    /// Arguments:
    ///
    /// * `parent_ident` — the identifier of the area being split.
    /// * `left_proportion` — the proportion of the initial area attributed to the left part.
    ///
    /// Return value:
    ///
    /// A pair `(l, r)` where `l` is the identifier of the left area and `r` the identifier of the
    /// right area.
    ///
    /// Note — The identifier of the root area is usually `0`.
    ///
    pub fn vsplit(
        &mut self,
        parent_ident: usize,
        left_proportion: f64,
        resizable: bool,
    ) -> (usize, usize) {
        // Define identifiers for the new subareas.
        let left_ident = self.areas.len();
        let right_ident = self.areas.len() + 1;
        // Split and store subareas.
        let (left_area, right_area) = {
            let mut area = self.areas[parent_ident].borrow_mut();
            area.vsplit(left_proportion, left_ident, right_ident, resizable)
        };
        self.areas.push(left_area);
        self.areas.push(right_area);
        self.parents.push(parent_ident);
        self.parents.push(parent_ident);
        self.element_types.push(GuiComponentType::Unattributed);
        self.element_types.push(GuiComponentType::Unattributed);
        let parent_element_type = self.element_types[parent_ident];
        self.area_identifiers.remove(&parent_element_type);
        self.element_types[parent_ident] = GuiComponentType::Unattributed;
        (left_ident, right_ident)
    }

    /// Split an area horizontally in two.
    ///
    /// Arguments:
    ///
    /// * `parent_ident`: the identifier of the area being split.
    /// * `top_proportion`: the proportion of the initial area attributed to the top part.
    ///
    /// Return value:
    ///
    /// A pair `(t, b)` where `t` is the identifier of the top area and `b` the identifier of the
    /// bottom area.
    ///
    /// Note — The identifier of the root area is usually `0`.
    ///
    pub fn hsplit(
        &mut self,
        parent_ident: usize,
        top_proportion: f64,
        resizable: bool,
    ) -> (usize, usize) {
        // Define identifiers for the new subareas.
        let top_ident = self.areas.len();
        let bottom_ident = self.areas.len() + 1;
        // Split and store subareas.
        let (top_area, bottom_area) = {
            let mut area = self.areas[parent_ident].borrow_mut();
            area.hsplit(top_proportion, top_ident, bottom_ident, resizable)
        };
        self.areas.push(top_area);
        self.areas.push(bottom_area);
        self.parents.push(parent_ident);
        self.parents.push(parent_ident);
        self.element_types.push(GuiComponentType::Unattributed);
        self.element_types.push(GuiComponentType::Unattributed);
        let parent_element_type = self.element_types[parent_ident];
        self.area_identifiers.remove(&parent_element_type);
        self.element_types[parent_ident] = GuiComponentType::Unattributed;
        (top_ident, bottom_ident)
    }

    /// Undo a split by deleting the leaf with type `old_leaf` and its sibling and giving their parent the type
    /// `new_leaf`.
    pub fn merge(&mut self, old_leaf: GuiComponentType, new_leaf: GuiComponentType) {
        let area_ident = *self
            .area_identifiers
            .get(&old_leaf)
            .expect("Try to get the area of an element that was not given one");
        let parent_ident = self.parents[area_ident];
        let (_, child2) = self.areas[parent_ident].borrow_mut().merge(parent_ident);
        let old_brother = self.element_types[child2];
        self.area_identifiers.remove(&old_leaf);
        self.area_identifiers.remove(&old_brother);
        self.attribute_element(parent_ident, new_leaf);
    }

    /// Get the Element owning the pixel `(x, y)`
    pub(super) fn get_area_pixel(&self, x: f64, y: f64) -> PixelRegion {
        let identifier = self.root.borrow().get_area_pixel(x, y);
        if let PixelRegion::Area(identifier) = identifier {
            PixelRegion::Element(self.element_types[identifier])
        } else {
            identifier
        }
    }

    /// Return the boundaries of the area attributed to an element
    pub fn get_area(&self, element: GuiComponentType) -> Option<(f64, f64, f64, f64)> {
        let area_id = *self.area_identifiers.get(&element)?;
        match *self.areas[area_id].borrow() {
            LayoutNode::Area {
                left,
                top,
                right,
                bottom,
                ..
            } => Some((left, top, right, bottom)),
            _ => panic!("got split_node"),
        }
    }

    pub fn get_area_id(&self, element: GuiComponentType) -> Option<usize> {
        self.area_identifiers.get(&element).copied()
    }

    /// Attribute an element_type to an area.
    pub fn attribute_element(&mut self, area_ident: usize, element_type: GuiComponentType) {
        let old_element = self.element_types[area_ident];
        self.area_identifiers.remove(&old_element);
        self.element_types[area_ident] = element_type;
        self.area_identifiers.insert(element_type, area_ident);
    }

    pub fn resize(&self, node_id: usize, new_prop: f64) {
        self.areas[node_id].borrow_mut().resize(new_prop);
    }

    pub fn resize_click(
        &self,
        node_id: usize,
        position: &PhysicalPosition<f64>,
        clicked_position: &PhysicalPosition<f64>,
        old_proportion: f64,
    ) {
        self.areas[node_id]
            .borrow_mut()
            .resize_click(position, clicked_position, old_proportion);
        if log::log_enabled!(log::Level::Debug) {
            log::debug!("node {node_id} resized");
            log::debug!("{:#?}", self.areas[node_id]);
        }
    }

    pub fn get_proportion(&self, region: usize) -> Option<f64> {
        self.areas.get(region).and_then(|a| a.borrow().proportion())
    }

    pub fn log_tree(&self) {
        println!("{:#?}", self.root);
    }
}

impl LayoutNode {
    /// Horizontally split the current area in two.
    ///
    /// Arguments:
    ///
    /// * `top_proportion` — the proportion of the initial area attributed to the top area. It must
    ///   be between 0.0 and 1.0
    /// * `top_ident` — identifier given to the top area.
    /// * `bottom_ident` — identifier given to the bottom area.
    ///
    /// Return value:
    ///
    /// A pair `(t, b)` where `t` is a pointer to the top area and `b` is a pointer to the bottom
    /// area
    pub fn hsplit(
        &mut self,
        top_proportion: f64,
        top_ident: usize,
        bottom_ident: usize,
        resizable: bool,
    ) -> (LayoutNodePtr, LayoutNodePtr) {
        assert!((0. ..=1.).contains(&top_proportion));
        match self {
            Self::Area {
                left,
                top,
                right,
                bottom,
                identifier,
                ..
            } => {
                let separation = top_proportion * (*top + *bottom);
                let top_area = Rc::new(RefCell::new(Self::Area {
                    left: *left,
                    top: *top,
                    right: *right,
                    bottom: separation,
                    identifier: top_ident,
                }));
                let bottom_area = Rc::new(RefCell::new(Self::Area {
                    left: *left,
                    top: separation,
                    right: *right,
                    bottom: *bottom,
                    identifier: bottom_ident,
                }));
                *self = Self::HSplit {
                    top: *top,
                    bottom: *bottom,
                    left: *left,
                    right: *right,
                    top_proportion,
                    top_child: Rc::clone(&top_area),
                    bottom_child: Rc::clone(&bottom_area),
                    resizable: Some(*identifier).filter(|_| resizable),
                };
                (top_area, bottom_area)
            }
            _ => {
                panic!("You should not be splitting a HSplit of VSplit node.");
            }
        }
    }

    /// Vertically split the current area in two.
    ///
    /// Arguments
    ///
    /// * `left_proportion` — the proportion of the initial area attributed to the left area. It
    ///   must be between 0. and 1.
    /// * `left_ident` — identifier be given to the left area.
    /// * `right_ident` — identifier be given to the right area.
    ///
    /// Return value
    ///
    /// A pair `(l, r)` where `l` is a pointer to the left area and `r` is a pointer to the right
    /// area
    pub fn vsplit(
        &mut self,
        left_proportion: f64,
        left_ident: usize,
        right_ident: usize,
        resizable: bool,
    ) -> (LayoutNodePtr, LayoutNodePtr) {
        assert!((0. ..=1.).contains(&left_proportion));
        match self {
            Self::Area {
                left,
                top,
                right,
                bottom,
                identifier,
                ..
            } => {
                let separation = left_proportion * (*left + *right);
                let left_area = Rc::new(RefCell::new(Self::Area {
                    left: *left,
                    top: *top,
                    right: separation,
                    bottom: *bottom,
                    identifier: left_ident,
                }));
                let right_area = Rc::new(RefCell::new(Self::Area {
                    left: separation,
                    top: *top,
                    right: *right,
                    bottom: *bottom,
                    identifier: right_ident,
                }));
                *self = Self::VSplit {
                    left: *left,
                    top: *top,
                    right: *right,
                    bottom: *bottom,
                    left_proportion,
                    left_child: Rc::clone(&left_area),
                    right_child: Rc::clone(&right_area),
                    resizable: Some(*identifier).filter(|_| resizable),
                };
                (left_area, right_area)
            }
            _ => {
                panic!("You should not be splitting a HSplit of VSplit node.");
            }
        }
    }

    /// Merge the two children.
    ///
    /// The children of the current node must must be leaves. This is the inverse operation of
    /// [vsplit](Self::vsplit) or [hsplit](Self::hsplit).
    ///
    /// Gives the identifier `ident` to the new merged Area, and return the identifier of the two
    /// merged children.
    ///
    pub fn merge(&mut self, ident: usize) -> (usize, usize) {
        let merged_node;
        let new_self = match self {
            Self::VSplit {
                left_child: l_child,
                right_child: r_child,
                ..
            } => match (l_child.borrow().clone(), r_child.borrow().clone()) {
                (
                    Self::Area {
                        left,
                        top,
                        bottom,
                        identifier: c1,
                        ..
                    },
                    Self::Area {
                        right,
                        identifier: c2,
                        ..
                    },
                ) => {
                    merged_node = (c1, c2);
                    Self::Area {
                        left,
                        top,
                        right,
                        bottom,
                        identifier: ident,
                    }
                }
                _ => panic!("You cannot merge non-Area nodes."),
            },
            Self::HSplit {
                top_child: t_child,
                bottom_child: b_child,
                ..
            } => match (t_child.borrow().clone(), b_child.borrow().clone()) {
                (
                    Self::Area {
                        left,
                        top,
                        right,
                        identifier: c1,
                        ..
                    },
                    Self::Area {
                        bottom,
                        identifier: c2,
                        ..
                    },
                ) => {
                    merged_node = (c1, c2);
                    Self::Area {
                        left,
                        top,
                        right,
                        bottom,
                        identifier: ident,
                    }
                }
                _ => panic!("You cannot merge non-Area nodes."),
            },
            Self::Area { .. } => panic!("You cannot merge an Area."),
        };
        *self = new_self;
        merged_node
    }

    /// Return the identifier of the leaf owning pixel `(x, y)`
    pub fn get_area_pixel(&self, x: f64, y: f64) -> PixelRegion {
        match self {
            Self::Area { identifier, .. } => PixelRegion::Area(*identifier),
            Self::VSplit {
                left,
                right,
                left_proportion,
                left_child: l_child,
                right_child: r_child,
                resizable,
                ..
            } => {
                let separation = *left + *left_proportion * (*right - *left);
                if let Some(id) = resizable.filter(|_| {
                    x >= separation - RESIZE_REGION_WIDTH && x <= separation + RESIZE_REGION_WIDTH
                }) {
                    PixelRegion::Resize(id)
                } else if x <= separation {
                    l_child.borrow().get_area_pixel(x, y)
                } else {
                    r_child.borrow().get_area_pixel(x, y)
                }
            }
            Self::HSplit {
                top,
                bottom,
                top_proportion,
                top_child: t_child,
                bottom_child: b_child,
                resizable,
                ..
            } => {
                let separation = *top + *top_proportion * (*bottom - *top);
                if let Some(id) =
                    resizable.filter(|_| y >= separation - 0.02 && y <= separation + 0.02)
                {
                    PixelRegion::Resize(id)
                } else if y <= separation {
                    t_child.borrow().get_area_pixel(x, y)
                } else {
                    b_child.borrow().get_area_pixel(x, y)
                }
            }
        }
    }

    /// Resize a split layout according to mouse click.
    pub fn resize_click(
        &mut self,
        position: &PhysicalPosition<f64>,
        clicked_position: &PhysicalPosition<f64>,
        old_proportion: f64,
    ) {
        match self {
            Self::VSplit { left, right, .. } => {
                let delta = position.x - clicked_position.x;
                let delta_prop = delta / (*right - *left);
                let new_prop = (old_proportion + delta_prop).clamp(0.05, 0.95);
                self.resize(new_prop);
            }
            Self::HSplit { top, bottom, .. } => {
                let delta = position.y - clicked_position.y;
                let delta_prop = delta / (*bottom - *top);
                let new_prop = (old_proportion + delta_prop).clamp(0.05, 0.95);
                self.resize(new_prop);
            }
            Self::Area { .. } => {
                println!("WARNING, RESIZING AREA, THIS IS A BUG");
            }
        }
    }

    /// Resize a split layout according to the new proportion given.
    pub fn resize(&mut self, new_proportion: f64) {
        match self {
            Self::VSplit {
                left,
                top,
                right,
                bottom,
                left_proportion,
                left_child: l_child,
                right_child: r_child,
                ..
            } => {
                let separation = *left + new_proportion * (*right - *left);
                l_child
                    .borrow_mut()
                    .propagate_resize(*left, *top, separation, *bottom);
                r_child
                    .borrow_mut()
                    .propagate_resize(separation, *top, *right, *bottom);
                *left_proportion = new_proportion;
            }
            Self::HSplit {
                left,
                top,
                right,
                bottom,
                top_proportion,
                top_child: t_child,
                bottom_child: b_child,
                ..
            } => {
                let separation = *top + new_proportion * (*bottom - *top);
                t_child
                    .borrow_mut()
                    .propagate_resize(*left, *top, *right, separation);
                b_child
                    .borrow_mut()
                    .propagate_resize(*left, separation, *right, *bottom);
                *top_proportion = new_proportion;
            }
            Self::Area { .. } => println!("WARNING RESIZING LEAF, THIS IS A BUG!!!"),
        }
    }

    /// Propagate a resize through the tree.
    fn propagate_resize(&mut self, new_left: f64, new_top: f64, new_right: f64, new_bottom: f64) {
        match self {
            Self::HSplit {
                left,
                top,
                right,
                bottom,
                top_proportion,
                top_child: c1,
                bottom_child: c2,
                ..
            } => {
                let separation = new_top + *top_proportion * (new_bottom - new_top);
                *left = new_left;
                *top = new_top;
                *right = new_right;
                *bottom = new_bottom;
                c1.borrow_mut()
                    .propagate_resize(new_left, new_top, new_right, separation);
                c2.borrow_mut()
                    .propagate_resize(new_left, separation, new_right, new_bottom);
            }
            Self::VSplit {
                left,
                top,
                right,
                bottom,
                left_proportion,
                left_child: c1,
                right_child: c2,
                ..
            } => {
                let separation = new_left + *left_proportion * (new_right - new_left);
                *left = new_left;
                *top = new_top;
                *right = new_right;
                *bottom = new_bottom;
                c1.borrow_mut()
                    .propagate_resize(new_left, new_top, separation, new_bottom);
                c2.borrow_mut()
                    .propagate_resize(separation, new_top, new_right, new_bottom);
            }
            Self::Area {
                left,
                top,
                right,
                bottom,
                ..
            } => {
                *left = new_left;
                *top = new_top;
                *right = new_right;
                *bottom = new_bottom;
            }
        }
    }

    pub fn proportion(&self) -> Option<f64> {
        match self {
            Self::VSplit {
                left_proportion, ..
            } => Some(*left_proportion),
            Self::HSplit { top_proportion, .. } => Some(*top_proportion),
            Self::Area { .. } => None,
        }
    }
}

/// Types of region in which a pixel may lie.
#[derive(Debug)]
pub(super) enum PixelRegion {
    /// The pixel is on a region attributed to a certain element
    Element(GuiComponentType),
    /// The pixel is on a region where clicking must resize a panel
    Resize(usize),
    /// The pixel is on a given area
    Area(usize),
}
