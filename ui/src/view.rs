use client_lib::Client;
use gpui::*;
use crate::input::*;
use crate::message::{ChatMessage, MessageList, MessageType};


struct ChatApplication {
    client: Client,
    text_input: Entity<TextInput>,
    message_list: Entity<MessageList>,

}

impl ChatApplication {
    pub fn new(client: Client, input: Entity<TextInput>, message_list: Entity<MessageList>) -> Self {
        Self {
            client,
            text_input: input,
            message_list,
        }
    }

    pub fn handle_send_button(&mut self, _: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let message = self.text_input.read(cx).content.clone();
        if !message.trim().is_empty() {
            let message_str = message.as_ref();

            self.message_list.update(cx, |list, _cx| {
                list.add_message(ChatMessage::new(message_str, MessageType::User));
            });

            let client = self.client.clone();
            let message_to_send = message_str.to_string();
            tokio::spawn(async move {
                let _ = client.send_message(&message_to_send).await;
            });

            self.text_input.update(cx, |input, _cx| {
                input.content = "".into();
                input.selected_range = 0..0;
            });
        }
    }

    fn check_for_messages(&mut self, cx: &mut Context<Self>) {
        println!("Checking for messages...");

        // Clone what you need from self before moving into the closure
        let client = self.client.clone();
        let message_list = self.message_list.clone();

        cx.spawn(|_weak_self, _| async move {
            while let Some(message) = client.receive_message().await {
                cx.update(|cx| {
                    message_list.update(cx, |list, _cx| {
                        list.add_message(ChatMessage::new(message.content, MessageType::User));
                    });
                });
            }
        }).detach();
    }
}

impl Render for ChatApplication {
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
                self.message_list.clone()
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
                            .child(self.text_input.clone())
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

pub fn spawn_application(client: Client) {
    Application::new().run(|cx: &mut App| {
        cx.bind_keys([
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

        let text_input = cx.new(|cx| TextInput {
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

        let message_list = cx.new(|_| MessageList::new());

        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                let app = cx.new(|_| ChatApplication::new(client, text_input, message_list));
                app.update(cx, |app, cx| {
                    app.check_for_messages(cx);
                });
                app
            },
        )
        .unwrap();

        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
    });
}
