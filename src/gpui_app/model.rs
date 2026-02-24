use gpui::*;
use crate::core::{LogLine, FilterState};
use crate::filter::parse_filter;
use crate::state::AppState as PersistentState;
use fancy_regex::Regex;
use std::sync::mpsc::Sender;
use crate::source::{SourceEvent, LogSource, start_source};
use std::sync::Arc;

pub struct AppState {
    pub lines: Vec<LogLine>,
    pub filtered_indices: Vec<usize>,
    pub filter_state: FilterState,
    pub follow_tail: bool,
    pub show_time: bool,
    pub wrap_lines: bool,
    pub hide_text: String,
    pub filter_text: String,
    pub highlight_text: String,
    pub line_start_text: String,
    pub hide_error: Option<String>,
    pub filter_error: Option<String>,
    pub highlight_error: Option<String>,
    pub connection_status: String,
    pub is_listen_popup_open: bool,
    pub source_tx: Option<Sender<SourceEvent>>,
}

impl AppState {
    pub fn new(cx: &mut impl gpui::AppContext, source_tx: Option<Sender<SourceEvent>>) -> Entity<Self> {
        let persistent_state = PersistentState::load();
        
        let mut model = Self {
            lines: Vec::new(),
            filtered_indices: Vec::new(),
            filter_state: FilterState::default(),
            follow_tail: true,
            show_time: true,
            wrap_lines: persistent_state.wrap_lines,
            hide_text: persistent_state.hide_input.clone(),
            filter_text: persistent_state.filter_input.clone(),
            highlight_text: persistent_state.highlight_input.clone(),
            line_start_text: persistent_state.line_start_regex.clone(),
            hide_error: None,
            filter_error: None,
            highlight_error: None,
            connection_status: "Disconnected".to_string(),
            is_listen_popup_open: false,
            source_tx,
        };

        if !model.hide_text.trim().is_empty() {
            if let Ok(re) = Regex::new(&model.hide_text) {
                model.filter_state.hide_regex = Some(re);
            }
        }

        if !model.filter_text.trim().is_empty() {
            if let Ok(expr) = parse_filter(&model.filter_text) {
                model.filter_state.filter_expr = Some(expr);
            }
        }

        if !model.highlight_text.trim().is_empty() {
            if let Ok(expr) = parse_filter(&model.highlight_text) {
                model.filter_state.highlight_expr = Some(expr);
            }
        }

        cx.new(|_cx| model)
    }

    pub fn add_line(&mut self, content: String, cx: &mut gpui::Context<Self>) {
        let now = chrono::Local::now();
        let line = LogLine {
            content: content.trim_end().to_string(),
            timestamp: now,
        };
        
        let matches = self.matches_filter(&line);
        self.lines.push(line);
        
        if matches {
            self.filtered_indices.push(self.lines.len() - 1);
        }
        cx.notify();
    }

    pub fn get_display_content(&self, line: &LogLine) -> String {
        self.filter_state.apply_hide(&line.content).unwrap_or_else(|_| line.content.clone())
    }

    fn matches_filter(&self, line: &LogLine) -> bool {
        let content = self.get_display_content(line);
        self.filter_state.matches_filter(&content)
    }

    fn rebuild_filtered_indices(&mut self) {
        self.filtered_indices.clear();
        for (i, line) in self.lines.iter().enumerate() {
            if self.matches_filter(line) {
                self.filtered_indices.push(i);
            }
        }
    }

    pub fn save_state(&self) {
        let state = PersistentState {
            hide_input: self.hide_text.clone(),
            filter_input: self.filter_text.clone(),
            highlight_input: self.highlight_text.clone(),
            wrap_lines: self.wrap_lines,
            line_start_regex: self.line_start_text.clone(),
        };
        state.save();
    }

    pub fn apply_filter(&mut self, cx: &mut gpui::Context<Self>) {
        if self.filter_text.trim().is_empty() {
            self.filter_state.filter_expr = None;
            self.filter_error = None;
        } else {
            match parse_filter(&self.filter_text) {
                Ok(expr) => {
                    self.filter_state.filter_expr = Some(expr);
                    self.filter_error = None;
                }
                Err(e) => {
                    self.filter_error = Some(e.to_string());
                }
            }
        }
        self.rebuild_filtered_indices();
        self.save_state();
        cx.notify();
    }

    pub fn apply_hide(&mut self, cx: &mut gpui::Context<Self>) {
        if self.hide_text.trim().is_empty() {
            self.filter_state.hide_regex = None;
            self.hide_error = None;
        } else {
            match Regex::new(&self.hide_text) {
                Ok(re) => {
                    self.filter_state.hide_regex = Some(re);
                    self.hide_error = None;
                }
                Err(e) => {
                    self.hide_error = Some(e.to_string());
                }
            }
        }
        self.rebuild_filtered_indices();
        self.save_state();
        cx.notify();
    }

    pub fn apply_highlight(&mut self, cx: &mut gpui::Context<Self>) {
        if self.highlight_text.trim().is_empty() {
            self.filter_state.highlight_expr = None;
            self.highlight_error = None;
        } else {
            match parse_filter(&self.highlight_text) {
                Ok(expr) => {
                    self.filter_state.highlight_expr = Some(expr);
                    self.highlight_error = None;
                }
                Err(e) => {
                    self.highlight_error = Some(e.to_string());
                }
            }
        }
        self.save_state();
        cx.notify();
    }

    pub fn toggle_time(&mut self, cx: &mut gpui::Context<Self>) {
        self.show_time = !self.show_time;
        cx.notify();
    }

    pub fn toggle_wrap(&mut self, cx: &mut gpui::Context<Self>) {
        self.wrap_lines = !self.wrap_lines;
        self.save_state();
        cx.notify();
    }

    pub fn toggle_follow_tail(&mut self, cx: &mut gpui::Context<Self>) {
        self.follow_tail = !self.follow_tail;
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut gpui::Context<Self>) {
        self.lines.clear();
        self.filtered_indices.clear();
        cx.notify();
    }

    pub fn toggle_listen_popup(&mut self, cx: &mut gpui::Context<Self>) {
        self.is_listen_popup_open = !self.is_listen_popup_open;
        cx.notify();
    }

    pub fn start_listening(&mut self, port: u16, cx: &mut gpui::Context<Self>) {
        if let Some(tx) = &self.source_tx {
            let regex = if self.line_start_text.trim().is_empty() {
                None
            } else {
                Regex::new(&self.line_start_text).ok().map(Arc::new)
            };

            if let Err(e) = start_source(LogSource::Network(port), tx.clone(), regex) {
                self.connection_status = format!("Failed to start: {}", e);
            } else {
                self.connection_status = format!("Listening on port {}", port);
            }
        }
        self.is_listen_popup_open = false;
        cx.notify();
    }
}
