use std::vec;

use genai::{
    Client,
    chat::{self, ChatRequest},
};

use crate::error::Error;

#[derive(Debug)]
pub enum Message {
    User(String),
    Agent(String),
    Error(String),
}

#[derive(Debug)]
pub struct Session {
    pub messages: Vec<Message>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub async fn think(&mut self) -> Result<(), Error> {
        let chatreq = ChatRequest::new(vec![]);

        let chat_response = Client::default()
            .exec_chat("claude-haiku-4-5", chatreq, None)
            .await?;

        Ok(())
    }
}
