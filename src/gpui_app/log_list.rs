use gpui::*;
use gpui::ScrollStrategy;
use crate::gpui_app::model::AppState;
use std::cell::RefCell;
use std::rc::Rc;
use std::ops::Range;

actions!(log_list, [PageUp, PageDown, Home, End]);

pub struct LogListView {
    model: Entity<AppState>,
    scroll_handle: UniformListScrollHandle,
    focus_handle: FocusHandle,
    visible_range: Rc<RefCell<Range<usize>>>,
}

impl LogListView {
    pub fn new(model: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        Self {
            model,
            scroll_handle: UniformListScrollHandle::new(),
            focus_handle: cx.focus_handle(),
            visible_range: Rc::new(RefCell::new(0..0)),
        }
    }

    pub fn page_up(&mut self, _: &PageUp, _window: &mut Window, cx: &mut Context<Self>) {
        self.perform_page_up(cx);
    }

    pub fn perform_page_up(&mut self, cx: &mut Context<Self>) {
        let range = self.visible_range.borrow().clone();
        let page_size = range.end - range.start;
        self.scroll_handle.scroll_to_item(range.start.saturating_sub(page_size), ScrollStrategy::Top);
        self.model.update(cx, |state, cx| {
            if state.follow_tail {
                state.follow_tail = false;
                cx.notify();
            }
        });
    }

    pub fn page_down(&mut self, _: &PageDown, _window: &mut Window, cx: &mut Context<Self>) {
        self.perform_page_down(cx);
    }

    pub fn perform_page_down(&mut self, cx: &mut Context<Self>) {
        let range = self.visible_range.borrow().clone();
        let page_size = range.end - range.start;
        self.scroll_handle.scroll_to_item(range.start + page_size, ScrollStrategy::Top);
        self.model.update(cx, |state, cx| {
            if state.follow_tail {
                state.follow_tail = false;
                cx.notify();
            }
        });
    }

    pub fn home(&mut self, _: &Home, _window: &mut Window, cx: &mut Context<Self>) {
        self.perform_home(cx);
    }

    pub fn perform_home(&mut self, cx: &mut Context<Self>) {
        self.model.update(cx, |state, cx| {
            if state.follow_tail {
                state.follow_tail = false;
                cx.notify();
            }
        });
        self.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
        cx.notify();
    }

    pub fn end(&mut self, _: &End, _window: &mut Window, cx: &mut Context<Self>) {
        self.perform_end(cx);
    }

    pub fn perform_end(&mut self, cx: &mut Context<Self>) {
        let count = self.model.read(cx).filtered_indices.len();
        if count > 0 {
             self.scroll_handle.scroll_to_item(count - 1, ScrollStrategy::Bottom);
        }
        // Enable follow tail? User usually wants to follow tail if they go to end.
        self.model.update(cx, |state, cx| {
            if !state.follow_tail {
                state.follow_tail = true;
                cx.notify();
            }
        });
        cx.notify();
    }
}

impl Focusable for LogListView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for LogListView {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let model_read = self.model.read(cx);
        let count = model_read.filtered_indices.len();
        let follow_tail = model_read.follow_tail;
        let show_time = model_read.show_time;
        let _wrap_lines = model_read.wrap_lines;
        // Clone highlight_expr to move into closure
        let highlight_expr = model_read.filter_state.highlight_expr.clone();

        // Handle follow tail
        if follow_tail && count > 0 {
             self.scroll_handle.scroll_to_item(count - 1, ScrollStrategy::Center);
        }

        let list = uniform_list(
            "log-list",
            count,
            {
                let model = self.model.clone();
                let visible_range = self.visible_range.clone();
                move |range, _window, cx| {
                    *visible_range.borrow_mut() = range.clone();
                    let model = model.read(cx);
                    let mut items = Vec::with_capacity(range.end - range.start);
                    
                    for i in range {
                        if let Some(&line_idx) = model.filtered_indices.get(i) {
                            if let Some(line) = model.lines.get(line_idx) {
                                let mut row = div()
                                    .flex()
                                    .flex_row()
                                    .w_full()
                                    .px_4() // Increased padding
                                    .py_0p5() // Add vertical padding
                                    .text_sm()
                                    .text_color(rgb(0x000000));

                                // Alternating background
                                if i % 2 == 1 {
                                    row = row.bg(rgb(0xf7f7f7));
                                } else {
                                    row = row.bg(rgb(0xffffff));
                                }
                                
                                // Selection/Focus indication could be added here

                                if show_time {
                                    // Use simple formatting or chrono
                                    let time_str = line.timestamp.format("%H:%M:%S.%3f").to_string();
                                    row = row.child(
                                        div()
                                            .w(px(100.0))
                                            .flex_shrink_0()
                                            .text_color(rgb(0x888888))
                                            .child(time_str)
                                    );
                                }

                                let display_content = model.get_display_content(line);
                                let content_element = if let Some(expr) = &highlight_expr {
                                    let matches: Vec<(usize, usize)> = expr.find_all_matches(&display_content);
                                    if matches.is_empty() {
                                        // Force nowrap for uniform_list compatibility
                                        div().child(display_content.clone()).whitespace_nowrap()
                                    } else {
                                        let mut container = div().flex().flex_row();
                                        // Force nowrap for uniform_list compatibility
                                        container = container.whitespace_nowrap();
                                        
                                        let mut last_end = 0;
                                        for (start, end) in matches {
                                            if start > last_end {
                                                container = container.child(
                                                    div().child(display_content[last_end..start].to_string())
                                                );
                                            }
                                            container = container.child(
                                                div()
                                                    .bg(rgb(0xffeb3b))
                                                    .text_color(rgb(0x000000))
                                                    .child(display_content[start..end].to_string())
                                            );
                                            last_end = end;
                                        }
                                        if last_end < display_content.len() {
                                            container = container.child(
                                                div().child(display_content[last_end..].to_string())
                                            );
                                        }
                                        container
                                    }
                                } else {
                                    // Force nowrap for uniform_list compatibility
                                    div().child(display_content.clone()).whitespace_nowrap()
                                };

                                items.push(row.child(content_element));
                            }
                        }
                    }
                    items
                }
            }
        )
        .track_scroll(&self.scroll_handle.clone())
        .size_full();

        div()
            .size_full()
            .bg(rgb(0xffffff))
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(Self::page_up))
            .on_action(cx.listener(Self::page_down))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_scroll_wheel({
                let model = self.model.clone();
                move |_, _, cx| {
                    model.update(cx, |state, cx| {
                        if state.follow_tail {
                            state.follow_tail = false;
                            cx.notify();
                        }
                    })
                }
            })
            .child(list)
    }
}
