//! Markdown utilities for parsing and manipulation

use regex::Regex;

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
    let re = Regex::new(r"(?m)^(#{1,6})\s+(.+)$").expect("valid regex: headings pattern (#{1,6})");
    re.captures_iter(md)
        .map(|c| {
            let level = c[1].len() as u8;
            let text = c[2].to_string();
            (level, text)
        })
        .collect()
}

/// Extract code blocks with language
pub fn extract_code_blocks(md: &str) -> Vec<(Option<String>, String)> {
    let re = Regex::new(r"(?s)```(\w*?)\n(.+?)```").expect("valid regex: fenced code block pattern");
    re.captures_iter(md)
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
    let re = Regex::new(r"`([^`]+)`").expect("valid regex: inline code pattern");
    re.captures_iter(md).map(|c| c[1].to_string()).collect()
}

/// Extract links [text](url)
pub fn extract_links(md: &str) -> Vec<(String, String)> {
    let re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").expect("valid regex: link pattern [text](url)");
    re.captures_iter(md)
        .map(|c| (c[1].to_string(), c[2].to_string()))
        .collect()
}

/// Extract images ![alt](url)
pub fn extract_images(md: &str) -> Vec<(String, String)> {
    let re = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").expect("valid regex: image pattern ![alt](url)");
    re.captures_iter(md)
        .map(|c| (c[1].to_string(), c[2].to_string()))
        .collect()
}

/// Extract URLs from markdown
pub fn extract_urls(md: &str) -> Vec<String> {
    let re = Regex::new(r"https?://[^\s\)>\]`]+").expect("valid regex: URL pattern");
    re.find_iter(md).map(|m| m.as_str().to_string()).collect()
}

/// Extract bullet list items
pub fn extract_list_items(md: &str) -> Vec<String> {
    let re = Regex::new(r"(?m)^[-*]\s+(.+)$").expect("valid regex: unordered list pattern");
    re.captures_iter(md).map(|c| c[1].to_string()).collect()
}

/// Extract numbered list items
pub fn extract_numbered_items(md: &str) -> Vec<String> {
    let re = Regex::new(r"(?m)^\d+\.\s+(.+)$").expect("valid regex: numbered list pattern");
    re.captures_iter(md).map(|c| c[1].to_string()).collect()
}

/// Extract task list items (\- [ ] or \- \[x\])
pub fn extract_tasks(md: &str) -> Vec<(bool, String)> {
    let re = Regex::new(r"(?m)^[-*]\s+\[([ x])\]\s+(.+)$").expect("valid regex: task list pattern");
    re.captures_iter(md)
        .map(|c| {
            let checked = c[1].starts_with('x');
            let text = c[2].to_string();
            (checked, text)
        })
        .collect()
}

/// Extract tables
pub fn extract_tables(md: &str) -> Vec<Vec<Vec<String>>> {
    let re = Regex::new(r"(?ms)^\|.+\|\n\|[-:|]+\|\n(.+?)\n\n").expect("valid regex: table block pattern");
    re.captures_iter(md)
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
    let re = Regex::new(r"(?m)^>\s+(.+)$").expect("valid regex: blockquote pattern");
    re.captures_iter(md).map(|c| c[1].to_string()).collect()
}

/// Extract bold (**text** or __text__)
pub fn extract_bold(md: &str) -> Vec<String> {
    let re = Regex::new(r"\*\*(.+?)\*\*|__(.+?)__").expect("valid regex: bold pattern **text** or __text__");
    re.captures_iter(md)
        .map(|c| c.get(1).map_or("", |m| m.as_str()).to_string())
        .collect()
}

/// Extract italic (*text* or _text_)
pub fn extract_italic(md: &str) -> Vec<String> {
    // Match *text* or _text_ patterns (single asterisk/underscore delimiters)
    let re = Regex::new(r"\*(.+?)\*|_(.+?)_").expect("valid regex: italic pattern *text* or _text_");
    re.captures_iter(md)
        .map(|c| c.get(1).map_or("", |m| m.as_str()).to_string())
        .collect()
}

/// Count words in markdown (excluding code blocks and URLs)
pub fn count_words(md: &str) -> usize {
    // Remove code blocks
    let re = Regex::new(r"```[\s\S]*?```").expect("valid regex: code block fence pattern");
    let md = re.replace_all(md, "");

    // Remove inline code
    let re = Regex::new(r"`[^`]+`").expect("valid regex: inline code backtick pattern");
    let md = re.replace_all(&md, "");

    // Remove URLs
    let re = Regex::new(r"https?://[^\s]+").expect("valid regex: URL pattern for word count");
    let md = re.replace_all(&md, "");

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
        let re = Regex::new(&pattern).expect("valid regex: header replacement pattern");
        html = re
            .replace(&html, |caps: &regex::Captures| {
                format!("<h{}>{}</h{}>", i, &caps[1], i)
            })
            .to_string();
    }

    // Bold
    html = Regex::new(r"\*\*(.+?)\*\*")
        .expect("valid regex: bold pattern **text**")
        .replace_all(&html, "<strong>$1</strong>")
        .to_string();

    // Italic
    html = Regex::new(r"\*(.+?)\*")
        .expect("valid regex: italic pattern *text*")
        .replace_all(&html, "<em>$1</em>")
        .to_string();

    // Links
    html = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)")
        .expect("valid regex: link pattern [text](url)")
        .replace_all(&html, "<a href=\"$2\">$1</a>")
        .to_string();

    // Code blocks
    html = Regex::new(r"```(\w*)\n([\s\S]*?)```")
        .expect("valid regex: fenced code block pattern")
        .replace_all(&html, "<pre><code class=\"lang-$1\">$2</code></pre>")
        .to_string();

    // Inline code
    html = Regex::new(r"`([^`]+)`")
        .expect("valid regex: inline code pattern")
        .replace_all(&html, "<code>$1</code>")
        .to_string();

    // List items
    html = Regex::new(r"(?m)^[-*]\s+(.+)$")
        .expect("valid regex: unordered list pattern")
        .replace_all(&html, "<li>$1</li>")
        .to_string();

    // Wrap list items in <ul>
    html = Regex::new(r"(<li>.*</li>\n)+")
        .expect("valid regex: list wrapping pattern")
        .replace_all(&html, "<ul>$0</ul>")
        .to_string();

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
mod tests {
    use super::*;

    #[test]
    fn test_title() {
        let md = "# Hello World\n\nSome content";
        assert_eq!(extract_title(md), Some("Hello World".to_string()));
    }

    #[test]
    fn test_links() {
        let md = "[Link](http://example.com)";
        let links = extract_links(md);
        assert_eq!(links.len(), 1);
    }

    #[test]
    fn test_count_words() {
        let md = "# Title\n\nSome **bold** text";
        assert!(count_words(md) >= 3);
    }
}
