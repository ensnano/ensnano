use iced::widget::{Svg, svg::Handle};
use icondata::Icon;
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

static ICON_HANDLE_CACHE: LazyLock<Mutex<HashMap<Icon, Handle>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(super) const ICON_SIZE: u16 = 10;

fn icon_to_svg(icon: Icon) -> Svg {
    let handle = ICON_HANDLE_CACHE.lock().unwrap().entry(icon).or_insert_with(|| {
            let xml = format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{}" fill="currentColor">{}</svg>"#,
                icon.view_box.unwrap_or("0 0 24 24"),
                icon.data,
            );
            Handle::from_memory(xml.into_bytes())
        }).clone(); // Handle owns an Arc<Data> so the clones are efficient

    Svg::new(handle)
}

pub(super) fn icon(icon: Icon) -> Svg {
    icon_to_svg(icon).width(ICON_SIZE).height(ICON_SIZE)
}

pub(super) fn expand_icon(expanded: bool) -> Svg {
    icon(if expanded {
        icondata::BsCaretDown
    } else {
        icondata::BsCaretRight
    })
}

pub(super) fn plus_icon() -> Svg {
    icon(icondata::BsPlus)
}

pub(super) fn edit_icon() -> Svg {
    icon(icondata::BsVectorPen)
}
