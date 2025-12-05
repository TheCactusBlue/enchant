use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct InputBoxProps {
    pub value: String,
    pub on_change: HandlerMut<'static, String>,
}

#[component]
pub fn InputBox(props: &mut InputBoxProps) -> impl Into<AnyElement<'static>> {
    element! {
        View (
            border_style: BorderStyle::Single,
            padding_left: 1,
            padding_right: 1,
            min_width: 80,
            min_height: 5,
            max_height: 40,
            border_color: Color::Rgb { r: 181, g: 128, b: 255 }
        ) {
            TextInput(
                has_focus: true,
                value: props.value.clone(),
                on_change: props.on_change.take(),
                multiline: true,
            )
        }
    }
}
