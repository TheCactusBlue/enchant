use iocraft::prelude::*;

use crate::agent::{Message, Session};

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
                value: input.to_string(),
                on_change: move |new_value| input.set(new_value),
                multiline: true,
            )
        }
      }
    }
}

#[tokio::main]
async fn main() {
    print!("{}[2J", 27 as char); // clear console
    element!(App).render_loop().await.unwrap();
    print!("{}[2J", 27 as char); // clear console
}
