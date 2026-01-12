use crate::filter::FilterExpr;
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HighlightStyle {
    None,
    Error,
    Warning,
    Info,
    Debug,
    Bracket,
    Timestamp,
    CustomHighlight,
    JsonKey,
    JsonString,
    JsonNumber,
    JsonBool,
    JsonNull,
}

impl HighlightStyle {
    pub fn css_class(&self) -> &'static str {
        match self {
            HighlightStyle::None => "",
            HighlightStyle::Error => "hl-error",
            HighlightStyle::Warning => "hl-warn",
            HighlightStyle::Info => "hl-info",
            HighlightStyle::Debug => "hl-debug",
            HighlightStyle::Bracket => "hl-bracket",
            HighlightStyle::Timestamp => "hl-timestamp",
            HighlightStyle::CustomHighlight => "hl-custom",
            HighlightStyle::JsonKey => "hl-json-key",
            HighlightStyle::JsonString => "hl-json-string",
            HighlightStyle::JsonNumber => "hl-json-number",
            HighlightStyle::JsonBool => "hl-json-bool",
            HighlightStyle::JsonNull => "hl-json-null",
        }
    }

    pub fn to_ratatui_style(&self) -> ratatui::style::Style {
        use ratatui::style::{Color, Modifier, Style};
        match self {
            HighlightStyle::None => Style::default(),
            HighlightStyle::Error => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            HighlightStyle::Warning => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            HighlightStyle::Info => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            HighlightStyle::Debug => Style::default().fg(Color::Cyan),
            HighlightStyle::Bracket => Style::default().fg(Color::Blue),
            HighlightStyle::Timestamp => Style::default().fg(Color::Magenta),
            HighlightStyle::CustomHighlight => Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD),
            HighlightStyle::JsonKey => Style::default().fg(Color::Cyan),
            HighlightStyle::JsonString => Style::default().fg(Color::Green),
            HighlightStyle::JsonNumber => Style::default().fg(Color::Yellow),
            HighlightStyle::JsonBool => Style::default().fg(Color::Magenta),
            HighlightStyle::JsonNull => Style::default().fg(Color::Red),
        }
    }
}

#[derive(Clone)]
struct HeuristicRule {
    regex: Regex,
    style: HighlightStyle,
}

static HEURISTIC_RULES: LazyLock<Vec<HeuristicRule>> = LazyLock::new(|| {
    vec![
        HeuristicRule {
            regex: Regex::new(r"(?i)\b(error|err|fatal|fail(ed)?|panic)\b").unwrap(),
            style: HighlightStyle::Error,
        },
        HeuristicRule {
            regex: Regex::new(r"(?i)\b(warn(ing)?)\b").unwrap(),
            style: HighlightStyle::Warning,
        },
        HeuristicRule {
            regex: Regex::new(r"(?i)\b(info)\b").unwrap(),
            style: HighlightStyle::Info,
        },
        HeuristicRule {
            regex: Regex::new(r"(?i)\b(debug|trace)\b").unwrap(),
            style: HighlightStyle::Debug,
        },
        HeuristicRule {
            regex: Regex::new(r"\[[^\]]+\]").unwrap(),
            style: HighlightStyle::Bracket,
        },
        HeuristicRule {
            regex: Regex::new(r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}").unwrap(),
            style: HighlightStyle::Timestamp,
        },
        HeuristicRule {
            regex: Regex::new(r"\d{2}:\d{2}:\d{2}").unwrap(),
            style: HighlightStyle::Timestamp,
        },
    ]
});

#[derive(Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub style: HighlightStyle,
    pub priority: u8,
}

pub fn highlight_line(
    text: &str,
    custom_filter: Option<&FilterExpr>,
    heuristic_enabled: bool,
    json_enabled: bool,
) -> Vec<Span> {
    let mut spans = Vec::new();

    if let Some(filter) = custom_filter {
        let matches = filter.find_all_matches(text);
        for (start, end) in matches {
            spans.push(Span {
                start,
                end,
                style: HighlightStyle::CustomHighlight,
                priority: 100,
            });
        }
    }

    if json_enabled {
        if let Some(json_spans) = highlight_json(text) {
            spans.extend(json_spans);
        }
    }

    if heuristic_enabled {
        for rule in HEURISTIC_RULES.iter() {
            for m in rule.regex.find_iter(text) {
                spans.push(Span {
                    start: m.start(),
                    end: m.end(),
                    style: rule.style,
                    priority: 10,
                });
            }
        }
    }

    spans.sort_by(|a, b| {
        a.start.cmp(&b.start).then(b.priority.cmp(&a.priority))
    });
    spans
}

