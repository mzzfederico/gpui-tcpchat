use anyhow::{Result};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};
use tokio::net::{TcpStream};
use tokio::sync::{broadcast, Mutex};
use crate::messages::Message;
use tokio::sync::mpsc::{Sender, Receiver};

pub type ClientId = Uuid;
pub type ClientRegistry = Arc<Mutex<HashMap<ClientId, (Sender<Message>, Receiver<Message>)>>>;

pub struct ChatInstance {
    clients: ClientRegistry,
    broadcast: broadcast::Sender<Message>,
}

impl ChatInstance {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);

        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            broadcast: tx
        }
    }

    pub async fn handle_connection(&self, stream: TcpStream) -> Result<()> {
        self.handle_client_session(stream).await?;
        Ok(())
    }

    // Client registration
    async fn register_client(&self, client_id: ClientId) -> ClientId {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        let mut clients = self.clients.lock().await;
        clients.insert(client_id, (sender, receiver));
        client_id
    }

    async fn unregister_client(&self, client_id: &ClientId) {
        let mut clients = self.clients.lock().await;
        clients.remove(&client_id);
    }

    async fn handle_client_session(&self, stream: TcpStream) -> Result<()> {
        let (client_rx, client_tx) = tokio::io::split(stream);
        let mut reader = BufReader::new(client_rx);
        let mut line_buffer = String::with_capacity(1000);

        let client_id = match reader.read_line(&mut line_buffer).await {
            Ok(0) => return Err(anyhow::anyhow!("Client disconnected during registration")),
            Ok(_) => {
                println!("{}", line_buffer);
                let message = line_buffer.trim();
                let message = serde_json::from_str::<Message>(message)?;
                if let Message::Log { client_id } = message {
                    self.register_client(client_id).await
                } else {
                    return Err(anyhow::anyhow!("Invalid message type during registration"));
                }
            }
            Err(e) => return Err(anyhow::anyhow!("Error reading from client: {}", e)),
        };

        // Handle incoming messages from this client
        let incoming_task = self.spawn_message_handler(&client_id, reader);
        let outgoing_task = self.spawn_message_routing(&client_id, client_tx);

        tokio::select! {
            _ = incoming_task => {},
            _ = outgoing_task => {},
        }

        self.unregister_client(&client_id).await;

        Ok(())
    }

    fn spawn_message_handler(
        &self,
        sending_id: &ClientId,
        mut reader: BufReader<ReadHalf<TcpStream>>
    ) -> tokio::task::JoinHandle<()>  {
        let broadcast_tx = self.broadcast.clone();
        let sending_id = sending_id.clone();

        tokio::spawn(async move {
            let mut line_buffer = String::new();

            loop {
                println!("{}", line_buffer);
                line_buffer.clear();
                match reader.read_line(&mut line_buffer).await {
                    Ok(0) => break,
                    Ok(_) => {
                        println!("Received message: {:?}", line_buffer);
                        let message = Self::parse_message(&line_buffer).unwrap();
                        match message {
                            Message::Chat {client_id: _, timestamp: _, content } => {
                                let _ = broadcast_tx.send(Message::chat(sending_id, content.as_str()));
                            }
                            _ => {}
                        }
                    }
                    Err(_) => break,
                }
            }
        })
    }

    fn parse_message(encoded: &str) -> Result<Message, serde_json::Error> {
        serde_json::from_str::<Message>(encoded.trim())
    }

    fn spawn_message_routing(
        &self,
        receiving_client: &ClientId,
        mut client_tx: WriteHalf<TcpStream>
    ) -> tokio::task::JoinHandle<()> {
        let mut broadcast_rx = self.broadcast.subscribe();
        let receiving_client = receiving_client.clone();

        tokio::spawn(async move {
            loop {
                if let Ok(message) = broadcast_rx.recv().await {
                    match message {
                        Message::Chat { content, client_id, timestamp: _ } => {
                            if receiving_client != client_id {
                                if Self::send_message_to_client(&mut client_tx, &Message::chat(client_id, content.as_str())).await.is_err() {
                                    break;
                                }
                            }
                        }
                        _ => continue
                    }
                }
            }
        })
    }

    async fn send_message_to_client(
        writer: &mut WriteHalf<TcpStream>,
        message: &Message,
    ) -> Result<()> {
        let json = serde_json::to_string(message)?;
        let json_with_newline = format!("{}\n", json);
        writer.write_all(json_with_newline.as_bytes()).await?;
        writer.flush().await?;
        Ok(())
    }
}
