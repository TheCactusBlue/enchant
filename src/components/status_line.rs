use iocraft::prelude::*;

use crate::{
    agent::{Session, models::model_info::get_model_info},
    components::COLOR_PRIMARY,
};

#[derive(Default, Props)]
pub struct StatusLineProps {
    pub session: Option<Session>,
}

#[component]
pub fn StatusLine(props: &StatusLineProps) -> impl Into<AnyElement<'static>> {
    let token_text = if let Some(session) = &props.session {
        let model_info = get_model_info(&session.model);
        if let Some(total) = session.total_tokens {
            let mut s = format!("Tokens: {:.1}k", total as f64 / 1e3);
            if let Some(max) = model_info.max_context {
                s.push_str(&format!(" / {:.1}k", max as f64 / 1e3));
            }
            s
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
