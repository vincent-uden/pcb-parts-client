use iced::widget::{Svg, svg};
use lazy_static::lazy_static;

const SVG_SERVER: &[u8] = include_bytes!("../assets/icons/server.svg");
const SVG_USER: &[u8] = include_bytes!("../assets/icons/user.svg");

pub fn server() -> svg::Handle {
    svg::Handle::from_memory(SVG_SERVER)
}

pub fn user() -> svg::Handle {
    svg::Handle::from_memory(SVG_USER)
}
