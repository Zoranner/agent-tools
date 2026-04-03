use std::sync::OnceLock;

use async_trait::async_trait;
use regex::Regex;
use reqwest::{Client, Url};
use serde_json::Value;

use crate::tool::ToolError;
use crate::web::error::{tool_error, WebErrorCode};

use super::super::backend::WebSearchBackend;
use super::super::types::WebSearchResult;

fn decode_ddg_href(href: &str) -> String {
    let absolute = if href.starts_with("//") {
        format!("https:{href}")
    } else if href.starts_with('/') && !href.starts_with("//") {
        format!("https://duckduckgo.com{href}")
    } else {
        href.to_string()
    };
    let Ok(parsed) = Url::parse(&absolute) else {
        return absolute;
    };
    for (k, v) in parsed.query_pairs() {
        if k == "uddg" {
            return v.into_owned();
        }
    }
    absolute
}

fn strip_html_tags(s: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"<[^>]+>").expect("valid regex"));
    let t = re.replace_all(s, "");
    super::super::html::html_entity_min_decode(&t)
}

fn push_result(
    out: &mut Vec<WebSearchResult>,
    title: String,
    url: String,
    snippet: String,
    limit: usize,
) {
    if out.len() >= limit || url.is_empty() {
        return;
    }
    out.push(WebSearchResult {
        title,
        url,
        snippet,
    });
}

fn collect_ddg_related(item: &Value, out: &mut Vec<WebSearchResult>, limit: usize) {
    if out.len() >= limit {
        return;
    }
    let Some(obj) = item.as_object() else {
        return;
    };
    if let Some(topics) = obj.get("Topics").and_then(|t| t.as_array()) {
        for t in topics {
            collect_ddg_related(t, out, limit);
        }
        return;
    }
    let Some(text) = obj.get("Text").and_then(|t| t.as_str()) else {
        return;
    };
    let Some(url) = obj.get("FirstURL").and_then(|u| u.as_str()) else {
        return;
    };
    if url.is_empty() {
        return;
    }
    let title = text
        .lines()
        .next()
        .unwrap_or(text)
        .chars()
        .take(200)
        .collect::<String>();
    push_result(
        out,
        title,
        url.to_string(),
        text.chars().take(500).collect::<String>(),
        limit,
    );
}

fn results_from_ddg_json(body: &str, limit: usize) -> Result<Vec<WebSearchResult>, ToolError> {
    let v: Value = serde_json::from_str(body).map_err(|e| {
        tool_error(
            WebErrorCode::NetworkError,
            format!("invalid search response JSON: {e}"),
        )
    })?;

    let mut out = Vec::new();

    let abs = v
        .get("Abstract")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim();
    let abs_url = v
        .get("AbstractURL")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim();
    let heading = v
        .get("Heading")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim();

    if !abs_url.is_empty() && !abs.is_empty() {
        let title = if heading.is_empty() {
            abs.chars().take(120).collect::<String>()
        } else {
            heading.to_string()
        };
        push_result(
            &mut out,
            title,
            abs_url.to_string(),
            abs.chars().take(500).collect::<String>(),
            limit,
        );
    }

    if let Some(topics) = v.get("RelatedTopics").and_then(|t| t.as_array()) {
        for t in topics {
            collect_ddg_related(t, &mut out, limit);
        }
    }

    Ok(out)
}

fn results_from_ddg_html(html: &str, limit: usize) -> Vec<WebSearchResult> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r#"(?si)<a[^>]+class="[^"]*?result__a[^"]*?"[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#)
            .expect("valid regex")
    });

    let mut out = Vec::new();
    for cap in re.captures_iter(html) {
        if out.len() >= limit {
            break;
        }
        let href = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let title_html = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let url = decode_ddg_href(href);
        let title = strip_html_tags(title_html);
        if title.is_empty() || url.is_empty() {
            continue;
        }
        push_result(&mut out, title, url, String::new(), limit);
    }
    out
}

/// DuckDuckGo instant JSON API plus HTML result-page fallback (no API key).
#[derive(Debug, Default, Clone, Copy)]
pub struct DuckDuckGoSearchBackend;

#[async_trait]
impl WebSearchBackend for DuckDuckGoSearchBackend {
    async fn search(
        &self,
        client: &Client,
        query: &str,
        limit: usize,
    ) -> Result<Vec<WebSearchResult>, ToolError> {
        let instant_url = Url::parse("https://api.duckduckgo.com/")
            .map_err(|e| tool_error(WebErrorCode::NetworkError, e.to_string()))?;
        let instant_resp = client
            .get(instant_url)
            .query(&[
                ("q", query),
                ("format", "json"),
                ("no_html", "1"),
                ("no_redirect", "1"),
            ])
            .send()
            .await
            .map_err(|e| tool_error(WebErrorCode::NetworkError, e.to_string()))?;

        let instant_text = instant_resp
            .text()
            .await
            .map_err(|e| tool_error(WebErrorCode::NetworkError, e.to_string()))?;

        let mut results = results_from_ddg_json(&instant_text, limit)?;

        if results.len() < limit {
            let html_url = Url::parse("https://html.duckduckgo.com/html/")
                .map_err(|e| tool_error(WebErrorCode::NetworkError, e.to_string()))?;
            let html_resp = client
                .post(html_url)
                .form(&[("q", query), ("b", "")])
                .send()
                .await
                .map_err(|e| tool_error(WebErrorCode::NetworkError, e.to_string()))?;

            let html = html_resp
                .text()
                .await
                .map_err(|e| tool_error(WebErrorCode::NetworkError, e.to_string()))?;

            let extra = results_from_ddg_html(&html, limit);
            for item in extra {
                if results.len() >= limit {
                    break;
                }
                let dup = results.iter().any(|r| r.url == item.url);
                if !dup && !item.url.is_empty() {
                    results.push(item);
                }
            }
        }

        results.truncate(limit);
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ddg_json_abstract_and_related() {
        let body = r#"{
            "Abstract": "Rust is a language",
            "AbstractURL": "https://rust-lang.org",
            "Heading": "Rust",
            "RelatedTopics": [
                { "Text": "Rust book\nMore", "FirstURL": "https://doc.rust-lang.org/book/" }
            ]
        }"#;
        let r = results_from_ddg_json(body, 10).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].url, "https://rust-lang.org");
        assert_eq!(r[1].url, "https://doc.rust-lang.org/book/");
    }

    #[test]
    fn ddg_html_links() {
        let html = r#"<a class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2F">Example</a>"#;
        let r = results_from_ddg_html(html, 5);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].url, "https://example.com/");
        assert_eq!(r[0].title, "Example");
    }
}
