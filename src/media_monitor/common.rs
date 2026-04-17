// src/media_monitor/common.rs
use regex::Regex;
use std::sync::LazyLock;

static TRACK_PREFIX_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(?:\d+|[A-D]\d+)[.\-]\s*").expect("Hard-coded regex is valid")
});

pub fn clean_title(raw: &str) -> String {
    TRACK_PREFIX_RE.replace(raw, "").into_owned()
}