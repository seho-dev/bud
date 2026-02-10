---
name: rust-best-practices
description: Enforces complete Rust code quality standards including optimization, error handling, documentation, testing, concurrency, security, and version control. Use when writing, reviewing, or optimizing Rust code, or automatically triggered when users request Rust code generation, refactoring, performance optimization, or concurrent programming.
allowed-tools: Read, Edit, Write, Grep, Glob, Bash
---

# Rust Best Practices Enforcer

This skill fully implements all Rust code quality rules defined in the project's CLAUDE.md.

## Core Principles

**All code MUST be fully optimized**, including:

- Maximizing algorithmic big-O efficiency for memory and runtime
- Using parallelization and SIMD where appropriate
- Following proper style conventions for Rust (e.g., maximizing code reuse, DRY principle)
- No extra code beyond what is absolutely necessary to solve the problem (i.e., no technical debt)

⚠️ **Warning**: Code that is not fully optimized will be rejected. You have permission to do another pass if you believe the code is not fully optimized.

## Quick Start

### Basic Workflow

1. **Read code**: Use Read to inspect existing code
2. **Apply rules**: Follow all rules in the sections below
3. **Modify code**: Use Edit/Write to apply improvements
4. **Verify**: Run all pre-commit checks

---

## Preferred Tools

### Project Management
- Use `cargo` for project management, building, and dependency management

### Common Libraries

```toml
[dependencies]
# Progress bars for long-running operations
indicatif = "0.17"

# JSON serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# TUI applications
ratatui = "0.26"
crossterm = "0.27"

# Web servers and HTTP APIs
axum = "0.7"
tower = "0.4"  # middleware

# Error handling
thiserror = "1.0"  # library code
anyhow = "1.0"     # application code

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# CPU-bound parallelism
rayon = "1.10"

# Logging
tracing = "0.1"
log = "0.4"

# Environment variables
dotenvy = "0.15"

# Sensitive data
secrecy = "0.8"
```

### Axum Best Practices

When writing web APIs:
- Keep request handlers async, returning `Result<Response, AppError>` to centralize error handling
- Use layered extractors and shared state structs instead of global mutable data
- Add `tower` middleware (timeouts, tracing, compression) for observability and resilience
- Offload CPU-bound work to `tokio::task::spawn_blocking` or background services to avoid blocking the reactor

### Error Reporting

- When reporting errors to the console, use `tracing::error!` or `log::error!`
- **DO NOT** use `println!`

---

## Code Style and Formatting

### Must Follow

- ✅ **MUST** use meaningful, descriptive variable and function names
- ✅ **MUST** follow Rust API Guidelines and idiomatic Rust conventions
- ✅ **MUST** use 4 spaces for indentation (never tabs)
- ❌ **NEVER** use emoji or unicode that emulates emoji (e.g., ✓, ✗)
  - Only exception: when writing tests and testing the impact of multibyte characters

### Naming Conventions

- Functions/variables/modules: `snake_case`
- Types/traits: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`

### Other Rules

- Line length limit: 100 characters (rustfmt default)
- **Special note**: Assume the user is a Python expert but a Rust novice. Include additional code comments around Rust-specific nuances that a Python developer may not recognize.

---

## Documentation

### Must Follow

- ✅ **MUST** include doc comments for all public functions, structs, enums, and methods
- ✅ **MUST** document function parameters, return values, and errors
- Keep comments up-to-date with code changes
- Include examples in doc comments for complex functions

### Standard Doc Comment Template

```rust
/// Brief description of functionality.
///
/// # Arguments
///
/// * `param1` - Description of parameter 1
/// * `param2` - Description of parameter 2
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// * `ErrorType::Variant1` - Error condition 1
/// * `ErrorType::Variant2` - Error condition 2
///
/// # Examples
///
/// ```
/// let result = function_name(arg1, arg2)?;
/// assert_eq!(result, expected);
/// ```
pub fn function_name(param1: Type1, param2: Type2) -> Result<ReturnType, ErrorType> {
    // Implementation
}
```

---

## Type System

### Must Follow

- ✅ **MUST** leverage Rust's type system to prevent bugs at compile time
- ❌ **NEVER** use `.unwrap()` in library code
- ✅ **ONLY** use `.expect()` for invariant violations with a descriptive message
- ✅ **MUST** use meaningful custom error types with `thiserror`

### Best Practices

- Use newtypes to distinguish semantically different values of the same underlying type
- Prefer `Option<T>` over sentinel values

---

## Error Handling

### Must Follow

- ❌ **NEVER** use `.unwrap()` in production code paths
- ✅ **MUST** use `Result<T, E>` for fallible operations
- ✅ **MUST** use `thiserror` for defining error types and `anyhow` for application-level errors
- ✅ **MUST** propagate errors with `?` operator where appropriate
- Provide meaningful error messages with context using `.context()` from `anyhow`

### Example

```rust
use thiserror::Error;
use anyhow::{Context, Result};

