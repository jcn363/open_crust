# Testing Guide

How to write and run tests for OpenCrust: unit tests, integration tests, property-based tests, and PR verification.

**For coding standards:** See **AGENTS.md**.  
**For development guide:** See **docs/DEVELOPMENT.md**.  
**For project structure:** See **docs/MODULES.md**.

---

## Quick Start

### Run All Tests

```bash
cargo test
```

**Output (439 tests total — 420 unit + 19 integration):**
```
running 420 tests (unit)
test result: ok. 420 passed; 0 failed; 0 ignored; 0 measured; finished in 2.02s

running 19 tests (integration)
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; finished in 0.01s
```

### Run Tests for One Module

```bash
cargo test permissions::
```

### Run Tests with Output

```bash
cargo test -- --nocapture
```

Useful to see `println!` debugging output.

### Run a Specific Test

```bash
cargo test test_file_pattern_match
```

---

## Unit Test Pattern

### Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path() {
        // Arrange: Set up test data
        let input = "test_data";
        
        // Act: Call the function being tested
        let result = parse_input(input);
        
        // Assert: Verify the result
        assert_eq!(result.len(), 9);
    }

    #[test]
    fn test_error_case() {
        // Test error handling
        let result = parse_input("");
        assert!(result.is_err());
    }

    #[test]
    #[should_panic(expected = "invalid state")]
    fn test_panic() {
        // Test that function panics as expected
        invalid_operation();
    }
}
```

### Example: Testing Permissions

```rust
// In permissions.rs
pub fn matches_pattern(pattern: &str, path: &str) -> bool {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert!(matches_pattern("src/main.rs", "src/main.rs"));
    }

    #[test]
    fn test_glob_pattern() {
        assert!(matches_pattern("src/**", "src/subdir/file.rs"));
        assert!(matches_pattern("src/**", "src/file.rs"));
    }

    #[test]
    fn test_no_match() {
        assert!(!matches_pattern("src/**", "docs/readme.md"));
    }

    #[test]
    fn test_deny_pattern() {
        assert!(!matches_pattern("!.env", ".env"));
    }
}
```

### Coverage

- **Happy path:** Normal input, expected output
- **Error case:** Invalid input, graceful failure
- **Edge case:** Boundary conditions (empty string, max size)
- **Panic:** When the function should panic

---

## Integration Tests

### Structure

```
opencrust/
├── src/
│   └── main.rs
├── tests/
│   ├── common/mod.rs          # Shared test utilities
│   ├── integration_chat.rs    # Test chat flow
│   ├── integration_tools.rs   # Test tool execution
│   └── integration_config.rs  # Test config loading
└── Cargo.toml
```

### Example: Test Full Chat Flow

```rust
// tests/integration_chat.rs
use opencrust::app::{App, Message};
use std::sync::Arc;

fn setup() -> App {
    // Create app with test config
    let config = Config::test_default();
    App::new(config)
}

#[test]
fn test_chat_message_stored() {
    let mut app = setup();
    
    // Simulate user message
    app.add_message(Message {
        role: "user".into(),
        content: "Hello".into(),
    });
    
    // Verify message stored
    assert_eq!(app.messages.len(), 1);
    assert_eq!(app.messages[0].content, "Hello");
}

