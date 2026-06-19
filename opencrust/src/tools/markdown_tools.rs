//! Markdown tool handlers (md_title, md_headings, md_links, etc.)

use serde_json::Value;

use crate::markdown;

/// Execute a markdown tool by name. Returns Some(result) if handled.
pub fn execute_markdown_tool(name: &str, args: &Value) -> Option<String> {
    match name {
        "md_title" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            Some(match markdown::extract_title(md) {
                Some(title) => title,
                None => "No title found".to_string(),
            })
        }
        "md_headings" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            let headings = markdown::extract_headings(md);
            Some(if headings.is_empty() {
                "No headings found".to_string()
            } else {
                headings
                    .iter()
                    .map(|(l, t)| format!("{} {}", "#".repeat(*l as usize), t))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }
        "md_links" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            let links = markdown::extract_links(md);
            Some(if links.is_empty() {
                "No links found".to_string()
            } else {
                links
                    .iter()
                    .map(|(text, url)| format!("[{}]({})", text, url))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }
        "md_word_count" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            let count = markdown::count_words(md);
            Some(format!("{} words", count))
        }
        "md_is_valid" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            Some(match markdown::is_valid(md) {
                true => "Valid markdown".to_string(),
                false => "Invalid markdown".to_string(),
            })
        }
        "md_to_html" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            Some(markdown::to_html(md))
        }
        "md_extract_code" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            let code_blocks = markdown::extract_code_blocks(md);
            Some(if code_blocks.is_empty() {
                "No code blocks found".to_string()
            } else {
                code_blocks
                    .iter()
                    .map(|(_, code)| code.as_str())
                    .collect::<Vec<_>>()
                    .join("\n---\n")
            })
        }
        "md_frontmatter" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            Some(match markdown::extract_frontmatter(md) {
                Some((fm, _)) => fm,
                None => "No frontmatter found".to_string(),
            })
        }
        "md_tables" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            let tables = markdown::extract_tables(md);
            Some(if tables.is_empty() {
                "No tables found".to_string()
            } else {
                let output: Vec<String> = tables
                    .iter()
                    .map(|t| {
                        t.iter()
                            .map(|row| row.join(" | "))
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .collect();
                output.join("\n\n")
            })
        }
        "md_images" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            let images = markdown::extract_images(md);
            Some(if images.is_empty() {
                "No images found".to_string()
            } else {
                images
                    .iter()
                    .map(|(alt, url)| format!("![{}]({})", alt, url))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }
        "md_tasks" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            let tasks = markdown::extract_tasks(md);
            Some(if tasks.is_empty() {
                "No tasks found".to_string()
            } else {
                tasks
                    .iter()
                    .map(|(done, text)| format!("[{}] {}", if *done { "x" } else { " " }, text))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }
        "md_list_items" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            let items = markdown::extract_list_items(md);
            Some(if items.is_empty() {
                "No list items found".to_string()
            } else {
                items.join("\n")
            })
        }
        "md_numbered" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            let items = markdown::extract_numbered_items(md);
            Some(if items.is_empty() {
                "No numbered items found".to_string()
            } else {
                items.join("\n")
            })
        }
        "md_quotes" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            let quotes = markdown::extract_quotes(md);
            Some(if quotes.is_empty() {
                "No blockquotes found".to_string()
            } else {
                quotes.join("\n")
            })
        }
        "md_urls" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            let urls = markdown::extract_urls(md);
            Some(if urls.is_empty() {
                "No URLs found".to_string()
            } else {
                urls.join("\n")
            })
        }
        "md_inline_code" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            let code = markdown::extract_inline_code(md);
            Some(if code.is_empty() {
                "No inline code found".to_string()
            } else {
                code.join("\n")
            })
        }
        "md_bold" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            let bold = markdown::extract_bold(md);
            Some(if bold.is_empty() {
                "No bold text found".to_string()
            } else {
                bold.join("\n")
            })
        }
        "md_italic" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            let italic = markdown::extract_italic(md);
            Some(if italic.is_empty() {
                "No italic text found".to_string()
            } else {
                italic.join("\n")
            })
        }
        "md_summary" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            Some(markdown::get_summary(md))
        }
        "md_headings_tree" => {
            let md = args.get("markdown").and_then(|v| v.as_str()).unwrap_or("");
            let tree = markdown::get_headings_tree(md);
            Some(if tree.is_empty() {
                "No headings found".to_string()
            } else {
                tree.iter()
                    .map(|(level, text, depth)| {
                        format!(
                            "{} {} {}",
                            "  ".repeat(*depth),
                            "#".repeat(*level as usize),
                            text
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }
        _ => None,
    }
}
