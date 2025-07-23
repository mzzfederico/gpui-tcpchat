use anyhow::Result;
use client_lib::Client;
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Connecting to server...");
    let client = Client::connect("127.0.0.1:8080")?;
    println!("Connected! Type messages and press Enter to send them.");
    println!("Type 'quit' to exit.\n");

    // Spawn a background task to continuously listen for messages
    let client_clone = client.clone();
    let _listener_task = tokio::spawn(async move {
        loop {
            if let Some(response) = client_clone.try_receive_message() {
                println!("< Client {}: {}", response.client_id, response.content);
                print!("> ");
                io::stdout().flush().unwrap();
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });

    // Handle user input
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                let message = line.trim();

                if message.is_empty() {
                    continue;
                }

                if message == "quit" {
                    println!("Goodbye!");
                    break;
                }

                if let Err(e) = client.send_message_blocking(message) {
                    eprintln!("Failed to send message: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }

    Ok(())
}
