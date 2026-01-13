use crate::elements::DnaAttribute;
use icondata::Icon;

pub enum AttributeDisplay {
    Icon(Icon),
    Text(String),
}

#[derive(Clone)]
pub struct AttributeWidget {
    pub value_if_pressed: DnaAttribute,
}
impl AttributeWidget {
    pub fn new(value_if_pressed: DnaAttribute) -> Self {
        Self { value_if_pressed }
    }
}
