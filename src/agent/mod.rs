pub mod tools;

use crate::{
    agent::tools::{get_default_tools, tool::Toolset},
    error::Error,
};
use genai::{
    Client,
    chat::{ChatMessage, ChatRequest},
};

pub struct Session {
    pub messages: Vec<ChatMessage>,
    pub tools: Toolset,
}

impl Session {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            tools: get_default_tools(),
        }
    }

    pub async fn think(&mut self) -> Result<(), Error> {
        let request = ChatRequest::new(self.messages.clone()).with_tools(self.tools.list_tools());

        let response = Client::default()
            .exec_chat("claude-haiku-4-5", request, None)
            .await?;

        let calls = response.tool_calls();
        self.messages.push(ChatMessage::assistant(response.content));

        Ok(())
    }

    pub fn message(&mut self, message: String) -> Result<(), Error> {
        self.messages.push(ChatMessage::user(message));

        Ok(())
    }
}
