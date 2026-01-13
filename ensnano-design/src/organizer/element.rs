use icondata::Icon;
use std::fmt::Debug;

use crate::elements::DnaAttributeDiscriminant;

pub trait OrganizerAttribute: Clone + Debug + 'static + Ord {
    /// Map any value to its discriminant
    fn discriminant(&self) -> DnaAttributeDiscriminant;
    /// The widget that will be used to change the value of self
    fn widget(&self) -> AttributeWidget<Self>;
    /// Map any value to a char that represents it
    fn char_repr(&self) -> AttributeDisplay;

    fn all_discriminants() -> &'static [DnaAttributeDiscriminant] {
        DnaAttributeDiscriminant::all_discriminants()
    }
}

pub enum AttributeDisplay {
    Icon(Icon),
    Text(String),
}

#[derive(Clone)]
pub struct AttributeWidget<A: OrganizerAttribute> {
    pub value_if_pressed: A,
}
impl<A: OrganizerAttribute> AttributeWidget<A> {
    pub fn new(value_if_pressed: A) -> Self {
        Self { value_if_pressed }
    }
}
