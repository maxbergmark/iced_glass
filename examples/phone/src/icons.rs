use iced::widget::svg;
use once_cell::sync::Lazy;
use std::collections::HashMap;

fn h(bytes: &'static [u8]) -> svg::Handle {
    svg::Handle::from_memory(bytes)
}

pub static SVGS: Lazy<HashMap<&'static str, svg::Handle>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // OUTLINE ICONS
    m.insert("wifi", h(include_bytes!("../assets/wifi-outline.svg")));
    m.insert(
        "bluetooth",
        h(include_bytes!("../assets/bluetooth-outline.svg")),
    );
    m.insert(
        "airplane",
        h(include_bytes!("../assets/airplane-outline.svg")),
    );
    m.insert("camera", h(include_bytes!("../assets/camera-outline.svg")));
    m.insert(
        "desktop",
        h(include_bytes!("../assets/desktop-outline.svg")),
    );
    m.insert(
        "finger-print",
        h(include_bytes!("../assets/finger-print-outline.svg")),
    );
    m.insert("at", h(include_bytes!("../assets/at-outline.svg")));
    m.insert("moon", h(include_bytes!("../assets/moon-outline.svg")));
    m.insert("sunny", h(include_bytes!("../assets/sunny-outline.svg")));
    m.insert(
        "volume-off",
        h(include_bytes!("../assets/volume-off-outline.svg")),
    );
    m.insert(
        "volume-high",
        h(include_bytes!("../assets/volume-high-outline.svg")),
    );

    // SOLID ICONS
    m.insert("gear", h(include_bytes!("../assets/gear.svg")));
    m.insert("play", h(include_bytes!("../assets/play.svg")));
    m.insert("play-back", h(include_bytes!("../assets/play-back.svg")));
    m.insert(
        "play-forward",
        h(include_bytes!("../assets/play-forward.svg")),
    );

    m
});

/// Get an SVG handle by name
pub fn svg_handle(name: &str) -> svg::Handle {
    SVGS.get(name).cloned().unwrap_or_else(|| {
        // fallback icon if missing
        svg::Handle::from_memory(include_bytes!("../assets/gear.svg"))
    })
}
