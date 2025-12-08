use iocraft::prelude::*;

use crate::{
    agent::{Session, ThinkResult, config::load_config, tools::tool::PermissionRequest},
    components::{
        AnsiText, COLOR_PRIMARY, InputBox, PermissionChoice, PermissionPrompt, ThinkingIndicator,
        message::Message,
    },
};

/// UI state for the app.
#[derive(Clone, Default)]
enum AppState {
    #[default]
    Idle,
    Thinking,
    AwaitingPermission(Vec<PermissionRequest>),
}

#[component]
pub fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    hooks.use_future(async move {
        load_config().await.unwrap();
    });
    element! {
        Terminal
    }
}

#[component]
pub fn Terminal(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut input = hooks.use_state(|| "".to_string());
    let mut session = hooks.use_state(|| Session::new());
    let mut app_state = hooks.use_state(AppState::default);

    // Handler for continuing the think loop after permission is resolved
    let continue_thinking = hooks.use_async_handler({
        move |_: ()| async move {
            loop {
                let mut sess = (*session.read()).clone();
                let result = sess.think_step().await.unwrap();
                *session.write() = sess;

                match result {
                    ThinkResult::Done => {
                        app_state.set(AppState::Idle);
                        break;
                    }
                    ThinkResult::NeedsPermission(requests) => {
                        app_state.set(AppState::AwaitingPermission(requests));
                        break;
                    }
                    ThinkResult::Continue => {
                        // Keep looping
                    }
                }
            }
        }
    });

    // Handler for submitting a new message
    let on_submit = hooks.use_async_handler({
        move |value: String| async move {
            session.write().message(value).unwrap();
            app_state.set(AppState::Thinking);

            loop {
                let mut sess = (*session.read()).clone();
                let result = sess.think_step().await.unwrap();
                *session.write() = sess;

                match result {
                    ThinkResult::Done => {
                        app_state.set(AppState::Idle);
                        break;
                    }
                    ThinkResult::NeedsPermission(requests) => {
                        app_state.set(AppState::AwaitingPermission(requests));
                        break;
                    }
                    ThinkResult::Continue => {
                        // Keep looping
                    }
                }
            }
        }
    });

    // Handler for permission choice
    let mut on_permission_choice = {
        move |choice: PermissionChoice, requests: Vec<PermissionRequest>| {
            // Apply choice to all pending requests
            for request in &requests {
                match choice {
                    PermissionChoice::Approve => {
                        session.write().approve_permission(&request.call_id);
                    }
                    PermissionChoice::Deny => {
                        session.write().deny_permission(&request.call_id);
                    }
                }
            }
            app_state.set(AppState::Thinking);
            continue_thinking(());
        }
    };

    // Get current state for rendering
    let current_state = (*app_state.read()).clone();
    let sess = session.read();
    element! {
      View (flex_direction: FlexDirection::Column) {
        View(flex_direction: FlexDirection::Column, align_items: AlignItems::Center, gap: 1) {
            AnsiText(content: include_str!("../../prompts/char.ansi"))
            Text(content: format!("Enchant CLI · {} · {}",
            sess.model,
            sess.working_directory.display()
        ), color: COLOR_PRIMARY, weight: Weight::Bold)
        }
        View(flex_direction: FlexDirection::Column) {
            #(session.read().messages.iter().map(|m| {
                element! {
                    Message (message: m.clone(), toolset: sess.tools.clone())
                }
            }))
        }

        View(margin_top: 1) {
            #(match &current_state {
                AppState::Thinking => Some(element! {
                    ThinkingIndicator()
                }),
                _ => None,
            })
        }

        #(match current_state {
            AppState::AwaitingPermission(ref requests) => {
                let description = requests
                    .iter()
                    .map(|r| r.description.clone())
                    .collect::<Vec<_>>()
                    .join("\n");
                // Collect the first preview if available
                let preview = requests.iter().find_map(|r| r.preview.clone());
                let requests_clone = requests.clone();
                element! {
                    PermissionPrompt(
                        description: description,
                        preview: preview,
                        on_choice: move |choice| {
                            on_permission_choice(choice, requests_clone.clone());
                        },
                    )
                }.into_any()
            }
            _ => {
                element! {
                    InputBox(
                        value: input.to_string(),
                        on_change: move |new_value| input.set(new_value),
                        on_submit: move |value| {
                            on_submit(value);
                            input.set("".to_string());
                        },
                    )
                }.into_any()
            }
        })
      }
    }
}
