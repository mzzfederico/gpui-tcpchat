use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;
use crate::chat::ChatInstance;

mod messages;
mod chat;

#[tokio::main]
async fn main() -> Result<()> {
    let chat = Arc::new(ChatInstance::new());
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    println!("Chat server listening on 127.0.0.1:8080");

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                println!("New client connected from: {}", addr);
                let chat = Arc::clone(&chat);
                tokio::spawn(async move {
                    if let Err(e) = chat.handle_connection(stream).await {
                        eprintln!("Error handling client {}: {}", addr, e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
