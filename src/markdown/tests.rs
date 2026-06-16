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
