use gpui::*;

use crate::chat_message::ChatMessage;

pub struct ChatList {
    pub messages: Entity<Vec<ChatMessage>>
}

impl Render for ChatList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let message_entities = self.messages.read(cx).clone();

        div()
            .flex()
            .flex_col()
            .h_full()
            .w_full()
            .p_3()
            .children(
                message_entities
                    .iter()
                    .map(|message| {
                        cx.new(|_| message.clone())
                    })
                    .collect::<Vec<_>>()
            )
    }
}
