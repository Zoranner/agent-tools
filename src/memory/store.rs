//! Markdown 记忆文件：`.agent/memory/YYYY/MM/dd.md`（按日追加）与 `MEMORY.md`（长期总结）。
//!
//! 与 [OpenClaw memory 布局](https://github.com/openclaw/openclaw/blob/main/docs/concepts/memory.md) 相近：
//! OpenClaw 使用工作区根下 `memory/YYYY-MM-DD.md` 与根目录 `MEMORY.md`；本库默认将二者统一放在 **`.agent/memory/`** 下，并按 **年/月/日** 分子目录。

use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use chrono::{Datelike, Utc};
use walkdir::WalkDir;

use super::error::{tool_error, MemoryErrorCode};
use crate::tool::ToolError;

/// 相对记忆根路径是否为总结文件 `MEMORY.md`。
pub(crate) fn is_summary_rel(file_rel: &str) -> bool {
    file_rel.eq_ignore_ascii_case("MEMORY.md")
}

pub(crate) fn block_kind(file_rel: &str) -> &'static str {
    if is_summary_rel(file_rel) {
        "summary"
    } else {
        "daily"
    }
}

/// 写入块末尾的机器可读元数据（单行 HTML 注释，便于解析与排序）。
fn format_block(key: &str, content: &str, tags: &[String], at: &str) -> String {
    let tags_part = if tags.is_empty() {
        String::new()
    } else {
        format!(" tags={}", tags.join("|"))
    };
    format!(
        "### {key}\n\n{body}\n\n<!-- agentool-memory: at={at}{tags_part} -->\n\n",
        body = content.trim_end(),
        key = key,
        at = at,
        tags_part = tags_part,
    )
}

pub(crate) fn daily_file_path(memory_dir: &Path, date: chrono::NaiveDate) -> PathBuf {
    memory_dir
        .join(format!("{:04}", date.year()))
        .join(format!("{:02}", date.month()))
        .join(format!("{:02}.md", date.day()))
}

pub(crate) fn summary_file_path(memory_dir: &Path) -> PathBuf {
    memory_dir.join("MEMORY.md")
}

pub(crate) fn append_block(
    path: &Path,
    key: &str,
    content: &str,
    tags: &[String],
) -> Result<(), ToolError> {
    if key.contains('\n') || key.contains('\r') {
        return Err(tool_error(
            MemoryErrorCode::InvalidKey,
            "memory key must be a single line",
        ));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            tool_error(
                MemoryErrorCode::StorageError,
                format!("create memory path: {e}"),
            )
        })?;
    }
    let at = Utc::now().to_rfc3339();
    let block = format_block(key, content, tags, &at);
    let mut existing = if path.exists() {
        fs::read_to_string(path).map_err(|e| {
            tool_error(
                MemoryErrorCode::StorageError,
                format!("read {}: {e}", path.display()),
            )
        })?
    } else {
        String::new()
    };
    if !existing.is_empty() && !existing.ends_with('\n') {
        existing.push('\n');
    }
    existing.push_str(&block);
    fs::write(path, existing).map_err(|e| {
        tool_error(
            MemoryErrorCode::StorageError,
            format!("write {}: {e}", path.display()),
        )
    })
}

/// 从文件中删掉 `[start, end)` 字节区间，在文件末尾追加新块（用于 `memory_update`）。
pub(crate) fn remove_span_and_append_block(
    path: &Path,
    span: Range<usize>,
    key: &str,
    content: &str,
    tags: &[String],
) -> Result<(), ToolError> {
    let mut text = fs::read_to_string(path).map_err(|e| {
        tool_error(
            MemoryErrorCode::StorageError,
            format!("read {}: {e}", path.display()),
        )
    })?;
    if span.start > text.len() || span.end > text.len() || span.start > span.end {
        return Err(tool_error(
            MemoryErrorCode::StorageError,
            "internal error: invalid byte span for memory block",
        ));
    }
    text.replace_range(span.clone(), "");
    let at = Utc::now().to_rfc3339();
    let block = format_block(key, content, tags, &at);
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str(&block);
    fs::write(path, text).map_err(|e| {
        tool_error(
            MemoryErrorCode::StorageError,
            format!("write {}: {e}", path.display()),
        )
    })
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedBlock {
    pub key: String,
    pub content: String,
    pub tags: Vec<String>,
    pub at: String,
    /// 相对记忆根目录的路径（使用 `/` 分隔，便于 JSON 与跨平台展示）。
    pub file_rel: String,
}

/// 磁盘上的块位置，供删除/替换。
#[derive(Debug, Clone)]
pub(crate) struct LocatedBlock {
    pub block: ParsedBlock,
    pub abs_path: PathBuf,
    pub byte_range: Range<usize>,
}

fn parse_meta_comment(line: &str) -> Option<(String, Vec<String>)> {
    let t = line.trim();
    let inner = t
        .strip_prefix("<!-- agentool-memory:")?
        .trim()
        .strip_suffix("-->")?
        .trim();
    let rest = inner.strip_prefix("at=")?;
    if let Some(pos) = rest.find(" tags=") {
        let at = rest[..pos].trim().to_string();
        let tagpart = rest[pos + " tags=".len()..].trim();
        let tags = if tagpart.is_empty() {
            vec![]
        } else {
            tagpart
                .split('|')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };
        Some((at, tags))
    } else {
        Some((rest.trim().to_string(), vec![]))
    }
}

fn line_byte_ranges(text: &str) -> Vec<Range<usize>> {
    let mut v = Vec::new();
    let mut start = 0usize;
    let b = text.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\n' {
            v.push(start..i + 1);
            start = i + 1;
        }
        i += 1;
    }
    if start < text.len() {
        v.push(start..text.len());
    }
    v
}

