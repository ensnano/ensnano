use {
    iced_widget::{svg::Handle, Svg},
    icondata::Icon,
    std::{
        collections::HashMap,
        sync::{LazyLock, Mutex},
    },
};

static ICON_HANDLE_CACHE: LazyLock<Mutex<HashMap<Icon, Handle>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn icon_to_svg(icon: Icon) -> Svg {
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
