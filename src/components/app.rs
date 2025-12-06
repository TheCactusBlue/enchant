use std::env;

use iocraft::prelude::*;

use crate::{
    agent::Session,
    components::{AnsiText, COLOR_PRIMARY, InputBox, ThinkingIndicator, message::Message},
};

#[component]
pub fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut input = hooks.use_state(|| "".to_string());
    let mut session = hooks.use_state(|| Session::new());
    let mut is_thinking = hooks.use_state(|| false);

    let on_submit = hooks.use_async_handler(move |value: String| async move {
        session.write().message(value).unwrap();
        is_thinking.set(true);

        let mut sess = (*session.read()).clone();
        sess.think().await.unwrap();
        *session.write() = sess;

        is_thinking.set(false);
    });

    element! {
      View (flex_direction: FlexDirection::Column) {
        View(flex_direction: FlexDirection::Column, align_items: AlignItems::Center, gap: 1) {
            AnsiText(content: include_str!("../../prompts/hat.ansi"))
            Text(content: format!("Enchant CLI"), color: COLOR_PRIMARY, weight: Weight::Bold)
            // Text(content: env::current_dir().unwrap().to_string_lossy())
        }
        View(flex_direction: FlexDirection::Column) {
            #(session.read().messages.iter().map(|m| {
                element! {
                    Message (message: m.clone())
                }
            }))
        }

        View(margin_top: 1) {
            #(if *is_thinking.read() {
                Some(element! {
                    ThinkingIndicator()

                })
            } else {
                None
            })
        }

        InputBox(
            value: input.to_string(),
            on_change: move |new_value| input.set(new_value),
            on_submit: move |value| {
                on_submit(value);
                input.set("".to_string());
            },
        )
      }
    }
}
