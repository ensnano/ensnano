use crate::icon::{ICON_SIZE, icon};
use iced::{
    Element,
    widget::{button, text},
};
use icondata::Icon;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// A key identifying an element
pub trait ElementKey: Clone + Ord + Debug + Serialize + Deserialize<'static> {
    type Section: Eq + Ord + TryFrom<usize> + Into<usize> + Debug;

    /// Name of the Element
    fn name(section: Self::Section) -> String;
    fn section(&self) -> Self::Section;
}

/// A root node of the organizer tree.
pub trait OrganizerElement: Clone + Debug + 'static {
    /// A type that describes all the attributes of an element that can be changed through
    /// interaction with the organizer.
    type Attribute: OrganizerAttribute;
    /// A type that is used to store the elements in a BTreeMap
    type Key: ElementKey;

    type AutoGroup: ToString + Ord + Eq + Clone + Debug;

    /// The name that will be displayed to represent the element
    fn display_name(&self) -> String;
    /// The key that will be used to store self in a BTreeMap
    fn key(&self) -> Self::Key;

    /// The aliases of the element that can be used to search it
    fn aliases(&self) -> Vec<String> {
        vec![self.display_name()]
    }

    fn attributes(&self) -> Vec<Self::Attribute>;

    fn all_discriminants() -> &'static [<Self::Attribute as OrganizerAttribute>::Discriminant] {
        Self::Attribute::all_discriminants()
    }
    fn min_max_domain_length_if_strand(&self) -> Option<(usize, usize)>;
    fn auto_groups(&self, last_domain_length_bounds: (usize, usize)) -> Vec<Self::AutoGroup>;
}

pub trait OrganizerAttributeDiscriminant:
    Ord + Eq + TryFrom<usize> + Into<usize> + Debug + Clone
{
    fn all_discriminants() -> &'static [Self];
}

pub trait OrganizerAttribute: Clone + Debug + 'static + Ord {
    /// A type used to represent the different values of self
    type Discriminant: OrganizerAttributeDiscriminant;

    /// Map any value to its discriminant
    fn discriminant(&self) -> Self::Discriminant;
    /// The widget that will be used to change the value of self
    fn widget(&self) -> AttributeWidget<Self>;
    /// Map any value to a char that represents it
    fn char_repr(&self) -> AttributeDisplay;

    fn all_discriminants() -> &'static [Self::Discriminant] {
        Self::Discriminant::all_discriminants()
    }
}

pub enum AttributeDisplay {
    Icon(Icon),
    Text(String),
}

#[derive(Clone)]
pub struct AttributeWidget<A: OrganizerAttribute> {
    value_if_pressed: A,
}
impl<A: OrganizerAttribute> AttributeWidget<A> {
    pub fn new(value_if_pressed: A) -> Self {
        Self { value_if_pressed }
    }
}

#[derive(Default, Clone)]
pub(crate) struct AttributeDisplayer<A: OrganizerAttribute> {
    being_modified: bool,
    widget: Option<AttributeWidget<A>>,
    attribute: Option<A>,
}

impl<Attrib: OrganizerAttribute> AttributeDisplayer<Attrib> {
    pub(crate) fn new() -> Self {
        Self {
            being_modified: false,
            widget: None,
            attribute: None,
        }
    }

    pub(crate) fn update_attribute(&mut self, attribute: Option<Attrib>) {
        self.update_widget(attribute.as_ref().map(OrganizerAttribute::widget));
        self.attribute = attribute;
    }

    pub(crate) fn update_widget(&mut self, widget: Option<AttributeWidget<Attrib>>) {
        self.being_modified = false;
        self.widget = widget;
    }

    pub(crate) fn view(&self) -> Option<Element<'_, Attrib>> {
        self.widget.as_ref().map(|widget| {
            match self.attribute.as_ref().map(OrganizerAttribute::char_repr) {
                Some(AttributeDisplay::Icon(c)) => button(icon(c)),
                Some(AttributeDisplay::Text(s)) => button(text(s).size(ICON_SIZE)),
                _ => button(text("???")),
            }
            .on_press(widget.value_if_pressed.clone())
            .into()
        })
    }
}
