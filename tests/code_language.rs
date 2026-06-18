//! Integration tests for code file detection and language classification.
//!
//! These tests exercise `is_code_file` and related utilities that the
//! RAG system uses to determine which files to index.

use opencrust::rag;

/// Standard code extensions are recognized.
#[test]
fn standard_code_extensions() {
    assert!(rag::is_code_file("main.rs"), ".rs");
    assert!(rag::is_code_file("app.py"), ".py");
    assert!(rag::is_code_file("index.js"), ".js");
    assert!(rag::is_code_file("component.tsx"), ".tsx");
    assert!(rag::is_code_file("lib.go"), ".go");
    assert!(rag::is_code_file("main.c"), ".c");
    assert!(rag::is_code_file("main.cpp"), ".cpp");
    assert!(rag::is_code_file("lib.h"), ".h");
}

/// Modern extensions that were added in recent updates are recognized.
#[test]
fn modern_code_extensions() {
    assert!(rag::is_code_file("main.zig"), ".zig");
    assert!(rag::is_code_file("app.dart"), ".dart");
    assert!(rag::is_code_file("script.lua"), ".lua");
    assert!(rag::is_code_file("analysis.r"), ".r");
    assert!(rag::is_code_file("app.scala"), ".scala");
    assert!(rag::is_code_file("page.elm"), ".elm");
    assert!(rag::is_code_file("core.clj"), ".clj");
    assert!(rag::is_code_file("app.ex"), ".ex");
    assert!(rag::is_code_file("shader.wgsl"), ".wgsl");
    assert!(rag::is_code_file("app.roc"), ".roc");
    assert!(rag::is_code_file("main.mojo"), ".mojo");
    assert!(rag::is_code_file("doc.typst"), ".typst");
}

/// Non-code extensions return false.
#[test]
fn non_code_extensions() {
    assert!(!rag::is_code_file("image.png"), ".png");
    assert!(!rag::is_code_file("doc.pdf"), ".pdf");
    assert!(!rag::is_code_file("style.css"), ".css");
    assert!(!rag::is_code_file("index.html"), ".html");
}

/// Markdown and JSON are treated as code for RAG purposes.
#[test]
fn markup_extensions() {
    assert!(rag::is_code_file("readme.md"), ".md");
    assert!(rag::is_code_file("config.json"), ".json");
    assert!(rag::is_code_file("config.yaml"), ".yaml");
    assert!(rag::is_code_file("config.toml"), ".toml");
}

/// Files without extensions return false.
#[test]
fn files_without_extension() {
    assert!(!rag::is_code_file("Makefile"), "Makefile");
    assert!(!rag::is_code_file("Dockerfile"), "Dockerfile");
    assert!(!rag::is_code_file("LICENSE"), "LICENSE");
}

/// Hidden files (dotfiles) return false since Path::extension() returns None.
#[test]
fn hidden_files_without_extension() {
    assert!(!rag::is_code_file(".bashrc"), ".bashrc");
    assert!(!rag::is_code_file(".gitignore"), ".gitignore");
    assert!(!rag::is_code_file(".env"), ".env");
}
