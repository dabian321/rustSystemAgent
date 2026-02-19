//! Web search tool using DuckDuckGo Lite (no API key).
//! Mirrors nodejd-system-agent/tools/WebSearchTool.js.

use colored::Colorize;
use regex::Regex;
use std::sync::OnceLock;

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const MAX_RESULTS: usize = 5;

fn strip_html(html: &str) -> String {
    let s = html
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");
    let re = Regex::new(r"<[^>]*>").unwrap();
    re.replace_all(&s, "").trim().to_string()
}

fn extract_real_url(ddg_url: &str) -> String {
    let re = Regex::new(r"uddg=([^&]+)").unwrap();
    if let Some(cap) = re.captures(ddg_url) {
        if let Ok(decoded) = urlencoding::decode(cap.get(1).unwrap().as_str()) {
            return decoded.into_owned();
        }
    }
    ddg_url.to_string()
}

fn is_ad(raw_url: &str) -> bool {
    raw_url.contains("/y.js?")
        || raw_url.contains("ad_provider")
        || raw_url.contains("ad_domain")
}

#[derive(Debug)]
struct SearchResult {
    title: String,
    snippet: String,
    url: String,
}

fn parse_results(html: &str) -> Vec<SearchResult> {
    static LINK_RE: OnceLock<Regex> = OnceLock::new();
    static SNIPPET_RE: OnceLock<Regex> = OnceLock::new();
    static LINK_TEXT_RE: OnceLock<Regex> = OnceLock::new();

    let link_re = LINK_RE.get_or_init(|| {
        Regex::new(r#"<a[^>]*href="([^"]*)"[^>]*class='result-link'[^>]*>([\s\S]*?)</a>"#).unwrap()
    });
    let snippet_re = SNIPPET_RE.get_or_init(|| {
        Regex::new(r"<td\s+class='result-snippet'>([\s\S]*?)</td>").unwrap()
    });
    let link_text_re = LINK_TEXT_RE.get_or_init(|| {
        Regex::new(r"<span\s+class='link-text'>([\s\S]*?)</span>").unwrap()
    });

    let mut links: Vec<(String, String)> = Vec::new();
    for cap in link_re.captures_iter(html) {
        let raw_url = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        let title = cap.get(2).map(|m| strip_html(m.as_str())).unwrap_or_default();
        links.push((raw_url, title));
    }

    let snippets: Vec<String> = snippet_re
        .find_iter(html)
        .filter_map(|m| {
            let inner = m.as_str();
            let content = inner
                .strip_prefix("<td class='result-snippet'>")
                .and_then(|s| s.strip_suffix("</td>"))
                .unwrap_or(inner);
            Some(strip_html(content))
        })
        .collect();

    let _display_urls: Vec<String> = link_text_re
        .find_iter(html)
        .filter_map(|m| {
            let inner = m.as_str();
            let content = inner
                .strip_prefix("<span class='link-text'>")
                .and_then(|s| s.strip_suffix("</span>"))
                .unwrap_or(inner);
            Some(strip_html(content))
        })
        .collect();

    let mut results = Vec::new();
    for (i, (raw_url, title)) in links.into_iter().enumerate() {
        if results.len() >= MAX_RESULTS {
            break;
        }
        if is_ad(&raw_url) {
            continue;
        }
        let mut url = extract_real_url(&raw_url);
        if url.starts_with("//") {
            url = format!("https:{url}");
        }
        let snippet = snippets.get(i).cloned().unwrap_or_default();
        results.push(SearchResult {
            title,
            snippet,
            url,
        });
    }
    results
}

/// Run a web search and return formatted results.
pub async fn web_search(query: &str) -> String {
    let query = query.trim();
    if query.is_empty() {
        return "搜索关键词不能为空。".to_string();
    }

    let url = format!(
        "https://lite.duckduckgo.com/lite/?q={}",
        urlencoding::encode(query)
    );

    eprintln!("{}\n{}", "🔍 正在搜索:".cyan(), query);
    eprintln!("{}", "-".repeat(50).bright_black());

    let client = match reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("搜索失败: 创建请求客户端错误 {}", e);
            eprintln!("{}", msg.red());
            return msg;
        }
    };

    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            let msg = format!("搜索失败: {}", e);
            eprintln!("{}", msg.red());
            return msg;
        }
    };

    if !response.status().is_success() {
        let msg = format!("搜索失败: HTTP {} {}", response.status(), response.status().as_str());
        eprintln!("{}", msg.red());
        return msg;
    }

    let html = match response.text().await {
        Ok(h) => h,
        Err(e) => {
            let msg = format!("搜索失败: 读取响应 {}", e);
            eprintln!("{}", msg.red());
            return msg;
        }
    };

    let results = parse_results(&html);

    if results.is_empty() {
        eprintln!("{}", "未找到搜索结果".yellow());
        eprintln!("{}", "-".repeat(50).bright_black());
        return "未找到相关搜索结果，请尝试更换关键词。".to_string();
    }

    let formatted: Vec<String> = results
        .iter()
        .enumerate()
        .map(|(i, r)| {
            format!(
                "[{}] {}\n    摘要: {}\n    链接: {}",
                i + 1,
                r.title,
                r.snippet,
                r.url
            )
        })
        .collect();

    eprintln!("{}", format!("✅ 找到 {} 条结果", results.len()).green());
    eprintln!("{}", "-".repeat(50).bright_black());

    format!(
        "搜索 \"{}\" 的结果 (共 {} 条):\n\n{}",
        query,
        results.len(),
        formatted.join("\n\n")
    )
}
