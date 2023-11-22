use iced::widget::Text;
use iced::Element;
use iced::{Background, Color};
use iced_native::widget::{self, helpers::*};

/// A key identifing an element
pub trait ElementKey:
    Clone + std::cmp::Ord + std::fmt::Debug + serde::Serialize + serde::Deserialize<'static>
{
    type Section: std::cmp::Eq
        + std::cmp::Ord
        + core::convert::TryFrom<usize>
        + std::fmt::Debug
        + core::convert::Into<usize>;

    fn name(section: Self::Section) -> String;
    fn section(&self) -> Self::Section;
}

/// A root node of the organizer tree.
pub trait OrganizerElement: Clone + std::fmt::Debug + 'static {
    /// A type that describes all the attributes of an element that can be changed through
    /// interaction with the organizer.
    type Attribute: OrganizerAttribute;
    /// A type that is used to store the elements in a BTreeMap
    type Key: ElementKey;

    type AutoGroup: ToString + std::cmp::Ord + std::cmp::Eq + Clone + std::fmt::Debug;

    /// The name that will be displayed to represent the element
    fn display_name(&self) -> String;
    /// The key that will be used to store self in a BTreeMap
    fn key(&self) -> Self::Key;

    /// The aliases of the element that can be used to search it
    fn aliases(&self) -> Vec<String> {
        vec![self.display_name()]
    }

    fn attributes(&self) -> Vec<Self::Attribute>;

    fn all_repr() -> &'static [<Self::Attribute as OrganizerAttribute>::Repr] {
        Self::Attribute::all_repr()
    }

    fn auto_groups(&self) -> Vec<Self::AutoGroup>;
}

pub trait OrganizerAttributeRepr:
    std::cmp::Ord
    + std::cmp::Eq
    + core::convert::TryFrom<usize>
    + core::convert::Into<usize>
    + std::fmt::Debug
    + Clone
{
    fn all_repr() -> &'static [Self];
}

pub trait OrganizerAttribute: Clone + std::fmt::Debug + 'static + Ord + std::fmt::Display {
    /// A type used to represent the different values of self
    type Repr: OrganizerAttributeRepr;

    /// Map any value to its representent
    fn repr(&self) -> Self::Repr;
    /// The widget that will be used to change the value of self
    fn widget(&self) -> AttributeWidget<Self>;
    /// Map any value to a char that represents it
    fn char_repr(&self) -> AttributeDisplay;

    fn all_repr() -> &'static [Self::Repr] {
        Self::Repr::all_repr()
    }
}

pub enum AttributeDisplay {
    Icon(char),
    Text(String),
}

#[derive(Clone)]
pub enum AttributeWidget<E: OrganizerAttribute> {
    PickList { choices: &'static [E] },
    FlipButton { value_if_pressed: E },
}

#[derive(Default, Clone)]
pub(crate) struct AttributeDisplayer<A: OrganizerAttribute> {
    being_modified: bool,
    widget: Option<AttributeWidget<A>>,
    attribute: Option<A>,
}

impl<A: OrganizerAttribute> AttributeDisplayer<A> {
    pub fn new() -> Self {
        Self {
            being_modified: false,
            widget: None,
            attribute: None,
        }
    }

    pub fn update_attribute(&mut self, attribute: Option<A>) {
        self.update_widget(attribute.as_ref().map(|a| a.widget()));
        self.attribute = attribute;
    }

    pub fn update_widget(&mut self, widget: Option<AttributeWidget<A>>) {
        // If the widget is no longer a picklist, reset self.being_modified
        if let Some(AttributeWidget::PickList { .. }) = widget {
            ()
        } else {
            self.being_modified = false;
        }
        self.widget = widget;
    }

    pub fn view(&mut self) -> Option<Element<A>> {
        if let Some(widget) = self.widget.as_mut() {
            match widget {
                AttributeWidget::PickList { choices } => {
                    let mut picklist =
                        pick_list(*choices, self.attribute.clone(), |a| a).style(NoIcon {});
                    if let Some(AttributeDisplay::Icon(_)) =
                        self.attribute.as_ref().map(|a| a.char_repr())
                    {
                        picklist = picklist.font(super::ICONS).text_size(super::ICON_SIZE);
                    }
                    Some(picklist.into())
                }
                AttributeWidget::FlipButton { value_if_pressed } => {
                    let content = match self.attribute.as_ref().map(|a| a.char_repr()) {
                        Some(AttributeDisplay::Icon(c)) => super::icon(c),
                        Some(AttributeDisplay::Text(s)) => {
                            Text::new(s.clone()).size(super::ICON_SIZE)
                        }
                        _ => Text::new("???"),
                    };
                    Some(button(content).on_press(value_if_pressed.clone()).into())
                }
            }
        } else {
            None
        }
    }
}

/// An [pick_list::Appearance] where there is no icon.
///
struct NoIcon {}

impl widget::pick_list::StyleSheet for NoIcon {
    type Style = ();
    //type Style = iced_style::theme::PickList;
    // I think the good way to do it is to implement a custom Style.

    fn active(&self, _style: &Self::Style) -> widget::pick_list::Appearance {
        widget::pick_list::Appearance {
            text_color: Color::BLACK,
            placeholder_color: [0.4, 0.4, 0.4].into(),
            handle_color: Color::BLACK, // TODO: Check and adapt this value on the UI
            background: Background::Color([0.87, 0.87, 0.87].into()),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: [0.7, 0.7, 0.7].into(),
            // The values above use to be provided by `Default::default()`. Maybe there is a
            // “Default apparance” somewhere in iced 0.5
        }
    }

    fn hovered(&self, _style: &Self::Style) -> widget::pick_list::Appearance {
        widget::pick_list::Appearance {
            text_color: Color::BLACK,
            placeholder_color: [0.4, 0.4, 0.4].into(),
            handle_color: Color::BLACK, // TODO: Check and adapt this value on the UI
            background: Background::Color([0.87, 0.87, 0.87].into()),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: [0.7, 0.7, 0.7].into(),
            // The values above use to be provided by `Default::default()`. Maybe there is a
            // “Default apparance” somewhere in iced 0.5
        }
    }
}

impl From<NoIcon> for iced::theme::PickList {
    fn from(_: NoIcon) -> Self {
        Default::default()
    }
}