#[derive(Debug, Error)]
pub enum MyError {
    #[error("file not found: {0}")]
    FileNotFound(String),
    #[error("parse failed")]
    ParseError,
}

pub fn process_file(path: &str) -> Result<Data> {
    let content = std::fs::read_to_string(path)
        .context(format!("failed to read file: {}", path))?;

    parse_data(&content)
        .context("failed to parse file content")
}
```

---

## Function Design

### Must Follow

- ✅ **MUST** keep functions focused on a single responsibility
- ✅ **MUST** prefer borrowing (`&T`, `&mut T`) over ownership when possible
- Limit function parameters to 5 or fewer; use a config struct for more
- Return early to reduce nesting
- Use iterators and combinators over explicit loops where clearer

### Examples

```rust
// ❌ Bad: takes ownership
fn process(data: Vec<String>) -> String { ... }

// ✅ Good: borrows
fn process(data: &[String]) -> String { ... }

// ❌ Bad: too many parameters
fn create_user(name: String, age: u32, email: String, phone: String,
               address: String, city: String) -> User { ... }

// ✅ Good: use config struct
fn create_user(config: UserConfig) -> User { ... }

// ✅ Good: early return
fn validate(input: &str) -> Result<(), Error> {
    if input.is_empty() {
        return Err(Error::Empty);
    }
    // Continue validation...
    Ok(())
}
```

---

## Struct and Enum Design

### Must Follow

- ✅ **MUST** keep types focused on a single responsibility
- ✅ **MUST** derive common traits: `Debug`, `Clone`, `PartialEq` where appropriate
- Use `#[derive(Default)]` when a sensible default exists
- Prefer composition over inheritance-like patterns
- Use builder pattern for complex struct construction
- Make fields private by default; provide accessor methods when needed

### Example

```rust
// ✅ Good design
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Config {
    timeout: u64,
    retry_count: u32,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    pub fn timeout(&self) -> u64 {
        self.timeout
    }
}

pub struct ConfigBuilder {
    timeout: Option<u64>,
    retry_count: Option<u32>,
}

impl ConfigBuilder {
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> Config {
        Config {
            timeout: self.timeout.unwrap_or(30),
            retry_count: self.retry_count.unwrap_or(3),
        }
    }
}
```

---

## Testing

### Must Follow

- ✅ **MUST** write unit tests for all new functions and types
- ✅ **MUST** mock external dependencies (APIs, databases, file systems)
- ✅ **MUST** use the built-in `#[test]` attribute and `cargo test`
- Follow the Arrange-Act-Assert pattern
- Do not commit commented-out tests
- Use `#[cfg(test)]` modules for test code

### Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_total() {
        // Arrange
        let items = vec![10.0, 20.0];
        let tax_rate = 0.08;

        // Act
        let result = calculate_total(&items, tax_rate);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 32.40);
    }
}
```

---

## Imports and Dependencies

### Must Follow

- ❌ **MUST** avoid wildcard imports (`use module::*`)
  - **Exceptions**: preludes, test modules (`use super::*`), and prelude re-exports
- ✅ **MUST** document dependencies in `Cargo.toml` with version constraints
- Use `cargo` for dependency management
- Organize imports: standard library → external crates → local modules
- Use `rustfmt` to automate import formatting

### Example

```rust
// ✅ Correct import order
use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::config::Config;
use crate::error::MyError;
```

---

## Rust Best Practices

### Must Follow

- ❌ **NEVER** use `unsafe` unless absolutely necessary; document safety invariants when used
- ✅ **MUST** call `.clone()` explicitly on non-`Copy` types; avoid hidden clones in closures and iterators
- ✅ **MUST** use pattern matching exhaustively; avoid catch-all `_` patterns when possible
- ✅ **MUST** use `format!` macro for string formatting
- Use iterators and iterator adapters over manual loops
- Use `enumerate()` instead of manual counter variables
- Prefer `if let` and `while let` for single-pattern matching

### Examples

```rust
// ✅ Good: explicit .clone()
let data = original_data.clone();

