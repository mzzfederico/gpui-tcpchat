use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt::{Display, Formatter, Result};
use crate::chat::ClientId;


#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub client_id: ClientId,
    pub content: String,
    pub timestamp: i64,
}

impl ChatMessage {
    pub fn new(client_id: ClientId, content: String) -> Self {
        Self {
            client_id,
            content,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }
    }
}

impl Display for ChatMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "[{}] Client {}: {}", self.timestamp, self.client_id, self.content)
    }
}
