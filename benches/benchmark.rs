//! Criterion benchmarks for OpenCrust core paths.
//!
//! Measures startup-time-critical operations (config reading, JSON parsing)
//! and render-time-critical operations (text diffing). Run with:
//!
//!     cargo bench
//!
//! or with HTML report:
//!
//!     cargo bench --bench benchmark -- --profile-time 30

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Config deserialization (mimics production Config)
// ---------------------------------------------------------------------------

#[derive(Deserialize, Default)]
#[allow(dead_code)]
struct BenchConfig {
    provider: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
    theme: Option<String>,
    context_budget: Option<u64>,
    rag_enabled: Option<bool>,
}

const CONFIG_JSON: &str = r#"{
    "provider": "openrouter",
    "model": "anthropic/claude-3.5-sonnet",
    "api_key": "sk-or-v1-test-key",
    "theme": "dark",
    "context_budget": 8000,
    "rag_enabled": true
}"#;

fn bench_config_parse(c: &mut Criterion) {
    c.bench_function("config_parse", |b| {
        b.iter(|| {
            let cfg: BenchConfig = serde_json::from_str(black_box(CONFIG_JSON)).unwrap();
            black_box(cfg);
        });
    });
}

// ---------------------------------------------------------------------------
// JSON string operations (mirrors json_utils module)
// ---------------------------------------------------------------------------

const SAMPLE_JSON: &str = r#"{
    "name": "OpenCrust",
    "version": "1.0.0",
    "features": {
        "tui": true,
        "mcp": true,
        "rag": false
    },
    "providers": [
        {"name": "openai", "key": "sk-..."},
        {"name": "anthropic", "key": "sk-ant-..."},
        {"name": "groq", "key": "gsk-..."}
    ],
    "settings": {
        "theme": "dark",
        "font_size": 14,
        "tab_size": 4,
        "word_wrap": true
    }
}"#;

fn bench_json_validate(c: &mut Criterion) {
    c.bench_function("json_validate", |b| {
        b.iter(|| {
            let v: serde_json::Value = serde_json::from_str(black_box(SAMPLE_JSON)).unwrap();
            black_box(v);
        });
    });
}

fn bench_json_format(c: &mut Criterion) {
    c.bench_function("json_format", |b| {
        b.iter(|| {
            let v: serde_json::Value = serde_json::from_str(SAMPLE_JSON).unwrap();
            let formatted = serde_json::to_string_pretty(&v).unwrap();
            black_box(formatted);
        });
    });
}

fn bench_json_get_path(c: &mut Criterion) {
    c.bench_function("json_get_path", |b| {
        b.iter(|| {
            let v: serde_json::Value = serde_json::from_str(SAMPLE_JSON).unwrap();
            let r = v.pointer("/features/tui");
            black_box(r);
        });
    });
}

fn bench_json_compact(c: &mut Criterion) {
    c.bench_function("json_compact", |b| {
        b.iter(|| {
            let v: serde_json::Value = serde_json::from_str(SAMPLE_JSON).unwrap();
            let compact = serde_json::to_string(&v).unwrap();
            black_box(compact);
        });
    });
}

fn bench_json_compare(c: &mut Criterion) {
    let left = r#"{"a":1,"b":2,"c":3}"#;
    let right = r#"{"a":1,"b":3,"c":4}"#;
    c.bench_function("json_compare", |b| {
        b.iter(|| {
            let l: serde_json::Value = serde_json::from_str(left).unwrap();
            let r: serde_json::Value = serde_json::from_str(right).unwrap();
            let equal = l == r;
            black_box(equal);
        });
    });
}

fn bench_json_merge(c: &mut Criterion) {
    let base = r#"{"a":1,"b":2}"#;
    let patch = r#"{"b":3,"c":4}"#;
    c.bench_function("json_merge", |b| {
        b.iter(|| {
            let mut base: serde_json::Value = serde_json::from_str(base).unwrap();
            let patch: serde_json::Value = serde_json::from_str(patch).unwrap();
            if let serde_json::Value::Object(ref mut m) = base {
                if let serde_json::Value::Object(p) = patch {
                    m.extend(p);
                }
            }
            black_box(base);
        });
    });
}

// ---------------------------------------------------------------------------
// Text diff (similar crate — used in review popup)
// ---------------------------------------------------------------------------

/// Shorter text sample for realistic file-sized diffing
const TEXT_A: &str = r#"use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
"#;

const TEXT_B: &str = r#"use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
"#;

fn bench_text_diff(c: &mut Criterion) {
    c.bench_function("text_diff", |b| {
        b.iter(|| {
            let diff = similar::TextDiff::from_lines(black_box(TEXT_A), black_box(TEXT_B));
            let mut count = 0usize;
            for change in diff.iter_all_changes() {
                count += change.value().len();
            }
            black_box(count);
        });
    });
}

criterion_group! {
    name = opencrust_benches;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_secs(3))
        .sample_size(50);
    targets =
        bench_config_parse,
        bench_json_validate,
        bench_json_format,
        bench_json_get_path,
        bench_json_compact,
        bench_json_compare,
        bench_json_merge,
        bench_text_diff,
}

criterion_main!(opencrust_benches);
