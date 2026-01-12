use crate::input::TextInput;
use crate::state::AppState;

#[derive(Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    HideEdit,
    FilterEdit,
    HighlightEdit,
    LineStartEdit,
}

#[derive(Clone)]
pub struct InputFields {
    pub hide: TextInput,
    pub filter: TextInput,
    pub highlight: TextInput,
    pub line_start: TextInput,
}

impl InputFields {
    pub fn from_state(state: &AppState) -> Self {
        Self {
            hide: TextInput::new(state.hide_input.clone()),
            filter: TextInput::new(state.filter_input.clone()),
            highlight: TextInput::new(state.highlight_input.clone()),
            line_start: TextInput::new(state.line_start_regex.clone()),
        }
    }

    pub fn get_active_mut(&mut self, mode: InputMode) -> Option<&mut TextInput> {
        match mode {
            InputMode::HideEdit => Some(&mut self.hide),
            InputMode::FilterEdit => Some(&mut self.filter),
            InputMode::HighlightEdit => Some(&mut self.highlight),
            InputMode::LineStartEdit => Some(&mut self.line_start),
            InputMode::Normal => None,
        }
    }
}
