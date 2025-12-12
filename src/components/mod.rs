pub mod ansi_text;
pub mod app;
pub mod enhanced_input;
pub mod input_box;
pub mod message;
pub mod markdown_mixed_text;
pub mod permission_prompt;
pub mod status_line;
pub mod thinking_indicator;

pub use ansi_text::*;
pub use enhanced_input::*;
pub use input_box::*;
pub use markdown_mixed_text::*;
use iocraft::Color;
pub use permission_prompt::*;
pub use status_line::*;
pub use thinking_indicator::*;

pub const COLOR_PRIMARY: iocraft::Color = Color::Rgb {
    r: 181,
    g: 128,
    b: 255,
};
