use std::sync::OnceLock;

use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TocItem {
    pub level: u8,
    pub title: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct DocumentStats {
    pub characters: usize,
    pub paragraphs: usize,
    pub headings: usize,
    pub lines: usize,
}

fn atx_heading_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(#{1,6})\s+(.+?)\s*#*\s*$").expect("atx heading regex"))
}

fn toggles_fence(trimmed_start: &str) -> bool {
    trimmed_start.starts_with("```") || trimmed_start.starts_with("~~~")
}

/// ATX headings outside fenced code blocks; line numbers are 1-based.
pub(crate) fn extract_toc(content: &str) -> Vec<TocItem> {
    let re = atx_heading_re();
    let mut in_fence = false;
    let mut out = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = line.trim_start();
        if toggles_fence(trimmed) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let Some(caps) = re.captures(line.trim_end()) else {
            continue;
        };
        let level = caps.get(1).map(|m| m.as_str().len()).unwrap_or(0);
        let title = caps
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        if !(1..=6).contains(&level) {
            continue;
        }
        out.push(TocItem {
            level: level as u8,
            title,
            line: line_no,
        });
    }
    out
}

pub(crate) fn document_stats(content: &str) -> DocumentStats {
    let lines = content.lines().count();

    let mut in_fence = false;
    let mut heading_count = 0usize;
    let re = atx_heading_re();

    for line in content.lines() {
        let trimmed = line.trim_start();
        if toggles_fence(trimmed) {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence && re.is_match(line.trim_end()) {
            heading_count += 1;
        }
    }

    let non_fence: String = strip_fenced_blocks(content);

    let characters = non_fence.chars().filter(|c| !c.is_whitespace()).count();

    let paragraphs = non_fence
        .split("\n\n")
        .map(str::trim)
        .filter(|b| !b.is_empty())
        .count();

    DocumentStats {
        characters,
        paragraphs,
        headings: heading_count,
        lines,
    }
}

/// Remove fenced ``` / ~~~ blocks (inclusive) for word/paragraph stats so code isn't counted.
fn strip_fenced_blocks(content: &str) -> String {
    let mut in_fence = false;
    let mut buf = String::with_capacity(content.len());
    for line in content.lines() {
        let trimmed = line.trim_start();
        if toggles_fence(trimmed) {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            if !buf.is_empty() {
                buf.push('\n');
            }
            buf.push_str(line);
        }
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toc_skips_fenced_headings() {
        let md = "# Real\n```\n# Fake\n```\n## Two\n";
        let t = extract_toc(md);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].title, "Real");
        assert_eq!(t[1].title, "Two");
    }

    #[test]
    fn stats_strip_fence_for_characters() {
        let md = "hello world\n```\nmore tokens here\n```\n";
        let s = document_stats(md);
        assert_eq!(
            s.characters,
            "hello world".chars().filter(|c| !c.is_whitespace()).count()
        );
    }
}
