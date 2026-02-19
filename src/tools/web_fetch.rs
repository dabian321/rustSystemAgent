//! Fetch a URL and return title + main text content.
//! Mirrors nodejd-system-agent/tools/WebFetchTool.js.

use colored::Colorize;
use regex::Regex;
use std::sync::OnceLock;

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const MAX_CONTENT_LENGTH: usize = 8000;
const TIMEOUT_SECS: u64 = 15;

/// Truncate at a UTF-8 character boundary so we never split a multi-byte character.
fn truncate_at_char_boundary(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

fn decode_entities(text: &str) -> String {
    let mut s = text
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ");

    static DECIMAL: OnceLock<Regex> = OnceLock::new();
    static HEX: OnceLock<Regex> = OnceLock::new();
    let dec = DECIMAL.get_or_init(|| Regex::new(r"&#(\d+);").unwrap());
    let hex = HEX.get_or_init(|| Regex::new(r"&#x([0-9a-fA-F]+);").unwrap());

    s = dec
        .replace_all(&s, |cap: &regex::Captures<'_>| {
            let n: u32 = cap.get(1).unwrap().as_str().parse().unwrap_or(0);
            char::from_u32(n).unwrap_or('\0').to_string()
        })
        .into_owned();
    s = hex
        .replace_all(&s, |cap: &regex::Captures<'_>| {
            let n = u32::from_str_radix(cap.get(1).unwrap().as_str(), 16).unwrap_or(0);
            char::from_u32(n).unwrap_or('\0').to_string()
        })
        .into_owned();
    s
}

fn extract_title(html: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"(?i)<title[^>]*>([\s\S]*?)</title>").unwrap());
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| decode_entities(m.as_str()).trim().to_string())
        .unwrap_or_default()
}

fn remove_unwanted_tags(html: &str) -> String {
    static TAGS: &[&str] = &[
        "script", "style", "noscript", "iframe", "svg", "nav", "footer", "header", "aside",
    ];
    let mut cleaned = html.to_string();
    for tag in TAGS {
        let re = Regex::new(&format!(
            r"(?is)<{tag}[^>]*>[\s\S]*?</{tag}>"
        ))
        .unwrap();
        cleaned = re.replace_all(&cleaned, " ").into_owned();
    }
    let comment_re = Regex::new(r"<!--[\s\S]*?-->").unwrap();
    comment_re.replace_all(&cleaned, " ").into_owned()
}

