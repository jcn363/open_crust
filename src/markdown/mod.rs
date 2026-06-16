//! Markdown utilities for parsing and manipulation

use regex::Regex;
use std::sync::LazyLock;

static RE_HEADINGS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(#{1,6})\s+(.+)$").unwrap());
static RE_CODE_BLOCKS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)```(\w*?)\n(.+?)```").unwrap());
static RE_INLINE_CODE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`([^`]+)`").unwrap());
static RE_LINKS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap());
static RE_IMAGES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap());
static RE_URLS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"https?://[^\s\)>\]`]+").unwrap());
static RE_LIST_ITEMS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^[-*]\s+(.+)$").unwrap());
static RE_NUMBERED_ITEMS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\d+\.\s+(.+)$").unwrap());
static RE_TASKS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[-*]\s+\[([ x])\]\s+(.+)$").unwrap());
static RE_TABLES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?ms)^\|.+\|\n\|[-:|]+\|\n(.+?)\n\n").unwrap());
static RE_QUOTES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^>\s+(.+)$").unwrap());
static RE_BOLD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*\*(.+?)\*\*|__(.+?)__").unwrap());
static RE_ITALIC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*(.+?)\*|_(.+?)_").unwrap());
static RE_CODE_FENCE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"```[\s\S]*?```").unwrap());
static RE_URLS_WORD_COUNT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https?://[^\s]+").unwrap());
static RE_LINKS_HTML: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap());
static RE_CODE_BLOCKS_HTML: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"```(\w*)\n([\s\S]*?)```").unwrap());
static RE_INLINE_CODE_HTML: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"`([^`]+)`").unwrap());
static RE_LIST_ITEMS_HTML: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^[-*]\s+(.+)$").unwrap());
static RE_LIST_WRAP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(<li>.*</li>\n)+").unwrap());
static RE_BOLD_HTML: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*\*(.+?)\*\*").unwrap());
static RE_ITALIC_HTML: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\*(.+?)\*").unwrap());

/// Extract frontmatter from markdown
pub fn extract_frontmatter(md: &str) -> Option<(String, String)> {
    let re = Regex::new(r"^---\n(.+?)\n---\n").ok()?;
    if let Some(caps) = re.captures(md) {
        let fm = caps.get(1)?.as_str().to_string();
        let end_pos = caps.get(0)?.end();
        let content = md[end_pos..].to_string();
        Some((fm, content))
    } else {
        None
    }
}

/// Extract title from markdown (# Title)
pub fn extract_title(md: &str) -> Option<String> {
    let re = Regex::new(r"(?m)^#\s+(.+)$").ok()?;
    re.captures(md).map(|c| c[1].to_string())
}

/// Extract all headings as (level, text)
pub fn extract_headings(md: &str) -> Vec<(u8, String)> {
    RE_HEADINGS
        .captures_iter(md)
        .map(|c| {
            let level = c[1].len() as u8;
            let text = c[2].to_string();
            (level, text)
        })
        .collect()
}

/// Extract code blocks with language
pub fn extract_code_blocks(md: &str) -> Vec<(Option<String>, String)> {
    RE_CODE_BLOCKS
        .captures_iter(md)
        .map(|c| {
            let lang = if c.get(1).is_some_and(|m| !m.as_str().is_empty()) {
                Some(c[1].to_string())
            } else {
                None
            };
            let code = c[2].to_string();
            (lang, code)
        })
        .collect()
}

/// Extract inline code (`code`)
pub fn extract_inline_code(md: &str) -> Vec<String> {
    RE_INLINE_CODE
        .captures_iter(md)
        .map(|c| c[1].to_string())
        .collect()
}

/// Extract links [text](url)
pub fn extract_links(md: &str) -> Vec<(String, String)> {
    RE_LINKS
        .captures_iter(md)
        .map(|c| (c[1].to_string(), c[2].to_string()))
        .collect()
}

/// Extract images ![alt](url)
pub fn extract_images(md: &str) -> Vec<(String, String)> {
    RE_IMAGES
        .captures_iter(md)
        .map(|c| (c[1].to_string(), c[2].to_string()))
        .collect()
}

