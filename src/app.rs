use crate::filter::{parse_filter, FilterExpr};
use crate::highlight::{apply_highlights, highlight_line};
use crate::source::SourceEvent;
use chrono::Local;
use std::sync::mpsc::Receiver;

#[derive(Clone)]
pub struct LogLine {
    pub timestamp: String,
    pub content: String,
}

pub struct App {
    pub lines: Vec<LogLine>,
    pub filtered_indices: Vec<usize>,
    pub scroll_offset: usize,
    pub filter_input: String,
    pub filter_expr: Option<FilterExpr>,
    pub filter_error: Option<String>,
    pub highlight_input: String,
    pub highlight_expr: Option<FilterExpr>,
    pub highlight_error: Option<String>,
    pub show_time: bool,
    pub heuristic_highlight: bool,
    pub wrap_lines: bool,
    pub input_mode: InputMode,
    pub follow_tail: bool,
    pub source_rx: Receiver<SourceEvent>,
    pub status_message: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    FilterEdit,
    HighlightEdit,
}

impl App {
    pub fn new(source_rx: Receiver<SourceEvent>) -> Self {
        Self {
            lines: Vec::new(),
            filtered_indices: Vec::new(),
            scroll_offset: 0,
            filter_input: String::new(),
            filter_expr: None,
            filter_error: None,
            highlight_input: String::new(),
            highlight_expr: None,
            highlight_error: None,
            show_time: true,
            heuristic_highlight: true,
            wrap_lines: false,
            input_mode: InputMode::Normal,
            follow_tail: true,
            source_rx,
            status_message: None,
        }
    }

    pub fn poll_source(&mut self) {
        while let Ok(event) = self.source_rx.try_recv() {
            match event {
                SourceEvent::Line(content) => {
                    let line = LogLine {
                        timestamp: Local::now().format("%H:%M:%S").to_string(),
                        content,
                    };
                    let idx = self.lines.len();
                    self.lines.push(line);
                    if self.matches_filter(idx) {
                        self.filtered_indices.push(idx);
                    }
                }
                SourceEvent::Error(e) => {
                    self.status_message = Some(format!("Source error: {}", e));
                }
            }
        }
    }

    fn matches_filter(&self, idx: usize) -> bool {
        match &self.filter_expr {
            Some(expr) => expr.matches(&self.lines[idx].content),
            None => true,
        }
    }

    pub fn apply_filter(&mut self) {
        if self.filter_input.trim().is_empty() {
            self.filter_expr = None;
            self.filter_error = None;
        } else {
            match parse_filter(&self.filter_input) {
                Ok(expr) => {
                    self.filter_expr = Some(expr);
                    self.filter_error = None;
                }
                Err(e) => {
                    self.filter_error = Some(e.to_string());
                    return;
                }
            }
        }
        self.rebuild_filtered_indices();
    }

    pub fn apply_highlight(&mut self) {
        if self.highlight_input.trim().is_empty() {
            self.highlight_expr = None;
            self.highlight_error = None;
        } else {
            match parse_filter(&self.highlight_input) {
                Ok(expr) => {
                    self.highlight_expr = Some(expr);
                    self.highlight_error = None;
                }
                Err(e) => {
                    self.highlight_error = Some(e.to_string());
                }
            }
        }
    }

    fn rebuild_filtered_indices(&mut self) {
        self.filtered_indices.clear();
        for i in 0..self.lines.len() {
            if self.matches_filter(i) {
                self.filtered_indices.push(i);
            }
        }
        self.scroll_offset = 0;
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.filtered_indices.clear();
        self.scroll_offset = 0;
        self.status_message = Some("Cleared".to_string());
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
        self.follow_tail = false;
    }

    pub fn scroll_down(&mut self, amount: usize, visible_height: usize) {
        let max_offset = self.filtered_indices.len().saturating_sub(visible_height);
        self.scroll_offset = (self.scroll_offset + amount).min(max_offset);
        if self.scroll_offset >= max_offset {
            self.follow_tail = true;
        }
    }

    pub fn scroll_to_end(&mut self, visible_height: usize) {
        let max_offset = self.filtered_indices.len().saturating_sub(visible_height);
        self.scroll_offset = max_offset;
        self.follow_tail = true;
    }

    pub fn scroll_to_start(&mut self) {
        self.scroll_offset = 0;
        self.follow_tail = false;
    }

    pub fn get_visible_lines(&self, height: usize) -> Vec<(usize, &LogLine)> {
        if self.follow_tail {
            let start = self.filtered_indices.len().saturating_sub(height);
            self.filtered_indices[start..]
                .iter()
                .map(|&i| (i, &self.lines[i]))
                .collect()
        } else {
            self.filtered_indices
                .iter()
                .skip(self.scroll_offset)
                .take(height)
                .map(|&i| (i, &self.lines[i]))
                .collect()
        }
    }

    pub fn render_line(&self, line: &LogLine) -> Vec<(String, ratatui::style::Style)> {
        let spans = highlight_line(
            &line.content,
            self.highlight_expr.as_ref(),
            self.heuristic_highlight,
        );
        apply_highlights(&line.content, &spans)
    }

    pub fn toggle_time(&mut self) {
        self.show_time = !self.show_time;
    }

    pub fn toggle_heuristic(&mut self) {
        self.heuristic_highlight = !self.heuristic_highlight;
    }

    pub fn toggle_wrap(&mut self) {
        self.wrap_lines = !self.wrap_lines;
    }
}
