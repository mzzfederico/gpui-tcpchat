use anyhow::{Context, Result};
use server::{ChatMessage, ClientId};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};

pub type MessageSender = mpsc::Sender<String>;
pub type MessageReceiver = Arc<Mutex<mpsc::Receiver<ChatMessage>>>;

#[derive(Clone)]
pub struct Client {
    message_sender: MessageSender,
    message_receiver: MessageReceiver,
    _connection_handle: Arc<tokio::task::JoinHandle<()>>,
}

impl Client {
    pub fn connect(address: &str) -> Result<Self> {
        let rt = tokio::runtime::Handle::current();
        let address = address.to_string();

        let (outgoing_tx, outgoing_rx) = mpsc::channel::<String>(100);
        let (incoming_tx, incoming_rx) = mpsc::channel::<ChatMessage>(100);

        let connection_handle = rt.spawn(async move {
            if let Err(e) = Self::run_connection(&address, outgoing_rx, incoming_tx).await {
                eprintln!("Connection error: {}", e);
            }
        });

        Ok(Client {
            message_sender: outgoing_tx,
            message_receiver: Arc::new(Mutex::new(incoming_rx)),
            _connection_handle: Arc::new(connection_handle),
        })
    }

    pub async fn send_message(&self, message: &str) -> Result<()> {
        self.message_sender
            .send(message.to_string())
            .await
            .with_context(|| "Failed to send message")
    }

    pub fn send_message_blocking(&self, message: &str) -> Result<()> {
        self.message_sender
            .try_send(message.to_string())
            .with_context(|| "Failed to send message")
    }

    pub async fn receive_message(&self) -> Option<ChatMessage> {
        self.message_receiver.lock().await.recv().await
    }

    pub fn try_receive_message(&self) -> Option<ChatMessage> {
        self.message_receiver
            .try_lock()
            .ok()?
            .try_recv()
            .ok()
    }

    async fn run_connection(
        address: &str,
        outgoing_rx: mpsc::Receiver<String>,
        incoming_tx: mpsc::Sender<ChatMessage>,
    ) -> Result<()> {
        let stream = TcpStream::connect(address)
            .await
            .with_context(|| format!("Failed to connect to {}", address))?;

        let (read_stream, write_stream) = tokio::io::split(stream);

        // Spawn task to handle outgoing messages
        let outgoing_task = Self::spawn_outgoing_handler(write_stream, outgoing_rx);

        // Spawn task to handle incoming messages
        let incoming_task = Self::spawn_incoming_handler(read_stream, incoming_tx);

        // Wait for either task to complete
        tokio::select! {
            _ = incoming_task => {},
            _ = outgoing_task => {},
        }

        Ok(())
    }

    fn spawn_outgoing_handler(
        mut write_stream: tokio::io::WriteHalf<TcpStream>,
        mut outgoing_rx: mpsc::Receiver<String>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(message) = outgoing_rx.recv().await {
                let formatted_message = format!("{}\n", message);
                if write_stream.write_all(formatted_message.as_bytes()).await.is_err() {
                    break;
                }
                if write_stream.flush().await.is_err() {
                    break;
                }
            }
        })
    }

    fn spawn_incoming_handler(
        read_stream: tokio::io::ReadHalf<TcpStream>,
        incoming_tx: mpsc::Sender<ChatMessage>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut reader = BufReader::new(read_stream);
            let mut line_buffer = String::new();

            loop {
                line_buffer.clear();
                match reader.read_line(&mut line_buffer).await {
                    Ok(0) => break, // Connection closed
                    Ok(_) => {
                        let message_text = line_buffer.trim();
                        println!("Received raw message: '{}'", message_text);

                        if !message_text.is_empty() {
                            if let Some(chat_message) = Self::parse_server_message(message_text) {
                                println!("Parsed message from {}: {}", chat_message.client_id, chat_message.content);
                                if incoming_tx.send(chat_message).await.is_err() {
                                    break; // Receiver dropped
                                }
                            } else {
                                println!("Failed to parse message: '{}'", message_text);
                            }
                        }
                    }
                    Err(_) => break, // Connection error
                }
            }
        })
    }

    fn parse_server_message(message: &str) -> Option<ChatMessage> {
        // Expected format: "Client 123: hello world"
        if !message.starts_with("Client ") {
            return None;
        }

        let colon_pos = message.find(": ")?;
        let client_id_str = &message[7..colon_pos]; // Skip "Client "
        let content = &message[colon_pos + 2..];    // Skip ": "

        let from_client_id = client_id_str.parse::<ClientId>().ok()?;

        Some(ChatMessage::new(from_client_id, content.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_invalid_message() {
        let message = "Invalid format";
        let parsed = Client::parse_server_message(message);

        assert!(parsed.is_none());
    }

    #[test]
    fn test_parse_valid_message() {
        let client_id = ClientId::new_v4();
        let message = format!("Client {}: Hello world", client_id);
        let parsed = Client::parse_server_message(&message);

        assert!(parsed.is_some());
        let chat_message = parsed.unwrap();
        assert_eq!(chat_message.client_id, client_id);
        assert_eq!(chat_message.content, "Hello world");
    }
}
