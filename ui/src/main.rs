mod input;
mod chat_message;
mod view;
mod list;

use anyhow::Result;
use client_lib::Client;
use gpui::*;
use server::Message;
use tokio::sync::mpsc::{self, UnboundedSender};

use crate::{
    chat_message::{ChatMessage, MessageType},
    view::MainView,
};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::connect("127.0.0.1:8080")?;
    let client_clone = client.clone();

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let _ = spawn_message_handler(client, tx);

    Application::new().run(|cx: &mut App| {
        let messages_entity = cx.new(|_cx| Vec::new());

        let messages_entity_clone = messages_entity.clone();
        cx.spawn(async move |cx| {
            loop {
                if let Some(content) = rx.recv().await {
                    let _ = messages_entity_clone.update(cx, |entity, _| {
                        entity.push(ChatMessage::new(content, MessageType::Other));
                    });
                    let _ = cx.refresh();
                }
            }
        }).detach();

        let _ = cx.open_window(WindowOptions::default(), |_, app| {
            app.new(|app| MainView::new(app, messages_entity, client_clone))
        });
    });

    Ok(())
}

fn spawn_message_handler(
    client: Client,
    app_tx: UnboundedSender<String>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            if let Some(message) = client.message_receiver.lock().await.recv().await {
                match message {
                    Message::Chat { content, client_id: _, timestamp: _ } => {
                        let _ = app_tx.send(content);
                        continue;
                    }
                    _ => continue
                }
            }
        }
    })
}
