use iocraft::prelude::*;

use crate::agent::{Message, Session};
use crate::components::InputBox;

pub mod agent;
pub mod components;

#[component]
fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut input = hooks.use_state(|| "".to_string());
    let mut session = hooks.use_state(|| Session::new());

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { kind, code, .. })
                if kind != KeyEventKind::Release && code == KeyCode::Enter =>
            {
                let mut sess = session.write();
                sess.messages.push(Message::User(input.to_string()));
                input.set("".to_string());
            }
            _ => {}
        }
    });
    element! {
      View (flex_direction: FlexDirection::Column) {
        #(session.read().messages.iter().map(|m| {
            element! {
                Text (content: format!("{:?}", m))
            }
        }))

        InputBox(
            value: input.to_string(),
            on_change: move |new_value| input.set(new_value),
        )
      }
    }
}

#[tokio::main]
async fn main() {
    print!("{}[2J", 27 as char); // clear console
    element!(App).render_loop().await.unwrap();
    print!("{}[2J", 27 as char); // clear console
}
