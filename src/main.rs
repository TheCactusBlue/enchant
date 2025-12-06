use iocraft::prelude::*;

use crate::agent::Session;
use crate::components::{InputBox, ThinkingIndicator};

pub mod agent;
pub mod components;
pub mod error;

#[component]
fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut input = hooks.use_state(|| "".to_string());
    let mut session = hooks.use_state(|| Session::new());
    let mut is_thinking = hooks.use_state(|| false);

    let on_submit = hooks.use_async_handler(move |value: String| async move {
        session.write().message(value).unwrap();
        is_thinking.set(true);
        Session::think_state(&mut session).await.unwrap();
        is_thinking.set(false);
    });

    element! {
      View (flex_direction: FlexDirection::Column) {
        View(flex_direction: FlexDirection::Column) {
            #(session.read().messages.iter().map(|m| {
                element! {
                    Text (content: format!("{:?}", m))
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

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    print!("{}[2J", 27 as char); // clear console
    element!(App).render_loop().await.unwrap();
    print!("{}[2J", 27 as char); // clear console
}