/// Extract URLs from markdown
pub fn extract_urls(md: &str) -> Vec<String> {
    RE_URLS
        .find_iter(md)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Extract bullet list items
pub fn extract_list_items(md: &str) -> Vec<String> {
    RE_LIST_ITEMS
        .captures_iter(md)
        .map(|c| c[1].to_string())
        .collect()
}

/// Extract numbered list items
pub fn extract_numbered_items(md: &str) -> Vec<String> {
    RE_NUMBERED_ITEMS
        .captures_iter(md)
        .map(|c| c[1].to_string())
        .collect()
}

/// Extract task list items (\- [ ] or \- \[x\])
pub fn extract_tasks(md: &str) -> Vec<(bool, String)> {
    RE_TASKS
        .captures_iter(md)
        .map(|c| {
            let checked = c[1].starts_with('x');
            let text = c[2].to_string();
            (checked, text)
        })
        .collect()
}

/// Extract tables
pub fn extract_tables(md: &str) -> Vec<Vec<Vec<String>>> {
    RE_TABLES
        .captures_iter(md)
        .map(|c| {
            let table_str = c.get(1).map_or("", |m| m.as_str());
            parse_table(table_str)
        })
        .collect()
}

fn parse_table(table_str: &str) -> Vec<Vec<String>> {
    table_str
        .lines()
        .map(|line| {
            line.split('|')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_string())
                .collect()
        })
        .collect()
}

/// Extract blockquotes
pub fn extract_quotes(md: &str) -> Vec<String> {
    RE_QUOTES
        .captures_iter(md)
        .map(|c| c[1].to_string())
        .collect()
}

/// Extract bold (**text** or __text__)
pub fn extract_bold(md: &str) -> Vec<String> {
    RE_BOLD
        .captures_iter(md)
        .map(|c| c.get(1).map_or("", |m| m.as_str()).to_string())
        .collect()
}

/// Extract italic (*text* or _text_)
pub fn extract_italic(md: &str) -> Vec<String> {
    RE_ITALIC
        .captures_iter(md)
        .map(|c| c.get(1).map_or("", |m| m.as_str()).to_string())
        .collect()
}

/// Count words in markdown (excluding code blocks and URLs)
pub fn count_words(md: &str) -> usize {
    // Remove code blocks
    let md = RE_CODE_FENCE.replace_all(md, "");

    // Remove inline code
    let md = RE_INLINE_CODE.replace_all(&md, "");

    // Remove URLs
    let md = RE_URLS_WORD_COUNT.replace_all(&md, "");

    // Count words
    md.split_whitespace().count()
}

/// Get document summary (first paragraph)
pub fn get_summary(md: &str) -> String {
    let paragraphs: Vec<_> = md.split("\n\n").filter(|s| !s.starts_with('#')).collect();
    paragraphs
        .first()
        .map(|s| s.trim())
        .unwrap_or("")
        .to_string()
}

/// Convert markdown to HTML (basic)
pub fn to_html(md: &str) -> String {
    let mut html = md.to_string();

    // Headers
    for i in (1..=6).rev() {
        let pattern = format!(r"(?m)^{}\s+(.+)$", "#".repeat(i));
        let re = Regex::new(&pattern).unwrap();
        html = re
            .replace(&html, |caps: &regex::Captures| {
                format!("<h{}>{}</h{}>", i, &caps[1], i)
            })
            .to_string();
    }

    // Bold
    html = RE_BOLD_HTML
        .replace_all(&html, "<strong>$1</strong>")
        .to_string();

    // Italic
    html = RE_ITALIC_HTML.replace_all(&html, "<em>$1</em>").to_string();

    // Links
    html = RE_LINKS_HTML
        .replace_all(&html, "<a href=\"$2\">$1</a>")
        .to_string();

    // Code blocks
    html = RE_CODE_BLOCKS_HTML
        .replace_all(&html, "<pre><code class=\"lang-$1\">$2</code></pre>")
        .to_string();

    // Inline code
    html = RE_INLINE_CODE_HTML
        .replace_all(&html, "<code>$1</code>")
        .to_string();

    // List items
    html = RE_LIST_ITEMS_HTML
        .replace_all(&html, "<li>$1</li>")
        .to_string();

    // Wrap list items in <ul>
    html = RE_LIST_WRAP.replace_all(&html, "<ul>$0</ul>").to_string();

    html
}

/// Check if markdown is valid (has at least a title)
pub fn is_valid(md: &str) -> bool {
    extract_title(md).is_some() || md.len() > 10
}

/// Get headings tree
pub fn get_headings_tree(md: &str) -> Vec<(u8, String, usize)> {
    let headings = extract_headings(md);
    let mut tree = Vec::new();
    let mut stack: Vec<u8> = Vec::new();

    for (level, text) in headings {
        while let Some(&last_level) = stack.last() {
            if level > last_level {
                break;
            }
            stack.pop();
        }

        let stack_depth = stack.len();
        tree.push((level, text, stack_depth));
        stack.push(level);
    }

    tree
}

#[cfg(test)]
mod tests;
