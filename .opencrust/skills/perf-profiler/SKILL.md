---
name: perf-profiler
description: Identify performance bottlenecks, suggest optimizations, and analyze resource usage
---

## Instructions

You are a performance optimization expert. Follow these guidelines when analyzing Rust code:

### Performance Analysis Tools

#### CPU Profiling
- Use `cargo-flamegraph` for flamegraphs: `cargo flamegraph --bin myapp`
- Use `perf` on Linux: `perf record --call-graph=dwarf ./target/release/myapp`
- Use `valgrind --tool=callgrind` for call analysis
- Use `criterion` for micro-benchmarks

#### Memory Profiling
- Use `heaptrack` for heap analysis
- Use `jemalloc` with profiling enabled
- Use `dhat` for detailed heap profiling
- Check for leaks with `valgrind --tool=memcheck`

#### Binary Analysis
```bash
# Binary size analysis
cargo bloat --release --crates

# Dependency tree analysis
cargo tree -e normal --depth 1

# Compile time analysis
cargo build -Ztimings
```

### Common Performance Issues

#### 1. Unnecessary Allocations
```rust
// BAD: Creates new String
fn get_name() -> String {
    "Alice".to_string()  // Allocation
}

// GOOD: Return &str when possible
fn get_name() -> &'static str {
    "Alice"  // No allocation
}
```

#### 2. Clone-on-Write Opportunities
```rust
// BAD: Always clones
fn process(input: &str) -> String {
    input.to_string()  // Clones even if not needed
}

// GOOD: Use Cow
fn process(input: &str) -> std::borrow::Cow<str> {
    if input.contains("special") {
        input.to_string().into()  // Clone only when needed
    } else {
        input.into()  // Zero-copy
    }
}
```

#### 3. Iterator Chain Efficiency
```rust
// BAD: Multiple passes
let len = items.iter().filter(|x| x > 10).count();
let sum: i32 = items.iter().filter(|x| x > 10).sum();

// GOOD: Single pass
let (len, sum) = items.iter()
    .filter(|x| x > 10)
    .fold((0, 0), |(cnt, total), x| (cnt + 1, total + x));
```

#### 4. Stack vs Heap
```rust
// BAD: Heap allocation for small fixed-size array
let data = vec![1, 2, 3, 4, 5];  // Heap

// GOOD: Stack allocation
let data = [1, 2, 3, 4, 5];  // Stack (faster)
```

### Optimization Strategies

#### Data Structures
- **Use `Vec<T>`** for dynamic arrays (cache-friendly)
- **Use `SmallVec<[T; N]>`** for small arrays (avoids heap)
- **Use `IndexMap`** instead of `HashMap` when order matters
- **Use `IndexSet`** for deduplication with order
- **Use `BTreeMap`** for sorted data or range queries

#### Concurrency
```rust
// Use rayon for data parallelism
use rayon::prelude::*;

let sum: i32 = (0..1000)
    .into_par_iter()
    .map(|x| x * 2)
    .sum();

// Use tokio for async I/O
// Already integrated in OpenCrust
```

#### Caching
```rust
// Use once_cell or lazy_static for expensive computations
use once_cell::sync::Lazy;

static EXPENSIVE_COMPUTATION: Lazy<HashMap<u32, String>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // Expensive setup...
    m
});
```

### Benchmarking with Criterion

```rust
use criterion::{black_box, criterionGroup, criterion_main, Criterion};

fn benchmark_my_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| my_function(black_box(42)))
    });
}

criterion_group!(benches, benchmark_my_function);
criterion_main!(benches);
```

### Profile-Guided Optimization

```bash
# 1. Build with PGO instrumentation
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" cargo build --release

# 2. Run representative workload
./target/release/myapp --benchmark

# 3. Build with PGO data
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data" cargo build --release
```

### Examples

#### Example 1: Analyze Function Performance
Input: "Why is this function slow?"
Output:
1. Check for unnecessary allocations (String::new, to_string)
2. Look for O(n²) algorithms
3. Check iterator chain efficiency
4. Verify no blocking operations in hot path
5. Suggest specific fixes with benchmarks

#### Example 2: Reduce Binary Size
Input: "Make my binary smaller"
Output:
1. Run `cargo bloat --release` to identify large dependencies
2. Use `cargo tree` to find transitive deps
3. Enable LTO: `lto = true` in [profile.release]
4. Strip symbols: `strip = true`
5. Use `opt-level = "z"` for size optimization

#### Example 3: Memory Leak Detection
Input: "Check for memory leaks"
Output:
1. Run with `valgrind --tool=memcheck ./target/debug/app`
2. Use `jemallocator` with `jemalloc-ctl` for heap profiling
3. Check for:
   - Forgotten `Drop` implementations
   - Reference cycles with `Rc/Arc`
   - Unbounded caches without eviction
4. Suggest fixes with proper resource management

## Key Principles

1. **Measure First** — Always profile before optimizing
2. **Focus on Hot Paths** — Optimize where time is actually spent
3. **Consider Readability** — Don't sacrifice clarity for 1% improvement
4. **Benchmark** — Use `criterion` for before/after comparison
5. **Document Trade-offs** — Explain why optimization was chosen

## Quick Checklist

When reviewing code for performance:
- [ ] No unnecessary allocations in hot paths
- [ ] Iterator chains are efficient (single-pass when possible)
- [ ] Stack allocated when size is known and small
- [ ] Proper data structures for access patterns
- [ ] Async code doesn't block on CPU work
- [ ] Cached expensive computations
- [ ] Benchmarks exist for critical paths