fn extract_main_content(html: &str) -> String {
    // <article>
    if let Some(m) = Regex::new(r"(?is)<article[^>]*>([\s\S]*?)</article>")
        .unwrap()
        .captures(html)
    {
        let inner = m.get(1).unwrap().as_str();
        if inner.len() > 200 {
            return inner.to_string();
        }
    }
    // <main>
    if let Some(m) = Regex::new(r"(?is)<main[^>]*>([\s\S]*?)</main>")
        .unwrap()
        .captures(html)
    {
        let inner = m.get(1).unwrap().as_str();
        if inner.len() > 200 {
            return inner.to_string();
        }
    }
    // role="main"
    if let Some(m) = Regex::new(r#"(?is)<[^>]*role=["']main["'][^>]*>([\s\S]*?)</\w+>"#)
        .unwrap()
        .captures(html)
    {
        let inner = m.get(1).unwrap().as_str();
        if inner.len() > 200 {
            return inner.to_string();
        }
    }
    // <body>
    if let Some(m) = Regex::new(r"(?is)<body[^>]*>([\s\S]*?)</body>")
        .unwrap()
        .captures(html)
    {
        return m.get(1).unwrap().as_str().to_string();
    }
    html.to_string()
}

fn html_to_text(html: &str) -> String {
    let mut text = html
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("<hr>", "\n")
        .replace("<hr/>", "\n")
        .replace("</p>", "\n\n")
        .replace("</div>", "\n\n")
        .replace("</h1>", "\n\n")
        .replace("</h2>", "\n\n")
        .replace("</h3>", "\n\n")
        .replace("</h4>", "\n\n")
        .replace("</h5>", "\n\n")
        .replace("</h6>", "\n\n")
        .replace("</li>", "\n\n")
        .replace("</tr>", "\n\n")
        .replace("</blockquote>", "\n\n")
        .replace("</pre>", "\n\n")
        .replace("</section>", "\n\n");

    let link_re = Regex::new(r#"<a[^>]*href=["']([^"']*)["'][^>]*>([\s\S]*?)</a>"#).unwrap();
    let tag_re_inner = Regex::new(r"<[^>]*>").unwrap();
    text = link_re
        .replace_all(&text, |cap: &regex::Captures<'_>| {
            let url = cap.get(1).unwrap().as_str();
            let link_text = cap.get(2).unwrap().as_str();
            let cleaned = tag_re_inner.replace_all(link_text, "");
            let clean = cleaned.trim();
            if url.starts_with("http") && !clean.is_empty() {
                format!("{} ({})", clean, url)
            } else {
                clean.to_string()
            }
        })
        .into_owned();

    let tag_re = Regex::new(r"<[^>]*>").unwrap();
    text = tag_re.replace_all(&text, " ").into_owned();
    text = decode_entities(&text);
    text = Regex::new(r"[ \t]+").unwrap().replace_all(&text, " ").into_owned();
    text = Regex::new(r"\n\s*\n\s*\n").unwrap().replace_all(&text, "\n\n").into_owned();
    text = text
        .lines()
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join("\n");
    text.trim().to_string()
}

/// Fetch URL and return title + main text (truncated).
pub async fn web_fetch(url: &str) -> String {
    let mut url = url.trim().to_string();
    if url.is_empty() {
        return "URL 不能为空。".to_string();
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        url = format!("https://{url}");
    }

    eprintln!("{}\n{}", "🌐 正在读取网页:".cyan(), url);
    eprintln!("{}", "-".repeat(50).bright_black());

    let client = match reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("读取网页失败: 创建客户端错误 {}", e);
            eprintln!("{}", msg.red());
            return msg;
        }
    };

    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            let msg = if e.is_timeout() {
                format!("读取网页超时 ({}秒): {}", TIMEOUT_SECS, url)
            } else {
                format!("读取网页失败: {}", e)
            };
            eprintln!("{}", msg.red());
            return msg;
        }
    };

    if !response.status().is_success() {
        let msg = format!(
            "读取网页失败: HTTP {} {}",
            response.status(),
            response.status().as_str()
        );
        eprintln!("{}", msg.red());
        return msg;
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let body = match response.text().await {
        Ok(b) => b,
        Err(e) => {
            let msg = format!("读取网页失败: {}", e);
            eprintln!("{}", msg.red());
            return msg;
        }
    };

    if !content_type.contains("text/html")
        && !content_type.contains("text/plain")
        && !content_type.contains("application/xhtml")
    {
        let truncated = if body.len() > MAX_CONTENT_LENGTH {
            format!("{}...", truncate_at_char_boundary(&body, MAX_CONTENT_LENGTH))
        } else {
            body.clone()
        };
        return format!("网页内容 ({}):\n类型: {}\n\n{}", url, content_type, truncated);
    }

    let title = extract_title(&body);
    let cleaned = remove_unwanted_tags(&body);
    let main = extract_main_content(&cleaned);
    let mut text = html_to_text(&main);

    let total_len = text.len();
    if text.len() > MAX_CONTENT_LENGTH {
        text = format!(
            "{}\n\n...[内容已截断，共 {} 字符]",
            truncate_at_char_boundary(&text, MAX_CONTENT_LENGTH),
            total_len
        );
    }

    eprintln!(
        "{}",
        format!("✅ 成功读取网页 ({} 字符)", text.len()).green()
    );
    eprintln!("{}", "-".repeat(50).bright_black());

    format!("网页内容 ({}):\n标题: {}\n\n{}", url, title, text)
}
