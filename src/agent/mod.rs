pub mod tools;

use crate::{agent::tools::get_default_tools, error::Error};
use genai::{
    Client,
    chat::{ChatMessage, ChatRequest},
};

#[derive(Clone, Debug)]
pub enum Message {
    User(String),
    Agent(String),
    Tool(String),
    Error(String),
}

#[derive(Clone, Debug)]
pub struct Session {
    pub messages: Vec<Message>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn build_chat(&self) -> Vec<ChatMessage> {
        self.messages
            .iter()
            .map(|m| match m {
                Message::User(msg) => ChatMessage::user(msg),
                _ => ChatMessage::system("Unknown"),
            })
            .collect()
    }

    pub async fn think(&mut self) -> Result<(), Error> {
        let chatreq = ChatRequest::new(self.build_chat()).with_tools(get_default_tools());

        let chat_response = Client::default()
            .exec_chat("claude-haiku-4-5", chatreq, None)
            .await?;

        Ok(())
    }
}