pub fn apply_highlights(text: &str, spans: &[Span]) -> Vec<(String, HighlightStyle)> {
    if spans.is_empty() {
        return vec![(text.to_string(), HighlightStyle::None)];
    }

    let mut style_at: Vec<(HighlightStyle, u8)> = vec![(HighlightStyle::None, 0); text.len()];
    
    for span in spans {
        let start = char_to_byte_pos(text, span.start);
        let end = char_to_byte_pos(text, span.end).min(text.len());
        
        for i in start..end {
            if span.priority >= style_at[i].1 {
                style_at[i] = (span.style, span.priority);
            }
        }
    }

    let mut result = Vec::new();
    let mut pos = 0;
    
    while pos < text.len() {
        let current_style = style_at[pos].0;
        let mut end = pos + 1;
        
        while end < text.len() && style_at[end].0 == current_style {
            end += 1;
        }
        
        result.push((text[pos..end].to_string(), current_style));
        pos = end;
    }

    result
}

pub fn apply_highlights_ratatui(text: &str, spans: &[Span]) -> Vec<(String, ratatui::style::Style)> {
    apply_highlights(text, spans)
        .into_iter()
        .map(|(s, style)| (s, style.to_ratatui_style()))
        .collect()
}

fn char_to_byte_pos(text: &str, char_pos: usize) -> usize {
    text.char_indices()
        .nth(char_pos)
        .map(|(i, _)| i)
        .unwrap_or(text.len())
        .min(text.len())
}

fn highlight_json(text: &str) -> Option<Vec<Span>> {
    let json_objects = find_all_json(text);
    if json_objects.is_empty() {
        return None;
    }
    
    let mut spans = Vec::new();
    for (json_start, value, json_end) in json_objects {
        let json_str = &text[json_start..json_start + json_end];
        highlight_json_value(json_str, &value, json_start, &mut spans);
    }
    Some(spans)
}

fn find_all_json(text: &str) -> Vec<(usize, Value, usize)> {
    let mut results = Vec::new();
    let mut search_start = 0;
    
    while let Some(pos) = text[search_start..].find(|c| c == '{' || c == '[') {
        let abs_pos = search_start + pos;
        let json_str = &text[abs_pos..];
        
        let bytes = json_str.as_bytes();
        let mut stream = serde_json::Deserializer::from_slice(bytes).into_iter::<Value>();
        
        if let Some(Ok(value)) = stream.next() {
            let end = stream.byte_offset();
            if end > 1 {
                results.push((abs_pos, value, end));
                search_start = abs_pos + end;
                continue;
            }
        }
        search_start = abs_pos + 1;
    }
    results
}

fn highlight_json_value(text: &str, value: &Value, base_offset: usize, spans: &mut Vec<Span>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                if let Some(key_pos) = find_json_key(text, key) {
                    spans.push(Span {
                        start: base_offset + key_pos,
                        end: base_offset + key_pos + key.len() + 2,
                        style: HighlightStyle::JsonKey,
                        priority: 50,
                    });
                }
                highlight_json_value(text, val, base_offset, spans);
            }
        }
        Value::Array(arr) => {
            for val in arr {
                highlight_json_value(text, val, base_offset, spans);
            }
        }
        Value::String(s) => {
            if let Some(pos) = find_json_string(text, s) {
                spans.push(Span {
                    start: base_offset + pos,
                    end: base_offset + pos + s.len() + 2,
                    style: HighlightStyle::JsonString,
                    priority: 50,
                });
            }
        }
        Value::Number(n) => {
            let n_str = n.to_string();
            if let Some(pos) = text.find(&n_str) {
                spans.push(Span {
                    start: base_offset + pos,
                    end: base_offset + pos + n_str.len(),
                    style: HighlightStyle::JsonNumber,
                    priority: 50,
                });
            }
        }
        Value::Bool(b) => {
            let b_str = if *b { "true" } else { "false" };
            if let Some(pos) = text.find(b_str) {
                spans.push(Span {
                    start: base_offset + pos,
                    end: base_offset + pos + b_str.len(),
                    style: HighlightStyle::JsonBool,
                    priority: 50,
                });
            }
        }
        Value::Null => {
            if let Some(pos) = text.find("null") {
                spans.push(Span {
                    start: base_offset + pos,
                    end: base_offset + pos + 4,
                    style: HighlightStyle::JsonNull,
                    priority: 50,
                });
            }
        }
    }
}

fn find_json_key(text: &str, key: &str) -> Option<usize> {
    let pattern = format!("\"{}\"", key);
    let pos = text.find(&pattern)?;
    let after = &text[pos + pattern.len()..];
    if after.trim_start().starts_with(':') {
        Some(pos)
    } else {
        None
    }
}

fn find_json_string(text: &str, s: &str) -> Option<usize> {
    let pattern = format!("\"{}\"", s);
    let mut search_start = 0;
    while let Some(pos) = text[search_start..].find(&pattern) {
        let abs_pos = search_start + pos;
        let after = &text[abs_pos + pattern.len()..];
        if !after.trim_start().starts_with(':') {
            return Some(abs_pos);
        }
        search_start = abs_pos + 1;
    }
    None
}
