use iocraft::prelude::*;

fn main() {
    element! {
        View(
            border_style: BorderStyle::Round,
            border_color: Color::Magenta,
        ) {
            Text(content: "Hello, world!")
        }
    }
    .print();
}
