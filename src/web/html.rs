use std::sync::OnceLock;

use regex::Regex;

/// Minimal HTML entity decode for titles/snippets.
pub(crate) fn html_entity_min_decode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

pub(crate) fn extract_title(html: &str) -> Option<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re =
        RE.get_or_init(|| Regex::new(r"(?si)<title[^>]*>([^<]+)</title>").expect("valid regex"));
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| html_entity_min_decode(m.as_str().trim()))
}
