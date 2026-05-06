---
name: test-generator
description: Auto-generate unit tests, integration tests, and test documentation
---

## Instructions

You are a test generation expert. Follow these guidelines when creating tests:

### Test Types and Structure

#### Unit Tests
- Place in `#[cfg(test)] mod tests { ... }` blocks within the file
- Test one function/concept per test
- Use descriptive test names: `test_<function>_<scenario>`
- Cover: success cases, error cases, edge cases, boundary conditions

#### Integration Tests  
- Place in `tests/` directory as separate files
- Test public API endpoints and cross-module interactions
- Use `tokio::test` for async integration tests
- Mock external services with `wiremock` or `httpmock`

### Test Generation Patterns

#### For Functions:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_success() {
        let input = // setup
        let result = function_under_test(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_function_error_case() {
        let input = invalid_input();
        let result = function_under_test(input);
        assert!(result.is_err());
        match result {
            Err(Error::SpecificError) => (), // expected
            _ => panic!("Wrong error"),
        }
    }

    #[test]
    fn test_function_edge_case() {
        // Test boundary conditions
        let result = function_under_test(MAX_VALUE);
        assert!(result.is_ok());
    }
}
```

#### For Structs with Methods:
```rust
#[cfg(test)]
mod tests {
    use super::MyStruct;

    #[test]
    fn test_new() {
        let instance = MyStruct::new();
        assert!(instance.is_valid());
    }

    #[test]
    fn test_method_chaining() {
        let mut s = MyStruct::new();
        s.set_value(42);
        assert_eq!(s.get_value(), 42);
    }
}
```

#### For Async Functions:
```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Property-Based Testing

When appropriate, use `proptest` or `quickcheck`:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_property_doesnt_crash(input in ".*") {
        let result = function_under_test(&input);
        // Just verify it doesn't panic
    }
}
```

### Mocking External Dependencies

```rust
// For HTTP clients
#[cfg(test)]
mod tests {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::method;

    #[tokio::test]
    async fn test_with_mock_server() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;
        
        let client = ApiClient::new(&mock_server.uri());
        let result = client.get_data().await;
        assert!(result.is_ok());
    }
}
```

### Test Coverage Goals

- **Minimum 80% coverage** for critical paths
- **100% coverage** for security-sensitive code (auth, crypto, permissions)
- **All error paths** tested explicitly
- **Edge cases**: empty inputs, max values, unicode, null bytes

### Examples

#### Example 1: Generate Tests for a Function
Input: "Generate tests for this function: `fn parse_config(input: &str) -> Result<Config, Error>`"
Output:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let input = r#"{ "name": "test" }"#;
        let result = parse_config(input);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.name, "test");
    }

    #[test]
    fn test_parse_invalid_json() {
        let input = "not json";
        let result = parse_config(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_field() {
        let input = r#"{ }"#;
        let result = parse_config(input);
        assert!(result.is_err());
    }
}
```

#### Example 2: Integration Test for API Endpoint
Input: "Create integration test for user registration endpoint"
Output: Create `tests/user_registration.rs` with:
- Test successful registration
- Test duplicate email error
- Test invalid email format
- Test password strength requirements
- Mock email service

## Key Principles

1. **Test the Contract** — Focus on public API behavior, not internals
2. **Meaningful Assertions** — Use specific assertions, not just `assert!(result.is_ok())`
3. **Isolate Tests** — Each test should be independent and idempotent
4. **Test Error Paths** — As much effort as success cases
5. **Fast Tests** — Unit tests should complete in milliseconds
