use iocraft::prelude::*;

use crate::components::COLOR_PRIMARY;
use crate::components::enhanced_input::EnhancedInput;

#[derive(Default, Props)]
pub struct InputBoxProps {
    pub value: String,
    pub on_change: HandlerMut<'static, String>,
    pub on_submit: HandlerMut<'static, String>,
}

#[component]
pub fn InputBox(mut hooks: Hooks, props: &mut InputBoxProps) -> impl Into<AnyElement<'static>> {
    let (w, _) = hooks.use_terminal_size();

    element! {
        View (
            border_style: BorderStyle::Single,
            padding_left: 1,
            padding_right: 1,
            min_width: w,
            min_height: 5,
            max_height: 40,
            border_color: COLOR_PRIMARY
        ) {
            EnhancedInput(
                has_focus: true,
                value: props.value.clone(),
                on_change: props.on_change.take(),
                on_submit: props.on_submit.take(),
                multiline: true,
                submit_on_enter: true,
            )
        }
    }
}
