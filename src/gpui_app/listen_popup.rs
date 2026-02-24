use gpui::*;
use super::model::AppState;
use super::input::TextInput;

pub struct ListenPopup {
    state: Entity<AppState>,
    port_input: Entity<TextInput>,
}

impl ListenPopup {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let port_input = cx.new(|cx| {
            TextInput::new(cx, "Port (8080)".to_string())
                .with_text("8080".to_string())
        });

        Self {
            state,
            port_input,
        }
    }

    fn confirm(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let port_text = self.port_input.read(cx).content.to_string();
        let port = port_text.parse::<u16>().unwrap_or(8080);
        
        self.state.update(cx, |state, cx| {
            state.start_listening(port, cx);
        });
    }

    fn cancel(&mut self, _: &ClickEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.toggle_listen_popup(cx);
        });
    }
}

impl Render for ListenPopup {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .flex()
            .flex_col()
            .size_full()
            .justify_center()
            .items_center()
            .bg(rgba(0x00000088))
            .child(
                div()
                    .w_96()
                    .bg(white())
                    .rounded_md()
                    .shadow_lg()
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .text_lg()
                            .child("Listen on Port")
                    )
                    .child(self.port_input.clone())
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap_2()
                            .child(
                                div()
                                    .id("cancel-btn")
                                    .child("Cancel")
                                    .bg(rgb(0xeeeeee))
                                    .p_2()
                                    .rounded_md()
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, event, window, cx| {
                                        this.cancel(event, window, cx)
                                    }))
                            )
                            .child(
                                div()
                                    .id("confirm-btn")
                                    .child("Listen")
                                    .bg(rgb(0x4488ff))
                                    .text_color(white())
                                    .p_2()
                                    .rounded_md()
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, event, window, cx| {
                                        this.confirm(event, window, cx)
                                    }))
                            )
                    )
            )
    }
}
