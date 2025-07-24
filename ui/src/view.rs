use client_lib::Client;
use gpui::*;

use crate::{input::*};
use crate::chat_message::{ChatMessage, MessageType};
use crate::list::ChatList;

pub struct MainView {
    input: Entity<TextInput>,
    messages: Entity<Vec<ChatMessage>>,
    list: Entity<ChatList>,
    client: Client,
}

impl MainView {
    pub fn new(app: &mut App, messages: Entity<Vec<ChatMessage>>, client: Client) -> Self {
        app.bind_keys([
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("delete", Delete, None),
            KeyBinding::new("left", Left, None),
            KeyBinding::new("right", Right, None),
            KeyBinding::new("shift-left", SelectLeft, None),
            KeyBinding::new("shift-right", SelectRight, None),
            KeyBinding::new("cmd-a", SelectAll, None),
            KeyBinding::new("cmd-v", Paste, None),
            KeyBinding::new("cmd-c", Copy, None),
            KeyBinding::new("cmd-x", Cut, None),
            KeyBinding::new("home", Home, None),
            KeyBinding::new("end", End, None),
            KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, None),
        ]);

        let input = app.new(|cx| TextInput {
            focus_handle: cx.focus_handle(),
            content: "".into(),
            placeholder: "Type here...".into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
        });

        let messages_clone = messages.clone();
        let list = app.new(|_| ChatList {
            messages: messages_clone,
        });

        Self {
            input,
            messages,
            list,
            client
        }
    }

    pub fn handle_send_button(&mut self, _: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let message = self.input.read(cx).content.clone();
        if !message.trim().is_empty() {
            let message_str = message.as_ref();

            self.messages.update(cx, |e, _cx| {
                e.push(ChatMessage::new(message_str, MessageType::User));
            });

            self.input.update(cx, |input, _cx| {
                input.content = "".into();
                input.selected_range = 0..0;
            });

            cx.notify();

            let client = self.client.clone();
            let message_to_send = message_str.to_string();
            cx.spawn(async move |_, _| {
                let _ = client.send_message(&message_to_send).await;
            }).detach();
        }
    }
}

impl Render for MainView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .h_full()
            .w_full()
            .bg(rgb(0x2f3136))
            .child(
                // Chat header
                div()
                    .flex()
                    .items_center()
                    .px_4()
                    .py_3()
                    .bg(rgb(0x36393f))
                    .border_b_1()
                    .border_color(rgb(0x202225))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(white())
                            .child("Chat")
                    )
            )
            .child(
                // Messages area
                self.list.clone()
            )
            .child(
                // Input area
                div()
                    .flex()
                    .flex_row()
                    .gap_3()
                    .p_4()
                    .bg(rgb(0x36393f))
                    .border_t_1()
                    .border_color(rgb(0x202225))
                    .child(
                        div()
                            .flex_1()
                            .child(self.input.clone())
                    )
                    .child(
                        div()
                            .px_4()
                            .py_2()
                            .bg(rgb(0x5865f2))
                            .hover(|style| style.bg(rgb(0x4752c4)))
                            .rounded_md()
                            .cursor_pointer()
                            .text_color(white())
                            .font_weight(FontWeight::MEDIUM)
                            .child("Send")
                            .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_send_button))
                    )
            )
    }
}
