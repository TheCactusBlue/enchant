use genai::chat::{ChatMessage, ChatRole};
use iocraft::prelude::*;

use crate::components::COLOR_PRIMARY;

const COLOR_TOOL: Color = Color::Rgb {
    r: 100,
    g: 149,
    b: 237,
}; // Cornflower blue

#[derive(Default, Props)]
pub struct MessageProps {
    pub message: Option<ChatMessage>,
    pub toolset: Option<std::sync::Arc<crate::agent::tools::tool::Toolset>>,
}

fn has_displayable_content(message: &ChatMessage) -> bool {
    message.content.contains_text() || !message.content.tool_calls().is_empty()
}

fn should_ignore_message(message: &ChatMessage) -> bool {
    if !has_displayable_content(message) {
        return true;
    }
    matches!(message.role, ChatRole::System)
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
pub fn Message(mut hooks: Hooks, props: &MessageProps) -> impl Into<AnyElement<'static>> {
    let (w, _) = hooks.use_terminal_size();
    let toolset = &props.toolset;

    element! {
        View() {
            #(if let Some(message) = &props.message && !should_ignore_message(&message) {
                let tool_calls = message.content.tool_calls();
                let text_content = message.content.clone().into_joined_texts().unwrap_or("".to_string());
                Some(element! {
                    View(flex_direction: FlexDirection::Column, max_width: w) {
                        #(tool_calls.iter().map(|tc| {
                            let display = toolset.as_ref().and_then(|ts| Some(ts.describe_action(&tc.fn_name, &tc.fn_arguments)))
                                .unwrap_or_else(|| format!("{}({})", tc.fn_name, tc.fn_arguments));
                            element! {
                                View(max_width: w, border_style: MESSAGE_LINE, padding_left: 1, border_color: COLOR_TOOL) {
                                    Text(content: display, color: COLOR_TOOL, wrap: TextWrap::Wrap)
                                }
                            }
                        }).collect::<Vec<_>>())
                        #(if !text_content.is_empty() {
                            Some(element! {
                                View (max_width: w, border_style: MESSAGE_LINE, padding_left: 1, border_color: match message.role {
                                    ChatRole::Assistant => Some(COLOR_PRIMARY),
                                    _ => None,
                                }) {
                                    Text (content: text_content)
                                }
                            })
                        } else { None })
                    }
                })
            } else { None })
        }
    }
}
