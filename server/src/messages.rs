use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::chat::ClientId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub client_id: ClientId,
    pub content: String,
    pub timestamp: i64,
}

impl Message {
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

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] Client {}: {}", self.timestamp, self.client_id, self.content)
    }
}
