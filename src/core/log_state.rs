use chrono::Local;

#[derive(Clone, PartialEq)]
pub struct LogLine {
    pub timestamp: String,
    pub content: String,
}

#[derive(Clone)]
pub struct LogState {
    pub lines: Vec<LogLine>,
    pub filtered_indices: Vec<usize>,
    pub bottom_line_idx: usize,
    pub follow_tail: bool,
}

impl Default for LogState {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            filtered_indices: Vec::new(),
            bottom_line_idx: 0,
            follow_tail: true,
        }
    }
}

impl LogState {
    pub fn add_line(&mut self, content: String) -> usize {
        let line = LogLine {
            timestamp: Local::now().format("%H:%M:%S").to_string(),
            content,
        };
        let idx = self.lines.len();
        self.lines.push(line);
        idx
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.filtered_indices.clear();
        self.bottom_line_idx = 0;
    }

    pub fn scroll_up(&mut self, amount: usize) {
        if self.follow_tail {
            self.bottom_line_idx = self.filtered_indices.len().saturating_sub(1);
        }
        self.bottom_line_idx = self.bottom_line_idx.saturating_sub(amount);
        self.follow_tail = false;
    }

    pub fn scroll_down(&mut self, amount: usize) {
        let max_idx = self.filtered_indices.len().saturating_sub(1);
        if self.follow_tail {
            return;
        }
        self.bottom_line_idx = (self.bottom_line_idx + amount).min(max_idx);
        if self.bottom_line_idx >= max_idx {
            self.follow_tail = true;
        }
    }

    pub fn scroll_to_start(&mut self) {
        self.bottom_line_idx = 0;
        self.follow_tail = false;
    }

    pub fn scroll_to_end(&mut self) {
        self.follow_tail = true;
        self.bottom_line_idx = self.filtered_indices.len().saturating_sub(1);
    }

    pub fn get_bottom_line_idx(&self) -> usize {
        if self.follow_tail {
            self.filtered_indices.len().saturating_sub(1)
        } else {
            self.bottom_line_idx
                .min(self.filtered_indices.len().saturating_sub(1))
        }
    }
}
