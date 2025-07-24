use gpui::*;

#[derive(Clone, Debug)]
pub enum MessageType {
    User,
    Other,
}

#[derive(Clone)]
pub struct ChatMessage {
    pub content: String,
    pub message_type: MessageType,
}

impl ChatMessage {
    pub fn new(content: impl Into<String>, message_type: MessageType) -> Self {
        Self { content: content.into(), message_type }
    }
}

impl Render for ChatMessage {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let (bg_color, text_color, alignment) = match self.message_type {
            MessageType::User => (rgb(0x5865f2), rgb(0xffffff), FlexDirection::RowReverse),
            MessageType::Other => (rgb(0x40444b), rgb(0xdcddde), FlexDirection::Row),
        };

        let mut container = div()
            .flex()
            .mb_2();

        container = match alignment {
            FlexDirection::RowReverse => container.flex_row_reverse(),
            FlexDirection::Row => container.flex_row(),
            _ => container.flex_row(),
        };

        if matches!(self.message_type, MessageType::User) {
            container = container.justify_end();
        }

        container
            .child(
                div()
                    .p_3()
                    .bg(bg_color)
                    .rounded_lg()
                    .text_color(text_color)
                    .max_w_80()
                    .child(self.content.clone())
            )
    }
}