// ✅ Good: exhaustive matching
match status {
    Status::Active => handle_active(),
    Status::Inactive => handle_inactive(),
    Status::Pending => handle_pending(),
    // Don't use _ => {} unless truly appropriate
}

// ✅ Good: use enumerate
for (index, item) in items.iter().enumerate() {
    println!("{}: {}", index, item);
}

// ✅ Good: if let
if let Some(value) = optional_value {
    process(value);
}
```

---

## Memory and Performance

### Must Follow

- ✅ **MUST** avoid unnecessary allocations; prefer `&str` over `String` when possible
- ✅ **MUST** use `Cow<'_, str>` when ownership is conditionally needed
- Use `Vec::with_capacity()` when the size is known
- Prefer stack allocation over heap when appropriate
- Use `Arc` and `Rc` judiciously; prefer borrowing

### Examples

```rust
use std::borrow::Cow;

// ✅ Good: use Cow to avoid unnecessary allocations
fn process(input: &str) -> Cow<'_, str> {
    if input.contains("special") {
        Cow::Owned(input.replace("special", "SPECIAL"))
    } else {
        Cow::Borrowed(input)
    }
}

// ✅ Good: pre-allocate capacity
let mut items = Vec::with_capacity(1000);
for i in 0..1000 {
    items.push(i);
}
```

---

## Concurrency

### Must Follow

- ✅ **MUST** use `Send` and `Sync` bounds appropriately
- ✅ **MUST** prefer `tokio` for async runtime in async applications
- ✅ **MUST** use `rayon` for CPU-bound parallelism
- Avoid `Mutex` when `RwLock` or lock-free alternatives are appropriate
- Use channels (`mpsc`, `crossbeam`) for message passing

### Examples

```rust
use rayon::prelude::*;
use tokio::sync::mpsc;
use std::sync::Arc;

// CPU-bound: use rayon
let results: Vec<_> = data.par_iter()
    .map(|item| expensive_computation(item))
    .collect();

// Async: use tokio
#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move {
        tx.send("message").await.unwrap();
    });

    while let Some(msg) = rx.recv().await {
        println!("{}", msg);
    }
}

// Shared state: use Arc
let shared_data = Arc::new(MyData::new());
let data_clone = Arc::clone(&shared_data);
```

---

## Security

### Must Follow

- ❌ **NEVER** store secrets, API keys, or passwords in code. Only store them in `.env`
  - Ensure `.env` is declared in `.gitignore`
- ✅ **MUST** use environment variables for sensitive configuration via `dotenvy` or `std::env`
- ❌ **NEVER** log sensitive information (passwords, tokens, PII)
- Use `secrecy` crate for sensitive data types

### Example

```rust
use secrecy::{Secret, ExposeSecret};
use dotenvy::dotenv;

fn load_api_key() -> Secret<String> {
    dotenv().ok();
    let key = std::env::var("API_KEY")
        .expect("API_KEY must be set");
    Secret::new(key)
}

// Only expose when needed
fn make_request(api_key: &Secret<String>) {
    let key_str = api_key.expose_secret();
    // Use key_str...
}
```

---

## Version Control

### Must Follow

- ✅ **MUST** write clear, descriptive commit messages
- ❌ **NEVER** commit commented-out code; delete it
- ❌ **NEVER** commit debug `println!` statements or `dbg!` macros
- ❌ **NEVER** commit credentials or sensitive data

---

## Tools

### Must Follow

- ✅ **MUST** use `rustfmt` for code formatting
- ✅ **MUST** use `clippy` for linting and follow its suggestions
- ✅ **MUST** ensure code compiles with no warnings (use `-D warnings` flag in CI, not `#![deny(warnings)]` in source)
- Use `cargo` for building, testing, and dependency management
- Use `cargo test` for running tests
- Use `cargo doc` for generating documentation
- ❌ **NEVER** build with `cargo build --features python`: this will always fail
- ✅ **ALWAYS** use `maturin` for Python bindings

