use gpui::*;
use super::model::AppState;
use super::input::TextInput;

pub struct ToolbarView {
    state: Entity<AppState>,
    hide_input: Entity<TextInput>,
    filter_input: Entity<TextInput>,
    highlight_input: Entity<TextInput>,
    line_start_input: Entity<TextInput>,
}

impl ToolbarView {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let hide_input = cx.new(|cx| {
            let state_clone = state.clone();
            TextInput::new(cx, "Hide...".to_string())
                .with_text(state.read(cx).hide_text.clone())
                .with_on_change(move |text, _window, cx| {
                    state_clone.update(cx, |state: &mut AppState, cx| {
                        state.hide_text = text.to_string();
                        state.apply_hide(cx);
                    });
                })
        });

        let filter_input = cx.new(|cx| {
            let state_clone = state.clone();
            TextInput::new(cx, "Filter...".to_string())
                .with_text(state.read(cx).filter_text.clone())
                .with_on_change(move |text, _window, cx| {
                    state_clone.update(cx, |state: &mut AppState, cx| {
                        state.filter_text = text.to_string();
                        state.apply_filter(cx);
                    });
                })
        });

        let highlight_input = cx.new(|cx| {
            let state_clone = state.clone();
            TextInput::new(cx, "Highlight...".to_string())
                .with_text(state.read(cx).highlight_text.clone())
                .with_on_change(move |text, _window, cx| {
                    state_clone.update(cx, |state: &mut AppState, cx| {
                        state.highlight_text = text.to_string();
                        state.apply_highlight(cx);
                    });
                })
        });

        let line_start_input = cx.new(|cx| {
            let state_clone = state.clone();
            TextInput::new(cx, "Line Start Regex...".to_string())
                .with_text(state.read(cx).line_start_text.clone())
                .with_on_change(move |text, _window, cx| {
                    state_clone.update(cx, |state: &mut AppState, cx| {
                        state.line_start_text = text.to_string();
                        state.save_state(); // Line start doesn't have apply method in model, just save
                        cx.notify();
                    });
                })
        });

        cx.observe(&state, |this: &mut Self, state: Entity<AppState>, cx| {
             let (hide_text, filter_text, highlight_text, line_start_text) = {
                 let state = state.read(cx);
                 (state.hide_text.clone(), state.filter_text.clone(), state.highlight_text.clone(), state.line_start_text.clone())
             };

             this.hide_input.update(cx, |input, cx| {
                 if input.content.as_ref() != hide_text {
                    input.content = hide_text.clone().into();
                    cx.notify();
                 }
             });
             this.filter_input.update(cx, |input, cx| {
                 if input.content.as_ref() != filter_text {
                    input.content = filter_text.clone().into();
                    cx.notify();
                 }
             });
             this.highlight_input.update(cx, |input, cx| {
                 if input.content.as_ref() != highlight_text {
                    input.content = highlight_text.clone().into();
                    cx.notify();
                 }
             });
             this.line_start_input.update(cx, |input, cx| {
                 if input.content.as_ref() != line_start_text {
                    input.content = line_start_text.clone().into();
                    cx.notify();
                 }
             });
             cx.notify();
        }).detach();

        Self {
            state,
            hide_input,
            filter_input,
            highlight_input,
            line_start_input,
        }
    }

    pub fn hide_focus_handle(&self, cx: &App) -> FocusHandle {
        self.hide_input.read(cx).focus_handle.clone()
    }

    pub fn filter_focus_handle(&self, cx: &App) -> FocusHandle {
        self.filter_input.read(cx).focus_handle.clone()
    }

    pub fn highlight_focus_handle(&self, cx: &App) -> FocusHandle {
        self.highlight_input.read(cx).focus_handle.clone()
    }

    pub fn line_start_focus_handle(&self, cx: &App) -> FocusHandle {
        self.line_start_input.read(cx).focus_handle.clone()
    }
}

impl Render for ToolbarView {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let show_time = state.show_time;
        let wrap_lines = state.wrap_lines;
        let follow_tail = state.follow_tail;
        let is_popup_open = state.is_listen_popup_open;

        let input_row = |label: &str, input: Entity<TextInput>, width: Length| {
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1()
                .child(div().child(label.to_string()).flex_none().text_sm())
                .child(
                    div()
                        .w(width)
                        .flex_none()
                        .border_1()
                        .border_color(rgb(0xcccccc))
                        .rounded_sm()
                        .child(input),
                )
        };

        let button_style = |label: &str, id: &str, active: bool| {
            div()
                .id(id.to_string())
                .child(label.to_string())
                .text_sm()
                .flex()
                .items_center()
                .justify_center()
                .px_2()
                .py_1()
                .bg(if active {
                    rgb(0xdddddd)
                } else {
                    rgb(0xffffff)
                })
                .border_1()
                .border_color(rgb(0xcccccc))
                .rounded_md()
                .cursor_pointer()
                .hover(|s| s.bg(rgb(0xe0e0e0)))
                .text_color(rgb(0x000000))
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_4()
            .p_2()
            .bg(rgb(0xf0f0f0))
            .border_b_1()
            .border_color(rgb(0xcccccc))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(input_row("Hide", self.hide_input.clone(), px(100.).into()))
                    .child(input_row("Filter", self.filter_input.clone(), px(200.).into()))
                    .child(input_row("Highlight", self.highlight_input.clone(), px(120.).into()))
                    .child(input_row("Line Start", self.line_start_input.clone(), px(150.).into())),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        button_style("Listen", "listen-btn", is_popup_open).on_click(cx.listener(
                            |this, _, _window, cx| {
                                this.state.update(cx, |state, cx| {
                                    state.toggle_listen_popup(cx);
                                });
                            },
                        )),
                    )
                    .child(
                        button_style("Time", "time-btn", show_time).on_click(cx.listener(
                            |this, _, _window, cx| {
                                this.state.update(cx, |state, cx| {
                                    state.toggle_time(cx);
                                });
                            },
                        )),
                    )
                    .child(
                        button_style("Wrap", "wrap-btn", wrap_lines)
                            .opacity(0.5)
                            .cursor_default() // No pointer cursor
                            // Disabled action
                            // .on_click(cx.listener(
                            //     |this, _, _window, cx| {
                            //         this.state.update(cx, |state, cx| {
                            //             state.toggle_wrap(cx);
                            //         });
                            //     },
                            // ))
                    )
                    .child(
                        button_style("Follow", "follow-btn", follow_tail).on_click(cx.listener(
                            |this, _, _window, cx| {
                                this.state.update(cx, |state, cx| {
                                    state.toggle_follow_tail(cx);
                                });
                            },
                        )),
                    )
                    .child(button_style("Clear", "clear-btn", false).on_click(cx.listener(
                        |this, _, _window, cx| {
                            this.state.update(cx, |state, cx| {
                                state.clear(cx);
                            });
                        },
                    ))),
            )
    }
}