/// 解析 `.md` 中的 `### key` 块，并记录该块在 `text` 中的字节区间（含结尾换行）。
pub(crate) fn parse_file_spanned(rel: &str, text: &str) -> Vec<(ParsedBlock, Range<usize>)> {
    let lines = line_byte_ranges(text);
    let mut i = 0;
    let mut out = Vec::new();
    while i < lines.len() {
        let line_range = &lines[i];
        let line = &text[line_range.start..line_range.end].trim_end_matches(['\r', '\n']);
        if let Some(rest) = line.strip_prefix("### ") {
            let key = rest.trim().to_string();
            if key.is_empty() {
                i += 1;
                continue;
            }
            let block_start = line_range.start;
            i += 1;
            while i < lines.len() {
                let t = text[lines[i].start..lines[i].end].trim();
                if t.is_empty() {
                    i += 1;
                } else {
                    break;
                }
            }
            let mut body = String::new();
            let mut at = String::new();
            let mut tags = Vec::new();
            let mut found_meta = false;
            let mut block_end = line_range.end;
            while i < lines.len() {
                let lr = &lines[i];
                let raw_line = &text[lr.start..lr.end];
                let l = raw_line.trim_end_matches(['\r', '\n']);
                if l.starts_with("### ") {
                    break;
                }
                if l.trim().starts_with("<!-- agentool-memory:") {
                    if let Some((a, t)) = parse_meta_comment(l) {
                        at = a;
                        tags = t;
                        found_meta = true;
                    }
                    block_end = lr.end;
                    i += 1;
                    break;
                }
                if !body.is_empty() || !l.is_empty() {
                    if !body.is_empty() {
                        body.push('\n');
                    }
                    body.push_str(l);
                }
                block_end = lr.end;
                i += 1;
            }
            if !found_meta {
                at.clear();
            }
            out.push((
                ParsedBlock {
                    key,
                    content: body.trim().to_string(),
                    tags,
                    at,
                    file_rel: rel.to_string(),
                },
                block_start..block_end,
            ));
            continue;
        }
        i += 1;
    }
    out
}

fn posix_rel(memory_dir: &Path, path: &Path) -> String {
    path.strip_prefix(memory_dir)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

pub(crate) fn collect_all_blocks(memory_dir: &Path) -> Result<Vec<ParsedBlock>, ToolError> {
    Ok(collect_located_blocks(memory_dir)?
        .into_iter()
        .map(|l| l.block)
        .collect())
}

pub(crate) fn collect_located_blocks(memory_dir: &Path) -> Result<Vec<LocatedBlock>, ToolError> {
    if !memory_dir.exists() {
        return Ok(Vec::new());
    }
    let mut blocks = Vec::new();
    for entry in WalkDir::new(memory_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path
            .extension()
            .and_then(|s| s.to_str())
            .map(|e| e.eq_ignore_ascii_case("md"))
            != Some(true)
        {
            continue;
        }
        let rel = posix_rel(memory_dir, path);
        let text = fs::read_to_string(path).map_err(|e| {
            tool_error(
                MemoryErrorCode::StorageError,
                format!("read {}: {e}", path.display()),
            )
        })?;
        for (b, range) in parse_file_spanned(&rel, &text) {
            blocks.push(LocatedBlock {
                block: b,
                abs_path: path.to_path_buf(),
                byte_range: range,
            });
        }
    }
    Ok(blocks)
}

pub(crate) fn any_key_exists(blocks: &[ParsedBlock], key: &str) -> bool {
    blocks.iter().any(|b| b.key == key)
}