---

## Before Committing

Before committing code, **MUST** confirm:

```bash
# [ ] All tests pass
cargo test

# [ ] No compiler warnings
cargo build

# [ ] Clippy passes
cargo clippy -- -D warnings

# [ ] Code is formatted
cargo fmt --check

# [ ] If the project creates a Python package and Rust code is touched,
#     rebuild the Python package
source .venv/bin/activate && maturin develop --release --features python

# [ ] If the project creates a WASM package and Rust code is touched,
#     rebuild the WASM package
wasm-pack build --target web --out-dir web/pkg

# [ ] All public items have doc comments
# [ ] No commented-out code or debug statements
# [ ] No hardcoded credentials
```

---

## Usage Examples

### Scenario 1: Creating a New Function

When user says: "Write a function to read a config file"

**Execution Steps**:
1. Design function signature using `Result<T, E>`
2. Define error types with `thiserror`
3. Add complete doc comments (Arguments/Returns/Errors/Examples)
4. Implement with borrowing preference, use `?` to propagate errors
5. Write unit tests (including success and failure cases)
6. Run `cargo test` to verify

### Scenario 2: Code Review

When user says: "Review this code"

**Checklist**:
1. Are there doc comments?
2. Is `.unwrap()` used?
3. Are errors handled correctly?
4. Is borrowing used instead of ownership?
5. Are there unnecessary allocations?
6. Are there tests?
7. Do names follow conventions?
8. Is there commented-out code or debug statements?

### Scenario 3: Performance Optimization

When user says: "This code is too slow"

**Optimization Strategies**:
1. Use `Vec::with_capacity()` for pre-allocation
2. Avoid unnecessary `.clone()`
3. Use iterator chains instead of multiple loops
4. CPU-bound: consider `rayon` parallelism
5. I/O-bound: consider `tokio` async
6. Use `Cow<'_, str>` to avoid conditional allocations

### Scenario 4: Writing Concurrent Code

When user says: "Need to process this data in parallel"

**Implementation Steps**:
1. Determine: CPU-bound (rayon) or I/O-bound (tokio)?
2. Use `Send` and `Sync` bounds on types
3. Consider using channels instead of shared state
4. Wrap shared data with `Arc` when necessary
5. Prefer `RwLock` over `Mutex` (for read-heavy workloads)
6. Write concurrent tests

---

## FAQ

### Q: When to use `anyhow` vs `thiserror`?

**A**:
- **Library code**: Use `thiserror` to define specific error types
- **Application code**: Use `anyhow` for quick propagation and context

### Q: Rust concepts that confuse Python developers?

**A**: Add comments in code to explain:
- Lifetime parameters
- Borrow checker
- Move semantics
- Trait bounds

### Q: When can I use `unsafe`?

**A**: Only in these cases:
- FFI calls
- Low-level memory operations (with thorough testing)
- Performance-critical paths (with benchmark proof)
- **MUST** document all safety invariants

### Q: How to annotate Rust code for Python developers?

**A**: Example

```rust
// Note: Rust has two string types
// - &str: string slice (like Python's immutable str)
// - String: growable string (like Python list, but only for characters)
pub fn process(data: &str) -> String {
    // to_string() creates a heap-allocated copy
    // (like Python's str.copy(), but Rust requires it to be explicit)
    data.to_string()
}

// Rust's ownership system: variables can only have one owner
// This is similar to Python's reference counting, but enforced at compile time
let s1 = String::from("hello");
let s2 = s1;  // s1 is "moved" to s2, s1 is no longer usable
              // (in Python, both would point to the same object)
// println!("{}", s1);  // ❌ Compile error! s1 has been moved

// To achieve Python-style sharing, use borrowing (&)
let s1 = String::from("hello");
let s2 = &s1;  // s2 borrows s1 (like Python aliasing)
println!("{} {}", s1, s2);  // ✅ Both can be used
```

---

## Version History

- v1.0 (2026-02-10) Initial version, complete mapping of project CLAUDE.md

---

**Remember**: Prioritize clarity and maintainability over cleverness.
