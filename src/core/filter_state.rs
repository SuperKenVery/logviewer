use crate::filter::FilterExpr;
use fancy_regex::Regex;

#[derive(Clone)]
pub struct FilterState {
    pub hide_regex: Option<Regex>,
    pub filter_expr: Option<FilterExpr>,
    pub highlight_expr: Option<FilterExpr>,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            hide_regex: None,
            filter_expr: None,
            highlight_expr: None,
        }
    }
}
