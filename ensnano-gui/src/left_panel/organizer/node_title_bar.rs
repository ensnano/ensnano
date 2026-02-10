use super::{
    AttributeDisplayer, GroupState,
    hoverable_container::HoverableContainer,
    theme::{OrganizerTheme, SelectionType},
};
use ensnano_design::{
    design_element::{DesignElement, DnaAttribute},
    organizer_tree::OrganizerNodeId,
};
use ensnano_state::gui::{
    drag_drop_target::{DragDropTarget, DragIdentifier},
    messages::OrganizerMessage,
};
use ensnano_utils::keyboard_priority::keyboard_priority;
use iced::{
    Element, Length,
    widget::{Row, Space, button, container, horizontal_space, mouse_area, row, text, text_input},
};

/// A data structure whose view is a "title bar" for a group or a section.
pub(super) struct NodeTitleBar {
    pub(super) state: GroupState,
    pub(super) attribute_displayers: Vec<AttributeDisplayer>,
    pub(super) name_input_id: text_input::Id,
}

impl NodeTitleBar {
    pub(super) fn new() -> Self {
        Self {
            state: GroupState::Idle,
            attribute_displayers: vec![
                AttributeDisplayer::new();
                DesignElement::all_discriminants().len()
            ],
            name_input_id: text_input::Id::unique(),
        }
    }

    pub(super) fn new_section() -> Self {
        Self {
            state: GroupState::NotEditable,
            attribute_displayers: vec![],
            name_input_id: text_input::Id::unique(),
        }
    }

    pub(super) fn start_editing(&mut self) {
        log::info!("reached view");
        self.state = GroupState::Editing;
    }

    pub(super) fn stop_editing(&mut self) {
        self.state = GroupState::Idle;
    }

    pub(super) fn view(
        &self,
        theme: &OrganizerTheme,
        is_selected: SelectionType,
        name: &String,
        id: OrganizerNodeId,
        expanded: bool,
    ) -> Element<'_, OrganizerMessage> {
        let title_row = match &self.state {
            GroupState::Idle => {
                let mut row: Row<'_, _> = row![
                    button(super::icon::expand_icon(expanded))
                        .on_press(OrganizerMessage::expand(id.clone(), !expanded)),
                    Space::with_width(5.0),
                    mouse_area(
                        text_input("New group name...", name).id(self.name_input_id.clone())
                    )
                    .on_press(OrganizerMessage::edit(id.clone())),
                    horizontal_space(),
                    button(super::icon::plus_icon())
                        .on_press(OrganizerMessage::add_selection_to_group(id.clone())), // TODO: change icon later !!!
                ];

                for ad in &self.attribute_displayers {
                    if let Some(view) = ad.view() {
                        let id = id.clone();
                        row = row.push(
                            view.map(move |m| OrganizerMessage::attribute_selected(m, id.clone())),
                        );
                    }
                }

                row = row.push(
                    button(super::icon::icon(icondata::BsTrash))
                        .on_press(OrganizerMessage::delete(id.clone())),
                );
                row
            }
            GroupState::Editing => {
                let mut row = row![
                    button(super::icon::expand_icon(expanded))
                        .on_press(OrganizerMessage::expand(id.clone(), !expanded)),
                    Space::with_width(5.0),
                    keyboard_priority(
                        "New group name...",
                        OrganizerMessage::SetKeyboardPriority,
                        text_input("New group name...", name)
                            .id(self.name_input_id.clone())
                            .on_input(|s| { OrganizerMessage::name_input(s) })
                            .on_submit(OrganizerMessage::stop_edit())
                    ),
                    horizontal_space(),
                    button(super::icon::plus_icon())
                        .on_press(OrganizerMessage::add_selection_to_group(id.clone())), // TODO: change icon later !!!
                ];
                for ad in &self.attribute_displayers {
                    if let Some(view) = ad.view() {
                        let id = id.clone();
                        row = row.push(
                            view.map(move |m| OrganizerMessage::attribute_selected(m, id.clone())),
                        );
                    }
                }
                row = row.push(
                    button(super::icon::icon(icondata::BsTrash))
                        .on_press(OrganizerMessage::delete(id.clone())),
                );
                row
            }
            GroupState::NotEditable => row![
                button(super::icon::expand_icon(expanded))
                    .on_press(OrganizerMessage::expand(id.clone(), !expanded)),
                Space::with_width(5.0),
                text(name),
            ],
        };

        let title_button = button(title_row)
            .on_press(OrganizerMessage::node_selected(id.clone()))
            .width(Length::Fill)
            .style(theme.selected(is_selected));
        let title_button = HoverableContainer::new(title_button)
            .on_hover(OrganizerMessage::node_hovered(id.clone(), true))
            .on_unhover(OrganizerMessage::node_hovered(id.clone(), false));
        let title_button = DragDropTarget::new(
            container(title_button).width(Length::Fill),
            DragIdentifier::Group { id },
        );
        container(title_button).into()
    }

    pub(super) fn update_attributes(&mut self, attributes: &[Option<DnaAttribute>]) {
        for (i, a) in attributes.iter().enumerate() {
            self.attribute_displayers[i].update_attribute(a.clone());
        }
    }
}
