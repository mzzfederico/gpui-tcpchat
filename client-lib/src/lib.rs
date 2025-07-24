use anyhow::{Context, Result};
use server::Message;
use std::str::FromStr;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, mpsc};

use uuid::Uuid;

#[derive(Clone)]
pub struct Client {
    pub message_sender: Arc<Mutex<mpsc::Sender<String>>>,
    pub message_receiver: Arc<Mutex<mpsc::Receiver<Message>>>,
    pub client_id: Arc<Mutex<Option<Uuid>>>,
    _connection_handle: Arc<tokio::task::JoinHandle<()>>,
}

impl Client {
    pub fn connect(address: &str) -> Result<Self> {
        let rt = tokio::runtime::Handle::current();
        let address = address.to_string();

        let (outgoing_tx, outgoing_rx) = mpsc::channel::<String>(100);
        let (incoming_tx, incoming_rx) = mpsc::channel::<Message>(100);

        let client_id = Arc::new(Mutex::new(None));
        let client_id_clone = client_id.clone();

        let connection_handle = rt.spawn(async move {
            if let Err(e) =
                Self::run_connection(&address, outgoing_rx, incoming_tx, client_id_clone).await
            {
                eprintln!("Connection error: {}", e);
            }
        });

        Ok(Client {
            client_id,
            message_sender: Arc::new(Mutex::new(outgoing_tx)),
            message_receiver: Arc::new(Mutex::new(incoming_rx)),
            _connection_handle: Arc::new(connection_handle),
        })
    }

    pub async fn send_message(&self, message: &str) -> Result<()> {
        self.message_sender
            .lock()
            .await
            .send(message.to_string())
            .await
            .with_context(|| "Failed to send message")
    }

    async fn run_connection(
        address: &str,
        outgoing_rx: mpsc::Receiver<String>,
        incoming_tx: mpsc::Sender<Message>,
        client_id: Arc<Mutex<Option<Uuid>>>,
    ) -> Result<()> {
        let stream = TcpStream::connect(address)
            .await
            .with_context(|| format!("Failed to connect to {}", address))?;

        let (read_stream, write_stream) = tokio::io::split(stream);

        // Spawn task to handle outgoing messages
        let outgoing_task = Self::spawn_outgoing_handler(write_stream, outgoing_rx);

        // Spawn task to handle incoming messages
        let incoming_task = Self::spawn_incoming_handler(read_stream, incoming_tx, client_id);

        // Wait for either task to complete
        tokio::select! {
            _ = incoming_task => {},
            _ = outgoing_task => {},
        }

        Ok(())
    }

    fn spawn_incoming_handler(
        read_stream: tokio::io::ReadHalf<TcpStream>,
        incoming_tx: mpsc::Sender<Message>,
        client_id: Arc<Mutex<Option<Uuid>>>,
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
                        if let Ok(message) = serde_json::from_str::<Message>(&message_text) {
                            if message.client_id == Uuid::max() {
                                if let Ok(new_id) = Uuid::from_str(&message.content) {
                                    *client_id.lock().await = Some(new_id);
                                }
                            } else {
                                incoming_tx.send(message).await.ok();
                            }
                        }
                    }
                    Err(_) => break, // Connection error
                }
            }
        })
    }

    fn spawn_outgoing_handler(
        mut write_stream: tokio::io::WriteHalf<TcpStream>,
        mut outgoing_rx: mpsc::Receiver<String>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(message) = outgoing_rx.recv().await {
                if Self::send_message_to_server(&mut write_stream, &message)
                    .await
                    .is_err()
                {
                    break;
                }
            }
        })
    }

    async fn send_message_to_server(
        writer: &mut tokio::io::WriteHalf<TcpStream>,
        message: &String,
    ) -> Result<()> {
        let message_with_newline = format!("{}\n", message);
        writer.write_all(message_with_newline.as_bytes()).await?;
        writer.flush().await?;
        Ok(())
    }
}
