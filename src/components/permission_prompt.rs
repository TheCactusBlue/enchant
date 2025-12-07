use iocraft::prelude::*;

use crate::agent::tools::tool::ToolPreview;
use crate::components::COLOR_PRIMARY;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PermissionChoice {
    Approve,
    Deny,
}

#[derive(Default, Props)]
pub struct PermissionPromptProps {
    pub description: String,
    pub preview: Option<ToolPreview>,
    pub on_choice: HandlerMut<'static, PermissionChoice>,
}

#[component]
pub fn PermissionPrompt(
    mut hooks: Hooks,
    props: &mut PermissionPromptProps,
) -> impl Into<AnyElement<'static>> {
    let mut selected = hooks.use_state(|| 0usize); // 0 = Approve, 1 = Deny
    let mut on_choice = props.on_choice.take();
    let description = props.description.clone();
    let preview = props.preview.clone();

    let (w, _) = hooks.use_terminal_size();

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { kind, code, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Left | KeyCode::Char('h') => {
                        selected.set(0);
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        selected.set(1);
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        (on_choice)(PermissionChoice::Approve);
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        (on_choice)(PermissionChoice::Deny);
                    }
                    KeyCode::Enter => {
                        let choice = if *selected.read() == 0 {
                            PermissionChoice::Approve
                        } else {
                            PermissionChoice::Deny
                        };
                        (on_choice)(choice);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    });

    let approve_style = if *selected.read() == 0 {
        BorderStyle::Double
    } else {
        BorderStyle::Single
    };

    let deny_style = if *selected.read() == 1 {
        BorderStyle::Double
    } else {
        BorderStyle::Single
    };

    let approve_color = if *selected.read() == 0 {
        Color::Green
    } else {
        Color::DarkGrey
    };

    let deny_color = if *selected.read() == 1 {
        Color::Red
    } else {
        Color::DarkGrey
    };

    // Parse preview content into displayable lines
    let preview_lines: Vec<(String, Color)> = match &preview {
        Some(ToolPreview::Edit { diff }) => diff
            .lines()
            .map(|line| {
                let color = if line.starts_with('+') {
                    Color::Green
                } else if line.starts_with('-') {
                    Color::Red
                } else {
                    Color::DarkGrey
                };
                (line.to_string(), color)
            })
            .collect(),
        Some(ToolPreview::Write { content }) => content
            .lines()
            .map(|line| (line.to_string(), Color::Green))
            .collect(),
        None => vec![],
    };

    element! {
        View(
            flex_direction: FlexDirection::Column,
            border_style: BorderStyle::Single,
            border_color: COLOR_PRIMARY,
            padding: 1,
            min_width: w,
        ) {
            Text(
                content: "Permission Required",
                weight: Weight::Bold,
                color: COLOR_PRIMARY,
            )
            View(margin_top: 1) {
                Text(content: description)
            }
            #(if !preview_lines.is_empty() {
                Some(element! {
                    View(
                        margin_top: 1,
                        flex_direction: FlexDirection::Column,
                        border_style: BorderStyle::Single,
                        border_color: Color::DarkGrey,
                        padding: 1,
                    ) {
                        #(preview_lines.iter().map(|(line, color)| {
                            element! {
                                Text(content: line.clone(), color: *color)
                            }
                        }))
                    }
                })
            } else {
                None
            })
            View(margin_top: 1, flex_direction: FlexDirection::Row, gap: 2) {
                View(
                    border_style: approve_style,
                    border_color: approve_color,
                    padding_left: 2,
                    padding_right: 2,
                ) {
                    Text(content: "[Y] Approve", color: approve_color)
                }
                View(
                    border_style: deny_style,
                    border_color: deny_color,
                    padding_left: 2,
                    padding_right: 2,
                ) {
                    Text(content: "[N] Deny", color: deny_color)
                }
            }
            View(margin_top: 1) {
                Text(
                    content: "Use arrow keys or Y/N to choose, Enter to confirm",
                    color: Color::DarkGrey,
                )
            }
        }
    }
}
