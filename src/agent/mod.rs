pub mod tools;

use crate::{agent::tools::tool::Toolset, error::Error};
use genai::{
    Client, ModelIden,
    chat::{ChatMessage, ChatRequest, ToolResponse},
    resolver::{AuthData, AuthResolver},
};
use iocraft::prelude::State;

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
        // Clone data out before async work to avoid holding non-Send guard across await
        let (messages, tools_list) = {
            let sess = session.read();
            (sess.messages.clone(), sess.tools.list_tools())
        };

        let request = ChatRequest::new(messages).with_tools(tools_list);

        let response = Client::builder()
            .with_auth_resolver(auth_resolver())
            .build()
            .exec_chat("claude-haiku-4-5", request, None)
            .await?;

        // TODO: Handle tool calls when tools are implemented
        // For now, tools list is empty so we skip tool execution

        // Now write results back
        session.write().messages.push(ChatMessage::assistant(response.content));
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
