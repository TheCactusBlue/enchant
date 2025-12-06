pub mod prompt;
pub mod tools;

use std::sync::Arc;

use crate::{
    agent::{
        prompt::build_system_prompt,
        tools::{edit::Edit, glob::Glob, read::Read, tool::Toolset},
    },
    error::Error,
};
use genai::{
    Client, ModelIden,
    chat::{ChatMessage, ChatRequest, ToolResponse},
    resolver::{AuthData, AuthResolver},
};

#[derive(Clone)]
pub struct Session {
    pub messages: Vec<ChatMessage>,
    pub tools: Arc<Toolset>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            messages: vec![ChatMessage::system(build_system_prompt())],
            tools: Arc::new(Toolset::new(vec![
                Box::new(Read),
                Box::new(Glob),
                Box::new(Edit),
            ])),
        }
    }

    pub async fn think(&mut self) -> Result<(), Error> {
        let client = Client::builder()
            .with_auth_resolver(auth_resolver())
            .build();

        loop {
            let request =
                ChatRequest::new(self.messages.clone()).with_tools(self.tools.list_tools());

            let response = client.exec_chat("claude-haiku-4-5", request, None).await?;

            let tool_calls = response.tool_calls();

            // If no tool calls, we're done - add the final response and exit
            if tool_calls.is_empty() {
                self.messages.push(ChatMessage::assistant(response.content));
                break;
            }

            // Execute all tool calls and collect responses
            // Tool errors are returned as error messages so the agent can learn and retry
            let tool_responses: Vec<ToolResponse> =
                futures::future::join_all(tool_calls.iter().map(async |call| {
                    match self
                        .tools
                        .call(call.fn_name.clone(), call.fn_arguments.clone())
                        .await
                    {
                        Ok(resp) => ToolResponse::new(call.call_id.clone(), resp),
                        Err(e) => ToolResponse::new(
                            call.call_id.clone(),
                            format!("Error: {}", e),
                        ),
                    }
                }))
                .await;

            // Add both the tool calls from the model and our tool responses to the chat history
            let tool_calls = response.into_tool_calls();
            self.messages.push(ChatMessage::from(tool_calls));
            for tool_response in tool_responses {
                self.messages.push(ChatMessage::from(tool_response));
            }
        }

        Ok(())
    }

    pub fn message(&mut self, message: String) -> Result<(), Error> {
        self.messages.push(ChatMessage::user(message));
        Ok(())
    }
}

pub fn auth_resolver() -> AuthResolver {
    AuthResolver::from_resolver_fn(
        |_model_iden: ModelIden| -> Result<Option<AuthData>, genai::resolver::Error> {
            Ok(Some(AuthData::from_single(
                std::env::var("ENCHANT_KEY").unwrap(),
            )))
        },
    )
}
