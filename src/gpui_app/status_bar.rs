use gpui::*;
use super::model::AppState;

pub struct StatusBarView {
    model: Entity<AppState>,
}

impl StatusBarView {
    pub fn new(model: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        cx.observe(&model, |_, _, cx| cx.notify()).detach();
        Self { model }
    }
}

impl Render for StatusBarView {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let model = self.model.read(cx);
        let total_lines = model.lines.len();
        let filtered_lines = model.filtered_indices.len();
        let following = model.follow_tail;
        let status = &model.connection_status;

        let mut row = div()
            .flex()
            .gap_2()
            .child(format!("Status: {}", status));

        if following {
            row = row.child("Following");
        }

        div()
            .flex()
            .flex_row()
            .justify_between()
            .w_full()
            .bg(rgb(0xeeeeee))
            .px_2()
            .py_1()
            .text_xs()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(format!("Lines: {} / {}", filtered_lines, total_lines))
            )
            .child(row)
    }
}
