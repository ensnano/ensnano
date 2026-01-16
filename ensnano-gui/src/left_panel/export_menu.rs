use crate::messages::LeftPanelMessage;
use crate::state::GuiAppState;
use ensnano_exports::ExportType;
use iced::widget::{button, column, scrollable};

#[derive(Default)]
pub(super) struct ExportMenu;

impl ExportMenu {
    pub(super) fn view<State>(&self) -> iced::Element<'_, LeftPanelMessage<State>>
    where
        State: GuiAppState,
    {
        let content = column![
            button("Cancel").on_press(LeftPanelMessage::CancelExport),
            button("Oxdna").on_press(LeftPanelMessage::Export(ExportType::Oxdna)),
            button("Pdb").on_press(LeftPanelMessage::Export(ExportType::Pdb)),
            button("Cadnano").on_press(LeftPanelMessage::Export(ExportType::Cadnano)),
        ];

        scrollable(content).into()
    }
}
