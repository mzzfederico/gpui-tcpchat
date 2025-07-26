use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use crate::chat::ClientId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Log {
        client_id: ClientId,
    },
    Chat {
        content: String,
        client_id: ClientId,
        timestamp: i64,
    },
    Heartbeat,
    System {
        content: String,
    },
}

impl Message {
    #[allow(dead_code)]
    pub fn system(content: &str) -> Self {
        Self::System {
            content: content.to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn log(client_id: ClientId) -> Self {
        Self::Log { client_id }
    }

    pub fn chat(client_id: ClientId, content: &str) -> Self {
        Self::Chat {
            content: content.to_string(),
            client_id: client_id,
            timestamp: Self::timestamp(),
        }
    }

    #[allow(dead_code)]
    pub fn heartbeat() -> Self {
        Self::Heartbeat
    }

    pub fn timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }
}
