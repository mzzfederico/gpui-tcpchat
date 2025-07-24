use anyhow::Result;
use client_lib::Client;

mod input;
mod message;
mod view;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::connect("127.0.0.1:8080")?;

    view::spawn_application(client);
    Ok(())
}
