pub mod tools;

use std::sync::Arc;

use crate::{agent::tools::tool::Toolset, error::Error};
use genai::{
    Client, ModelIden,
    chat::{ChatMessage, ChatRequest, ToolResponse},
    resolver::{AuthData, AuthResolver},
};
use iocraft::prelude::State;

#[derive(Clone)]
pub struct Session {
    pub messages: Vec<ChatMessage>,
    pub tools: Arc<Toolset>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            tools: Arc::new(Toolset::new(vec![])),
        }
    }

    pub async fn think(&mut self) -> Result<(), Error> {
        let (assistant_message, tool_messages) = self.think_impl().await?;
        self.messages.push(assistant_message);
        self.messages.extend(tool_messages);
        Ok(())
    }

    async fn think_impl(&self) -> Result<(ChatMessage, Vec<ChatMessage>), Error> {
        let request = ChatRequest::new(self.messages.clone()).with_tools(self.tools.list_tools());

        let response = Client::builder()
            .with_auth_resolver(auth_resolver())
            .build()
            .exec_chat("claude-haiku-4-5", request, None)
            .await?;

        let calls = response.tool_calls();
        let tool_messages = futures::future::join_all(calls.iter().map(async |call| {
            let resp = self
                .tools
                .call(call.fn_name.clone(), call.fn_arguments.clone())
                .await;
            ChatMessage::from(ToolResponse::new(call.call_id.clone(), resp))
        }))
        .await;

        Ok((ChatMessage::assistant(response.content), tool_messages))
    }

    /// Call think on a State<Session>, avoiding holding the guard across await points
    pub async fn think_state(session: &mut State<Session>) -> Result<(), Error> {
        let mut sess = (*session.read()).clone();
        sess.think().await?;
        *session.write() = sess;
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
