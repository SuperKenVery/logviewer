use gpui::*;
use gpui::prelude::FluentBuilder;
use super::model::AppState;
use super::toolbar::ToolbarView;
use super::log_list::{LogListView, PageUp, PageDown, Home, End};
use super::status_bar::StatusBarView;
use super::listen_popup::ListenPopup;

actions!(main_view, [FocusFilter, FocusHide, FocusHighlight, FocusLineStart, ToggleListen, ToggleTime, ToggleWrap, ToggleFollowTail, Clear, FocusList]);

pub struct MainView {
    state: Entity<AppState>,
    toolbar: Entity<ToolbarView>,
    log_list: Entity<LogListView>,
    status_bar: Entity<StatusBarView>,
    listen_popup: Entity<ListenPopup>,
}

impl MainView {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let toolbar = cx.new(|cx| ToolbarView::new(state.clone(), cx));
        let log_list = cx.new(|cx| LogListView::new(state.clone(), cx));
        let status_bar = cx.new(|cx| StatusBarView::new(state.clone(), cx));
        let listen_popup = cx.new(|cx| ListenPopup::new(state.clone(), cx));

        cx.observe(&state, |_, _, cx| cx.notify()).detach();

        Self {
            state,
            toolbar,
            log_list,
            status_bar,
            listen_popup,
        }
    }

    fn focus_filter(&mut self, _: &FocusFilter, window: &mut Window, cx: &mut Context<Self>) {
        let handle = self.toolbar.read(cx).filter_focus_handle(cx);
        window.focus(&handle, cx);
    }

    fn focus_hide(&mut self, _: &FocusHide, window: &mut Window, cx: &mut Context<Self>) {
        let handle = self.toolbar.read(cx).hide_focus_handle(cx);
        window.focus(&handle, cx);
    }

    fn focus_highlight(&mut self, _: &FocusHighlight, window: &mut Window, cx: &mut Context<Self>) {
        let handle = self.toolbar.read(cx).highlight_focus_handle(cx);
        window.focus(&handle, cx);
    }

    fn focus_line_start(&mut self, _: &FocusLineStart, window: &mut Window, cx: &mut Context<Self>) {
        let handle = self.toolbar.read(cx).line_start_focus_handle(cx);
        window.focus(&handle, cx);
    }

    fn focus_list(&mut self, _: &FocusList, window: &mut Window, cx: &mut Context<Self>) {
        let handle = self.log_list.read(cx).focus_handle(cx);
        window.focus(&handle, cx);
    }

    fn toggle_listen(&mut self, _: &ToggleListen, _: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| state.toggle_listen_popup(cx));
    }

    fn toggle_time(&mut self, _: &ToggleTime, _: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| state.toggle_time(cx));
    }

    fn toggle_wrap(&mut self, _: &ToggleWrap, _: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| state.toggle_wrap(cx));
    }

    fn toggle_follow_tail(&mut self, _: &ToggleFollowTail, _: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| state.toggle_follow_tail(cx));
    }

    fn clear(&mut self, _: &Clear, _: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| state.clear(cx));
    }
}

impl Render for MainView {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let is_popup_open = state.is_listen_popup_open;

        div()
            .flex()
            .flex_col()
            .size_full()
            .on_action(cx.listener(Self::focus_filter))
            .on_action(cx.listener(Self::focus_hide))
            .on_action(cx.listener(Self::focus_highlight))
            .on_action(cx.listener(Self::focus_line_start))
            .on_action(cx.listener(Self::focus_list))
            .on_action(cx.listener(Self::toggle_listen))
            .on_action(cx.listener(Self::toggle_time))
            .on_action(cx.listener(Self::toggle_wrap))
            .on_action(cx.listener(Self::toggle_follow_tail))
            .on_action(cx.listener(Self::clear))
            .on_action(cx.listener(|this, _: &PageUp, _window, cx| {
                this.log_list.update(cx, |view: &mut LogListView, cx| view.perform_page_up(cx));
            }))
            .on_action(cx.listener(|this, _: &PageDown, _window, cx| {
                this.log_list.update(cx, |view: &mut LogListView, cx| view.perform_page_down(cx));
            }))
            .on_action(cx.listener(|this, _: &Home, _window, cx| {
                this.log_list.update(cx, |view: &mut LogListView, cx| view.perform_home(cx));
            }))
            .on_action(cx.listener(|this, _: &End, _window, cx| {
                this.log_list.update(cx, |view: &mut LogListView, cx| view.perform_end(cx));
            }))
            .child(self.toolbar.clone())
            .child(div().flex_grow().child(self.log_list.clone()))
            .child(self.status_bar.clone())
            .when(is_popup_open, |div: Div| div.child(self.listen_popup.clone()))
    }
}
