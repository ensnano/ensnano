use crate::{GuiAppState, left_panel::Message};
use ensnano_exports::ExportType;
use iced::widget::{button, column, scrollable};

#[derive(Default)]
pub(super) struct ExportMenu;

impl ExportMenu {
    pub(super) fn view<State>(&self) -> iced::Element<'_, Message<State>>
    where
        State: GuiAppState,
    {
        let content = column![
            button("Cancel").on_press(Message::CancelExport),
            button("Oxdna").on_press(Message::Export(ExportType::Oxdna)),
            button("Pdb").on_press(Message::Export(ExportType::Pdb)),
            button("Cadnano").on_press(Message::Export(ExportType::Cadnano)),
        ];

        scrollable(content).into()
    }
}
