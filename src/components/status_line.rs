use iocraft::prelude::*;

use crate::components::COLOR_PRIMARY;

#[component]
pub fn StatusLine() -> impl Into<AnyElement<'static>> {
    element! {
        View (
            padding_left: 1,
            padding_right: 1,
            border_color: COLOR_PRIMARY
        ) {
            Text(
                content: "Ready".to_string(),
                color: COLOR_PRIMARY
            )
        }
    }
}
