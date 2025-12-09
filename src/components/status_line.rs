use iocraft::prelude::*;

use crate::{agent::Session, components::COLOR_PRIMARY};

#[derive(Default, Props)]
pub struct StatusLineProps {
    pub session: Option<Session>,
}

#[component]
pub fn StatusLine(props: &StatusLineProps) -> impl Into<AnyElement<'static>> {
    let token_text = if let Some(session) = &props.session {
        if let Some(total) = session.total_tokens {
            format!("Tokens: {}", total)
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };

    element! {
        View (
            padding_left: 1,
            padding_right: 1,
            border_color: COLOR_PRIMARY
        ) {
            Text(
                content: token_text,
                color: COLOR_PRIMARY
            )
        }
    }
}
