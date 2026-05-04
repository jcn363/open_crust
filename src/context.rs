use std::fs;

pub fn inject_file_context(prompt: &str) -> String {
    let mut enriched_prompt = String::from(prompt);
    let mut files_to_inject = Vec::new();

    for word in prompt.split_whitespace() {
        if word.starts_with('@') && word.len() > 1 {
            let path = &word[1..];
            let path = path.trim_end_matches(&[',', '.', ';', ':', '?', '!'][..]);
            
            if fs::metadata(path).is_ok() {
                if !files_to_inject.contains(&path.to_string()) {
                    files_to_inject.push(path.to_string());
                }
            }
        }
    }

    if !files_to_inject.is_empty() {
        enriched_prompt.push_str("\n\n---\nFile Contexts:\n");
        for path in files_to_inject {
            if let Ok(content) = fs::read_to_string(&path) {
                enriched_prompt.push_str(&format!("\n<file path=\"{}\">\n{}\n</file>\n", path, content));
            }
        }
    }

    enriched_prompt
}
