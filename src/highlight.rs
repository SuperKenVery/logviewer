use crate::filter::FilterExpr;
use ratatui::style::{Color, Modifier, Style};
use regex::Regex;
use std::sync::LazyLock;

#[derive(Clone)]
pub struct HighlightRule {
    pub regex: Regex,
    pub style: Style,
}

static HEURISTIC_RULES: LazyLock<Vec<HighlightRule>> = LazyLock::new(|| {
    vec![
        HighlightRule {
            regex: Regex::new(r"(?i)\b(error|err|fatal|fail(ed)?|panic)\b").unwrap(),
            style: Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        },
        HighlightRule {
            regex: Regex::new(r"(?i)\b(warn(ing)?)\b").unwrap(),
            style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        },
        HighlightRule {
            regex: Regex::new(r"(?i)\b(info)\b").unwrap(),
            style: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        },
        HighlightRule {
            regex: Regex::new(r"(?i)\b(debug|trace)\b").unwrap(),
            style: Style::default().fg(Color::Cyan),
        },
        HighlightRule {
            regex: Regex::new(r"\[[^\]]+\]").unwrap(),
            style: Style::default().fg(Color::Blue),
        },
        HighlightRule {
            regex: Regex::new(r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}").unwrap(),
            style: Style::default().fg(Color::Magenta),
        },
        HighlightRule {
            regex: Regex::new(r"\d{2}:\d{2}:\d{2}").unwrap(),
            style: Style::default().fg(Color::Magenta),
        },
    ]
});

#[derive(Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub style: Style,
}

pub fn highlight_line(
    text: &str,
    custom_filter: Option<&FilterExpr>,
    heuristic_enabled: bool,
) -> Vec<Span> {
    let mut spans = Vec::new();

    if let Some(filter) = custom_filter {
        let matches = filter.find_all_matches(text);
        for (start, end) in matches {
            spans.push(Span {
                start,
                end,
                style: Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            });
        }
    }

    if heuristic_enabled {
        for rule in HEURISTIC_RULES.iter() {
            for m in rule.regex.find_iter(text) {
                spans.push(Span {
                    start: m.start(),
                    end: m.end(),
                    style: rule.style,
                });
            }
        }
    }

    spans.sort_by_key(|s| s.start);
    spans
}

pub fn apply_highlights(text: &str, spans: &[Span]) -> Vec<(String, Style)> {
    if spans.is_empty() {
        return vec![(text.to_string(), Style::default())];
    }

    let mut result = Vec::new();
    let mut pos = 0;

    for span in spans {
        let start = char_to_byte_pos(text, span.start);
        let end = char_to_byte_pos(text, span.end);

        if start > pos {
            result.push((text[pos..start].to_string(), Style::default()));
        }
        if end > start && end <= text.len() {
            result.push((text[start..end].to_string(), span.style));
            pos = end;
        }
    }

    if pos < text.len() {
        result.push((text[pos..].to_string(), Style::default()));
    }

    result
}

fn char_to_byte_pos(text: &str, char_pos: usize) -> usize {
    text.char_indices()
        .nth(char_pos)
        .map(|(i, _)| i)
        .unwrap_or(text.len())
        .min(text.len())
}
