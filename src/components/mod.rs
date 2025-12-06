pub mod app;
pub mod input_box;
pub mod thinking_indicator;

pub use input_box::*;
use iocraft::Color;
pub use thinking_indicator::*;

pub const COLOR_PRIMARY: iocraft::Color = Color::Rgb {
    r: 181,
    g: 128,
    b: 255,
};