#[test]
fn test_multiple_messages() {
    let mut app = setup();
    
    app.add_message(Message { role: "user".into(), content: "Hi".into() });
    app.add_message(Message { role: "assistant".into(), content: "Hello!".into() });
    app.add_message(Message { role: "user".into(), content: "How are you?".into() });
    
    assert_eq!(app.messages.len(), 3);
}
```

---

## Testing Tools

### Test Tool Execution

```rust
// In tool_executor.rs tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_read_tool() {
        let tempfile = std::fs::File::create("/tmp/test.txt").unwrap();
        std::io::Write::all(&mut std::fs::File::open("/tmp/test.txt").unwrap(), b"Hello").unwrap();
        
        let result = execute_tool_file_read("/tmp/test.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello");
        
        std::fs::remove_file("/tmp/test.txt").ok();
    }

    #[test]
    fn test_permission_denied() {
        let config = Config {
            permissions: Permissions {
                file_patterns: vec!["src/**".into()],
                deny_patterns: vec![".env".into()],
            },
            ..Default::default()
        };
        
        let result = execute_tool_with_config(".env", &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("permission denied"));
    }
}
```

### Test Custom Tool Discovery

```rust
#[test]
fn test_discover_custom_tools() {
    // Create temp directory with test tools
    let tmpdir = tempfile::tempdir().unwrap();
    let tool_path = tmpdir.path().join("my_tool");
    
    // Write executable script
    std::fs::write(&tool_path, "#!/bin/bash\necho hello").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::PermissionsExt::set_mode(
        &mut std::fs::metadata(&tool_path).unwrap().permissions(),
        0o755,
    );
    
    // Discover tools
    let tools = discover_tools(tmpdir.path()).unwrap();
    
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "my_tool");
}
```

---

## Property-Based Testing

### Using Proptest

```toml
[dev-dependencies]
proptest = "1.0"
```

### Example: Glob Pattern Matching

```rust
use proptest::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn test_pattern_always_matches_itself(pattern in ".*") {
            // Any pattern should match an identical path
            prop_assume!(!pattern.is_empty());
            assert!(matches_pattern(&pattern, &pattern));
        }

        #[test]
        fn test_glob_star_matches_anything(path in ".*") {
            // "**" should match any path
            assert!(matches_pattern("**", &path));
        }
    }
}
```

### Example: Config Parsing Robustness

```rust
proptest! {
    #[test]
    fn test_config_parse_never_panics(input in ".*") {
        // Parse should never panic, even with garbage input
        let _result = parse_config(&input);
    }

    #[test]
    fn test_tool_execution_with_arbitrary_args(args in prop::collection::vec(".*", 0..10)) {
        // Tool execution should handle any arguments gracefully
        let result = execute_tool("test_tool", args);
        // Should either succeed or return error, never panic
        assert!(result.is_ok() || result.is_err());
    }
}
```

---

## Mocking & Stubbing

### Mock LLM Provider

```rust
use mockall::predicate::*;
use mockall::mock;

mock! {
    LlmProvider {}
    impl LlmProvider for LlmProvider {
        async fn call(&self, request: LlmRequest) -> Result<LlmResponse>;
    }
}

#[test]
fn test_llm_integration() {
    let mut mock_provider = MockLlmProvider::new();
    
    // Expect call with specific args
    mock_provider
        .expect_call()
        .with(eq(LlmRequest {
            model: "test-model".into(),
            ..Default::default()
        }))
        .times(1)
        .returning(|_| Ok(LlmResponse {
            content: "Hello!".into(),
            tool_calls: vec![],
            tokens_used: TokenUsage { input: 10, output: 5 },
        }));
    
    // Test code using mock_provider
}
```

### Stub Config

```rust
impl Config {
    #[cfg(test)]
    pub fn test_default() -> Self {
        Self {
            default_model: "test".into(),
            default_provider: "test".into(),
            context_budget: 1000,
            ..Default::default()
        }
    }
    
    #[cfg(test)]
    pub fn test_with_permissions(patterns: Vec<String>) -> Self {
        let mut config = Self::test_default();
        config.permissions.file_patterns = patterns;
        config
    }
}

#[test]
fn test_with_custom_config() {
    let config = Config::test_with_permissions(vec!["src/**".into()]);
    assert_eq!(config.permissions.file_patterns.len(), 1);
}
```

---

## Async Testing

### Testing Async Functions

```rust
#[tokio::test]
async fn test_async_llm_call() {
    let client = LlmClient::new(&config);
    
    let result = client.call(request).await;
    
    assert!(result.is_ok());
    assert!(!result.unwrap().content.is_empty());
}

