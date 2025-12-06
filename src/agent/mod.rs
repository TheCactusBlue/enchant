pub mod tools;

use std::sync::Arc;

use crate::{
    agent::tools::{read::Read, tool::Toolset},
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
            messages: vec![ChatMessage::system(include_str!("../../prompts/BASE.md"))],
            tools: Arc::new(Toolset::new(vec![Box::new(Read)])),
        }
    }

    pub async fn think(&mut self) -> Result<(), Error> {
        let request = ChatRequest::new(self.messages.clone()).with_tools(self.tools.list_tools());

        let response = Client::builder()
            .with_auth_resolver(auth_resolver())
            .build()
            .exec_chat("claude-haiku-4-5", request, None)
            .await?;

        let calls = response.tool_calls();
        let tool_messages: Vec<ChatMessage> =
            futures::future::try_join_all(calls.iter().map(async |call| {
                let resp = self
                    .tools
                    .call(call.fn_name.clone(), call.fn_arguments.clone())
                    .await?;
                Ok::<_, Error>(ChatMessage::from(ToolResponse::new(
                    call.call_id.clone(),
                    resp,
                )))
            }))
            .await?;

        self.messages.push(ChatMessage::assistant(response.content));
        self.messages.extend(tool_messages);
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
