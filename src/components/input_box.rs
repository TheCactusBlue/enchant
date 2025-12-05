use iocraft::prelude::*;

use crate::components::COLOR_PRIMARY;

#[derive(Default, Props)]
pub struct InputBoxProps {
    pub value: String,
    pub on_change: HandlerMut<'static, String>,
    pub on_submit: HandlerMut<'static, String>,
}

#[component]
pub fn InputBox(mut hooks: Hooks, props: &mut InputBoxProps) -> impl Into<AnyElement<'static>> {
    let mut on_submit = props.on_submit.take();
    let value = props.value.clone();

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { kind, code, .. })
                if kind != KeyEventKind::Release && code == KeyCode::Enter =>
            {
                (on_submit)(value.clone());
            }
            _ => {}
        }
    });

    element! {
        View (
            border_style: BorderStyle::Single,
            padding_left: 1,
            padding_right: 1,
            min_width: 80,
            min_height: 5,
            max_height: 40,
            border_color: COLOR_PRIMARY
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
