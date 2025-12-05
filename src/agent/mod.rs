pub mod tools;

use crate::{
    agent::tools::{get_default_tools, tool},
    error::Error,
};
use genai::{
    Client,
    chat::{ChatMessage, ChatRequest},
};

#[derive(Clone, Debug)]
pub struct Session {
    pub messages: Vec<ChatMessage>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub async fn think(&mut self) -> Result<(), Error> {
        let toolset = get_default_tools();

        let request = ChatRequest::new(self.messages.clone()).with_tools(toolset.list_tools());

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
