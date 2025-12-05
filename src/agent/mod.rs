#[derive(Debug)]
pub enum Message {
    User(String),
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
}
