pub mod tools;

use crate::{agent::tools::tool::Toolset, error::Error};
use genai::{
    Client, ModelIden,
    chat::{ChatMessage, ChatRequest, ToolResponse},
    resolver::{AuthData, AuthResolver},
};

pub struct Session {
    pub messages: Vec<ChatMessage>,
    pub tools: Toolset,
}

impl Session {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            tools: Toolset::new(vec![]),
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
        let calls = futures::future::join_all(calls.iter().map(async |call| {
            let resp = self
                .tools
                .call(call.fn_name.clone(), call.fn_arguments.clone())
                .await;
            ToolResponse::new(call.call_id.clone(), resp)
        }))
        .await;
        self.messages.push(ChatMessage::assistant(response.content));
        self.messages
            .append(&mut calls.into_iter().map(|x| ChatMessage::from(x)).collect());

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
