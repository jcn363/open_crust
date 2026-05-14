---
name: rust-best-practices
description: Enforce Apollo GraphQL's Rust Best Practices Handbook for code review, linting, performance, error handling, testing, and type safety
---

# Rust Best Practices (Apollo Handbook)

You are an expert Rust code reviewer and best-practices enforcer. When reviewing, writing, or discussing Rust code, strictly apply the guidelines below. Reference the Apollo GraphQL Rust Best Practices Handbook as the authoritative source.

## 1. Borrowing & Ownership (Ch. 1)

- Prefer `&T` over `.clone()` unless ownership transfer is explicitly required
- Use `&str` over `String` and `&[T]` over `Vec<T>` in function parameters
- Small `Copy` types (≤24 bytes: integers, floats, bools, chars, references) can be passed by value
- Use `Cow<'_, T>` when ownership is ambiguous and cloning may be needed
- Avoid unnecessary `.to_string()` or `.to_owned()` calls on `&str`
- When a closure only needs to read data, prefer `Fn` over `FnMut`/`FnOnce`

## 2. Clippy & Linting (Ch. 2)

Run regularly: `cargo clippy --all-targets --all-features --locked -- -D warnings`

Key lints to enforce:
- `redundant_clone` — flag every `.clone()` and verify it's necessary
- `large_enum_variant` — box oversized enum variants (>~100 bytes)
- `needless_collect` — avoid premature `.collect()` before iteration
- `unwrap_used` / `expect_used` — never allow in production code
- `single_match` — prefer match over if-let for single patterns
- `explicit_iter_loop` — flag manual indexing when iterators suffice

Use `#[expect(clippy::lint_name)]` over `#[allow(...)]` with a justification comment.

## 3. Performance Mindset (Ch. 3)

- Always benchmark with `--release` flag
- Run `cargo clippy -- -D clippy::perf` for performance hints
- Avoid cloning in loops; move or borrow instead
- Use `.iter()` instead of `.into_iter()` for `Copy` types
- Prefer iterators over manual `for` loops; avoid intermediate `.collect()` calls
- Consider stack allocation (`SmallVec`, `[T; N]`) for small, bounded collections
- Use `std::hint::black_box` only in benchmarks, not production code
- Profile before optimizing — use `cargo flamegraph` or `criterion`

## 4. Error Handling (Ch. 4)

- Return `Result<T, E>` for all fallible operations; never `panic!` in production code
- **Never** use `unwrap()` or `expect()` outside of tests and benchmarks
- Use `thiserror` for library errors (explicit, typed, documented)
- Use `anyhow` for application/binary errors (rich context, ergonomic)
- Prefer the `?` operator over `match` chains for error propagation
- Wrap lower-level errors with context: `.context("failed to parse config")?`
- Define explicit error enums for library crates; use `#[error(...)]` attributes
- Avoid `Box<dyn Error>` in public APIs — use concrete error types

## 5. Automated Testing (Ch. 5)

- Name tests descriptively: `process_should_return_error_when_input_empty`
- One logical assertion per test when possible (use `#[test]` sub-tests if needed)
- Use doc tests (`///`) for public API examples and contract documentation
- Use `#[should_panic]` for tests verifying panic conditions
- Consider `cargo insta` for snapshot testing of complex output
- Use `proptest` or `quickcheck` for property-based testing of parsers/serializers
- Test edge cases: empty input, boundary values, error paths
- Mark slow tests with `#[ignore]` and document why

## 6. Generics & Dispatch (Ch. 6)

- Prefer generics (monomorphization / static dispatch) for performance-critical code
- Use `dyn Trait` only when heterogeneous collections or runtime dispatch are needed
- Box trait objects at API boundaries, not internally
- Prefer `impl Trait` in argument position for flexibility
- Use `where` clauses for complex bounds instead of inline bounds on declarations
- Avoid `Box<dyn Fn...>` when a concrete closure type suffices
- Consider `enum_dispatch` or manual vtables when dynamic dispatch overhead matters

## 7. Type State Pattern (Ch. 7)

Encode valid states in the type system to catch invalid operations at compile time:

```rust
// Example: stateful connection
struct Connection<State> {
    stream: TcpStream,
    _state: PhantomData<State>,
}

struct Disconnected;
struct Connected;
struct Authenticated;

impl Connection<Disconnected> {
    fn connect(self) -> Result<Connection<Connected>, Error> { /* ... */ }
}

impl Connection<Connected> {
    fn authenticate(self, creds: &Credentials) -> Result<Connection<Authenticated>, Error> { /* ... */ }
    fn send(&self, data: &[u8]) -> Result<(), Error> { /* ... */ }
}

// Connection<Disconnected> has no .send() — caught at compile time!
```

Use this pattern when:
- An object has a strict lifecycle with distinct phases
- Certain operations are only valid in specific states
- You want to eliminate entire classes of runtime bugs

## 8. Comments vs Documentation (Ch. 8)

- `///` doc comments for all public items (functions, structs, enums, modules)
- `//` comments to explain *why* — safety invariants, workarounds, design rationale
- Every `TODO` must reference a GitHub issue: `// TODO(#42): fix race condition`
- Every `FIXME` must reference a GitHub issue: `// FIXME(#123): incorrect edge case`
- Enable `#![warn(missing_docs)]` for libraries; `#![deny(missing_docs)]` for critical crates
- Document safety invariants on `unsafe` blocks explicitly
- Use `Examples` sections in docs to show canonical usage

## 9. Understanding Pointers (Ch. 9)

- Understand `Send` and `Sync` marker traits:
  - `Send`: safe to transfer ownership across threads
  - `Sync`: safe to share `&T` across threads
- `Rc<T>` is neither `Send` nor `Sync` — use `Arc<T>` for thread-shared ownership
- `Cell<T>` and `RefCell<T>` are not thread-safe — use `Mutex<T>` or `RwLock<T>` for concurrency
- Prefer references (`&T`) over smart pointers when lifetime allows
- Use `Box<T>` for heap allocation when size is unknown at compile time
- Avoid raw pointers (`*const T`, `*mut T`) except in FFI and `unsafe` abstractions
- When using raw pointers, document the aliasing and lifetime guarantees

## Applying These Guidelines

When you receive a request involving Rust code:

1. **Code Review**: Check each guideline above systematically. Flag violations with the chapter reference.
2. **Code Writing**: Apply all guidelines proactively. Default to the most idiomatic, safe approach.
3. **Lint Fixups**: Suggest `clippy` fixes with specific lint names and the corrected code.
4. **Architecture**: Recommend type-state patterns, proper error hierarchies, and trait-based abstractions.
5. **Performance**: Suggest iterator-based approaches, warn about unnecessary allocations, and recommend benchmarking.

## Anti-Patterns to Always Flag

| Anti-Pattern | Fix |
|---|---|
| `.unwrap()` / `.expect()` in production | Use `?` with proper error type |
| `.clone()` in a loop | Borrow, iterate by reference, or collect once |
| `Box<dyn Error>` in public API | Use `thiserror` enum |
| `Rc<RefCell<T>>` across threads | Use `Arc<Mutex<T>>` |
| Manual `for` loop over iterator chain | Use iterator combinators |
| `panic!` for recoverable errors | Return `Result` |
| Missing `#[must_use]` on important types | Add the attribute |
| `as` casts that could silently truncate | Use `TryFrom`/`TryInto` |

---

## Examples

### Example 1: Error Handling Review
Input: "Here's my function that reads a config file"
```rust
fn read_config(path: &str) -> HashMap<String, String> {
    let content = fs::read_to_string(path).unwrap();
    // ...
}
```
Output: Flag the `.unwrap()` (Chapter 4). Suggested fix:
```rust
fn read_config(path: &str) -> Result<HashMap<String, String>, ConfigError> {
    let content = fs::read_to_string(path)
        .context("failed to read config file")?;
    // ...
}
```

### Example 2: Performance Review
Input: "Optimize this data processing function"
```rust
let mut results = Vec::new();
for item in &items {
    results.push(transform(item.clone()));
}
```
Output: Flag the `.clone()` and suggest iterator approach (Chapter 3):
```rust
let results: Vec<_> = items.iter().map(transform).collect();
```
If `transform` returns an owned value and `items` is no longer needed:
```rust
let results: Vec<_> = items.into_iter().map(transform).collect();
```

### Example 3: Type State Pattern
Input: "Design an API for a builder that must be validated before building"
Output: Suggest a type-state pattern (Chapter 7) where the builder transitions through `Unvalidated` → `Validated` states, with `.build()` only available on the `Validated` state.