# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**win_opt** is a Windows 11 optimization CLI tool written in Rust that provides system maintenance and optimization utilities. The tool is designed to run on Windows and includes operations that may require administrator privileges.

## Build & Run Commands

```bash
# Build the project
cargo build

# Build release version (optimized)
cargo build --release

# Run the project
cargo run -- <command>

# Run with specific subcommand
cargo run -- clean      # Clean temporary files
cargo run -- network    # Flush DNS and reset Winsock
cargo run -- repair     # Run SFC/DISM (requires admin)
cargo run -- info       # Show system information

# Run tests
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Architecture

This is a single-binary CLI application with all code in `src/main.rs`. The architecture is straightforward:

- **CLI Framework**: Uses `clap` with derive macros for command-line parsing
- **Command Pattern**: Each subcommand (`Clean`, `Network`, `Repair`, `Info`) maps to a dedicated function
- **Output Styling**: Uses `colored` crate for terminal color output
- **Platform-Specific**: Windows-only tool that executes Windows system commands via `std::process::Command`

### Key Components

1. **CLI Structure** (`Cli` and `Commands` enums):

   - Defines four subcommands using clap's derive macros
   - Each command is documented with help text in Spanish

2. **Core Functions**:

   - `clean_temp_files()`: Deletes files/directories from the system temp directory
   - `flush_dns()`: Executes `ipconfig /flushdns` and `netsh winsock reset`
   - `run_system_repair()`: Runs DISM and SFC for system file verification (admin only)
   - `show_system_info()`: Displays basic system information
   - `is_admin()`: Helper that checks admin privileges by attempting `net session`

3. **Error Handling**:
   - Uses `Result` pattern with `.is_ok()` checks
   - Gracefully handles locked/in-use files during cleanup
   - Provides user-friendly error messages for permission issues

## Important Notes

### Platform Constraints

- **Windows-only**: All system commands (`cmd`, `ipconfig`, `netsh`, `DISM`, `sfc`) are Windows-specific
- Some operations require administrator privileges
- Uses `cmd /C` to execute Windows commands

### Admin Privilege Detection

The `is_admin()` function runs `net session` to detect admin rights. This is a simple heuristic that works on Windows but may have edge cases.

### File Operations

The `clean_temp_files()` function uses `std::env::temp_dir()` and attempts to delete all contents. File/directory deletion errors are silently ignored (as files may be locked by running processes).

### Dependencies

- **clap**: CLI parsing with derive feature
- **colored**: Terminal color output
- **directories**: Cross-platform directory detection (though this tool is Windows-only)

## Code Style

- All user-facing strings are in Spanish
- Uses Rust 2024 edition
- Functions are well-documented with comments explaining their purpose
- Color-coded output: yellow for operations in progress, green for success, red for errors

### Format

Code must strictly adhere to the standard formatting. We do not debate brace styles or indentation.

- **Rule:** All code must pass `cargo fmt` before being committed.
- **CI Check:** `cargo fmt --check`

### Linting (`clippy`)

We use Clippy to detect common mistakes and non-idiomatic code.

- **Rule:** No warnings are allowed in the final codebase.
- **Command:** `cargo clippy -- -D warnings`
- **Exceptions:** If a lint must be ignored, use `#[allow(clippy::lint_name)]` and include a comment explaining why.

## Naming Conventions

We follow the standard **RFC 430** naming conventions.

| Item                       | Style                  | Example                             |
| :------------------------- | :--------------------- | :---------------------------------- |
| **Variables / Functions**  | `snake_case`           | `let user_id; fn calculate_total()` |
| **Types (Structs, Enums)** | `UpperCamelCase`       | `struct UserProfile;`               |
| **Traits**                 | `UpperCamelCase`       | `trait Printable { ... }`           |
| **Constants / Statics**    | `SCREAMING_SNAKE_CASE` | `const MAX_RETRIES: u32 = 5;`       |
| **Lifetimes**              | `snake_case` (short)   | `'a`, `'ctx`                        |
| **Files / Modules**        | `snake_case`           | `user_manager.rs`                   |

## Safety and Error Handling

### Prohibit `unwrap()` in Production

Using `.unwrap()` triggers unrecoverable panics.

- **Bad Practice:** `let config = File::open("config.toml").unwrap();`
- **Good Practice:** \* Use `.expect("context")` if a panic is intentional and provide clear reasoning.
  - Use the `?` operator to propagate errors.
  - Use `match` or `if let` for graceful handling.

### Idiomatic Error Handling

- **Libraries:** Use the `thiserror` crate to define custom error Enums.
- **Applications:** Use the `anyhow` crate for high-level error handling and easy context attachment.

### `unsafe` Blocks

The `unsafe` keyword should be a last resort.

- **Rule:** Every `unsafe` block must be preceded by a `// SAFETY:` comment explaining why the operation is safe and how it upholds Rust's invariants.

## Idiomatic Design

### Newtype Pattern

Avoid "primitive obsession." Use the type system to enforce logic.

```rust
// Bad
fn process(age: i32, id: i32)

// Good
struct Age(i32);
struct UserId(i32);
fn process(age: Age, id: UserId)
```

### Iterators over Loops

Prefer functional iterators (`map`, `filter`, `fold`) over imperative for loops. They are often more expressive and benefit from "zero-cost abstraction" optimizations.

### Standard Trait Implementation

Don't reinvent method names if a standard trait exists:

- Implement `Default` instead of a parameterless `new()` function.

- Implement `Display` instead of `to_string_custom()`.

- Implement `From/Into` for type conversions.

### Minimize .clone()

Excessive cloning to satisfy the borrow checker usually signals a design flaw. Prefer passing references (`&T`) unless ownership is strictly required.

## Organization and Visibility

- **Module Structure**: Keep `main.rs` small (setup and CLI parsing only). Business logic should reside in `lib.rs` or dedicated modules.

- **Visibility:**
  - `pub`: Only for the actual public API.
  - `pub(crate)`: For items shared within the project but hidden from users.
  - Private: The default for everything else.

## Testing and Documentation

### Unit Tests

Unit tests should live in the same file as the code, within a `tests` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logic() { ... }
}
```

### Documentation

- All `pub` functions must have documentation comments using `///`.
- **Doctests**: Include code examples in your documentation. Rust will run these as tests to ensure the documentation remains accurate.

````rust
/// Adds two numbers together.
///
/// # Examples
/// ```
/// let result = my_crate::add(1, 1);
/// assert_eq!(result, 2);
/// ```
pub fn add(a: i32, b: i32) -> i32 { a + b }
````