#[tokio::test]
async fn test_tool_execution_timeout() {
    let config = Config {
        timeout_seconds: 1,
        ..Default::default()
    };
    
    // This tool sleeps for 5 seconds
    let result = execute_tool_timeout(&config, "sleep 5").await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("timeout"));
}
```

### Testing Concurrent Execution

```rust
#[tokio::test]
async fn test_concurrent_tool_execution() {
    let config = Config::test_default();
    
    // Execute multiple tools concurrently
    let tasks = vec![
        execute_tool_async(&config, "tool1"),
        execute_tool_async(&config, "tool2"),
        execute_tool_async(&config, "tool3"),
    ];
    
    let results = futures::future::join_all(tasks).await;
    
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));
}
```

---

## Performance Tests

### Benchmark Tool Execution

```toml
[dev-dependencies]
criterion = "0.5"
```

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_file_read(c: &mut Criterion) {
    c.bench_function("file_read_1mb", |b| {
        b.iter(|| {
            execute_tool_file_read(black_box("/tmp/test_1mb.txt"))
        })
    });
}

criterion_group!(benches, bench_file_read);
criterion_main!(benches);
```

**Run benchmarks:**

```bash
cargo bench

# Output:
# file_read_1mb               time:   [2.3 ms 2.4 ms 2.5 ms]
```

### Memory Tests

```rust
#[test]
fn test_memory_usage() {
    // Test that loading a large config doesn't leak memory
    
    let config1 = Config::load("large_config.json").unwrap();
    let usage1 = memory_usage();
    
    drop(config1);
    
    let config2 = Config::load("large_config.json").unwrap();
    let usage2 = memory_usage();
    
    drop(config2);
    
    let usage3 = memory_usage();
    
    // Usage should not grow significantly after drop
    assert!(usage3 < usage2 + 1000000);  // Allow 1MB variance
}
```

---

## PR Verification Checklist

**Before submitting a PR, run:**

```bash
# 1. Format check
cargo fmt -- --check

# 2. Linting
cargo clippy -- -D warnings

# 3. All tests
cargo test

# 4. Doc tests
cargo test --doc

# 5. No clippy warnings
cargo clippy

# 6. Build release
cargo build --release

# 7. Test the feature manually
./target/release/opencrust
```

**Automated via CI (GitHub Actions):**
- Runs all above checks
- Tests on multiple Rust versions
- Tests on macOS, Linux, Windows
- Reports coverage

---

## Common Test Patterns

### Test Setup and Teardown

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            // Run once for all tests
            env_logger::try_init().ok();
        });
    }

    #[test]
    fn test_with_logging() {
        setup();
        // Logs now available
    }
}
```

### Test Fixtures

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct TestFixture {
        config: Config,
        app: App,
    }

    impl TestFixture {
        fn new() -> Self {
            let config = Config::test_default();
            let app = App::new(config.clone());
            Self { config, app }
        }
    }

    #[test]
    fn test_with_fixture() {
        let fixture = TestFixture::new();
        assert_eq!(fixture.config.context_budget, 1000);
    }
}
```

### Parameterized Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patterns() {
        let test_cases = vec![
            ("src/**", "src/main.rs", true),
            ("src/**", "tests/test.rs", false),
            ("**", "anything/goes/here.txt", true),
        ];

        for (pattern, path, expected) in test_cases {
            assert_eq!(
                matches_pattern(pattern, path),
                expected,
                "Pattern {} with {} failed",
                pattern,
                path
            );
        }
    }
}
```

---

## Debugging Failed Tests

### Run with Backtrace

```bash
RUST_BACKTRACE=1 cargo test test_name

# Full backtrace:
RUST_BACKTRACE=full cargo test test_name
```

### Print Debug Info

```rust
#[test]
fn test_debug_output() {
    let result = some_function();
    eprintln!("Debug: {:?}", result);  // Print to stderr
    assert_eq!(result, expected);
}

// Run with output:
// cargo test test_debug_output -- --nocapture
```

### Run Single Test in Debugger

```bash
# Build test binary
cargo test --no-run

# Debug with GDB
gdb ./target/debug/deps/opencrust-<hash>
(gdb) run test_name
```

---

## Test Coverage

### Generate Coverage Report

```bash
# Install tarpaulin (coverage tool)
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html

# Opens coverage/index.html
```

**Goal:** Aim for >70% coverage on critical modules (permissions, config, tools)

---

## References

- **AGENTS.md** — Coding standards (includes test expectations)
- **CONTRIBUTING.md** — PR requirements include test verification
- **docs/DEVELOPMENT.md** — Adding new features (include tests)
