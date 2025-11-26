use crate::{
    ensnano_exports::ExportType,
    ensnano_gui::{AppState, left_panel::Message},
};
use iced::widget::{button, column, scrollable, text};

#[derive(Default)]
pub struct ExportMenu;

impl ExportMenu {
    pub fn view<State>(&self) -> iced::Element<'_, Message<State>>
    where
        State: AppState,
    {
        let content = self::column![
            button(text("Cancel")).on_press(Message::CancelExport),
            button(text("Oxdna")).on_press(Message::Export(ExportType::Oxdna)),
            button(text("Pdb")).on_press(Message::Export(ExportType::Pdb)),
            button(text("Cadnano")).on_press(Message::Export(ExportType::Cadnano)),
        ];

        scrollable(content).into()
    }
}
