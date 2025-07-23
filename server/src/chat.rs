use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex};
use crate::messages::ChatMessage;

pub type ClientSender = tokio::sync::mpsc::Sender<ChatMessage>;
pub type ClientReceiver = tokio::sync::mpsc::Receiver<ChatMessage>;
pub type ClientId = Uuid;
pub type ClientRegistry = Arc<Mutex<HashMap<ClientId, ClientSender>>>;

pub struct ChatInstance {
    clients: ClientRegistry,
    broadcast: broadcast::Sender<ChatMessage>,
}

impl ChatInstance {
    pub fn new() -> Self {
        let (message_broadcaster, _) = broadcast::channel(1000);

        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            broadcast: message_broadcaster
        }
    }

    // Client registration
    async fn register_client(&self) -> (ClientId, ClientReceiver) {
        let client_id = ClientId::new_v4();
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        let mut clients = self.clients.lock().await;
        clients.insert(client_id, sender);
        (client_id, receiver)
    }

    async fn unregister_client(&self, client_id: ClientId) {
        let mut clients = self.clients.lock().await;
        clients.remove(&client_id);
    }

    pub async fn handle_connection(&self, stream: TcpStream) -> Result<()> {
        self.handle_client_session(stream).await
    }

    async fn handle_client_session(&self, stream: TcpStream) -> Result<()> {
        let (client_rx, client_tx) = tokio::io::split(stream);

        let (client_id, _) = self.register_client().await;

        println!("Client {} registered", client_id);

        // Spawn task to handle outgoing messages to this client
        let outgoing_task = self.spawn_message_routing(
            client_id,
            client_tx
        );

        // Handle incoming messages from this client
        let incoming_task = self.spawn_message_handler(client_id, client_rx);

        // Wait for either task to complete/break
        tokio::select! {
            _ = incoming_task => {},
            _ = outgoing_task => {},
        }

        println!("Client {} disconnected", client_id);

        self.unregister_client(client_id).await;

        Ok(())
    }

    fn spawn_message_routing(
        &self,
        _client_id: ClientId,
        mut client_tx: tokio::io::WriteHalf<TcpStream>
    ) -> tokio::task::JoinHandle<()> {
        let mut broadcast_rx = self.broadcast.subscribe();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    broadcast_result = broadcast_rx.recv() => {
                        match broadcast_result {
                            Ok(message) => {
                                if Self::send_message_to_client(&mut client_tx, &message).await.is_err() {
                                    break;
                                }
                                continue
                            }
                            Err(_) => break,   // Broadcast channel closed
                        }
                    }
                }
            }
        })
    }

    fn spawn_message_handler(
        &self,
        client_id: ClientId,
        client_rx: tokio::io::ReadHalf<TcpStream>,
    ) -> tokio::task::JoinHandle<()>  {
        let broadcast_tx = self.broadcast.clone();

        tokio::spawn(async move {
            let mut reader = BufReader::new(client_rx);
            let mut line_buffer = String::new();

            loop {
                line_buffer.clear();
                match reader.read_line(&mut line_buffer).await {
                    Ok(0) => break, // Connection closed
                    Ok(_) => {
                        let message_content = line_buffer.trim();
                        if !message_content.is_empty() {
                            let chat_message = ChatMessage::new(client_id, message_content.to_string());
                            let _ = broadcast_tx.send(chat_message);
                        }
                    }
                    Err(_) => break,
                }
            }
        })
    }

    async fn send_message_to_client(
        writer: &mut tokio::io::WriteHalf<TcpStream>,
        message: &ChatMessage,
    ) -> Result<()> {
        println!("{}: {}", message.client_id, message.content);
        let formatted_message = format!("Client {}: {}\n", message.client_id, message.content);
        writer.write_all(formatted_message.as_bytes()).await?;
        writer.flush().await?;
        Ok(())
    }
}
