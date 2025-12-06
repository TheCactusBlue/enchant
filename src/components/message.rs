use genai::chat::{ChatMessage, ChatRole};
use iocraft::prelude::*;

use crate::components::COLOR_PRIMARY;

#[derive(Default, Props)]
pub struct MessageProps {
    pub message: Option<ChatMessage>,
}

fn should_ignore_message(message: &ChatMessage) -> bool {
    match message.role {
        ChatRole::System => true,
        _ => false,
    }
}

const MESSAGE_LINE: BorderStyle = BorderStyle::Custom(BorderCharacters {
    top_left: ' ',
    top_right: ' ',
    bottom_left: ' ',
    bottom_right: ' ',
    left: '┃',
    right: ' ',
    top: ' ',
    bottom: ' ',
});

#[component]
pub fn Message(mut _hooks: Hooks, props: &MessageProps) -> impl Into<AnyElement<'static>> {
    element! {
        View() {
            #(if let Some(message) = &props.message && !should_ignore_message(&message) {
                Some(element! {
                    View (max_width: 80, border_style: MESSAGE_LINE, padding_left: 1, border_color: match message.role {
                        ChatRole::Assistant => Some(COLOR_PRIMARY),
                        _ => None,
                    }) {
                        Text (content: message.content.clone().into_joined_texts().unwrap_or("".to_string()) )
                    }
                })
            } else { None })
        }
    }
}
