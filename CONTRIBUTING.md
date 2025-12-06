# Contributing to UnsubMail

Thank you for your interest in contributing to UnsubMail! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Project Structure](#project-structure)
- [Code Style Guidelines](#code-style-guidelines)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Pull Request Process](#pull-request-process)
- [Issue Guidelines](#issue-guidelines)

## Code of Conduct

This project adheres to a code of conduct that we expect all contributors to follow:

- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on constructive feedback
- Respect differing viewpoints and experiences
- Accept responsibility and apologize for mistakes

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/unsubmail.git
   cd unsubmail
   ```
3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/unsubmail/unsubmail.git
   ```
4. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Environment

### Prerequisites

- **Rust 1.70+**: Install via [rustup](https://rustup.rs/)
- **Git**: For version control
- **Gmail account**: For testing (use a test account, not your primary)
- **Google Cloud Project**: With Gmail API enabled and OAuth2 credentials

### Setting Up

1. **Install Rust toolchain**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup update
   ```

2. **Install development tools**:
   ```bash
   rustup component add rustfmt clippy
   ```

3. **Set up environment variables**:
   Create a `.env.local` file in the project root:
   ```bash
   GOOGLE_CLIENT_ID=your_test_client_id
   GOOGLE_CLIENT_SECRET=your_test_client_secret
   ```

4. **Build the project**:
   ```bash
   cargo build
   ```

5. **Run tests**:
   ```bash
   cargo test
   ```

### Development Commands

```bash
# Build the project
cargo build

# Build optimized release
cargo build --release

# Run the application
cargo run

# Run with debug logging
RUST_LOG=unsubmail=debug cargo run

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Format code (REQUIRED before committing)
cargo fmt

# Check formatting without making changes
cargo fmt -- --check

# Run clippy linter (REQUIRED before committing)
cargo clippy

# Run clippy with all features
cargo clippy --all-targets --all-features -- -D warnings

# Check code without building
cargo check

# Generate documentation
cargo doc --open
```

## Project Structure

UnsubMail follows clean architecture principles. Please maintain this separation:

```
src/
├── cli/                    # CLI interface layer
│   ├── interactive.rs      # Interactive mode (TUI)
│   └── mod.rs             # Module exports
│
├── domain/                 # Business logic (pure, no dependencies)
│   ├── models.rs          # Core data structures
│   ├── analysis.rs        # Newsletter detection heuristics
│   └── planner.rs         # Action planning
│
├── infrastructure/         # External services & adapters
│   ├── imap/              # IMAP connection and operations
│   ├── storage/           # Keyring and JSON storage
│   └── network/           # HTTP client for unsubscribe
│
├── application/            # Application orchestration
│   └── workflow.rs        # Main workflows
│
└── main.rs                # CLI entry point
```

### Architecture Guidelines

1. **Domain Layer**:
   - No external dependencies (only std and core types)
   - Pure business logic
   - Test all logic thoroughly

2. **Infrastructure Layer**:
   - Adapter pattern for external services
   - Handle all I/O errors gracefully
   - Use async/await for I/O operations

3. **Application Layer**:
   - Orchestrate domain logic with infrastructure
   - Handle error propagation
   - Log important operations

4. **CLI Layer**:
   - Thin wrapper around application layer
   - Handle user interaction
   - Format output for users

## Code Style Guidelines

### Rust Style

- **Use `rustfmt`**: All code must be formatted with `cargo fmt`
- **Use `clippy`**: Fix all clippy warnings with `cargo clippy`
- **Follow Rust naming conventions**:
  - `snake_case` for functions, variables, modules
  - `PascalCase` for types, traits, enums
  - `SCREAMING_SNAKE_CASE` for constants

### Documentation

- **Public APIs**: Document all public functions, structs, and modules
- **Doc comments**: Use `///` for item documentation, `//!` for module documentation
- **Examples**: Include examples in doc comments where helpful
- **Format**:
  ```rust
  /// Brief one-line description
  ///
  /// More detailed description if needed. Explain the purpose,
  /// parameters, return values, and any important behavior.
  ///
  /// # Arguments
  ///
  /// * `param` - Description of parameter
  ///
  /// # Returns
  ///
  /// Description of return value
  ///
  /// # Errors
  ///
  /// When this function returns an error and why
  ///
  /// # Examples
  ///
  /// ```
  /// use unsubmail::domain::analysis::calculate_heuristic_score;
  ///
  /// let score = calculate_heuristic_score("newsletter@example.com", true, 50);
  /// assert!(score > 0.5);
  /// ```
  pub fn my_function(param: Type) -> Result<ReturnType> {
      // ...
  }
  ```

### Error Handling

- Use `anyhow::Result` for application code
- Use `thiserror` for domain errors
- Provide context with `.context()` or `.with_context()`
- Log errors before returning them

### Async/Await

- Use `tokio` runtime for async operations
- Mark all I/O functions as `async`
- Use `tokio::spawn` for concurrent operations
- Handle timeouts for network operations

### Testing

- Write unit tests for domain logic
- Write integration tests for workflows
- Use descriptive test names: `test_<what>_<when>_<expected>`
- Test error cases, not just happy paths

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in specific module
cargo test domain::analysis

# Run tests with logging
RUST_LOG=debug cargo test
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_score_with_unsubscribe_header() {
        let score = calculate_heuristic_score("newsletter@example.com", true, 35);
        assert!(score > 1.0, "Expected score > 1.0, got {}", score);
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

Place integration tests in `tests/` directory:

```rust
// tests/integration_test.rs
use unsubmail::application::workflow;

#[tokio::test]
async fn test_full_workflow() {
    // Test complete workflows
}
```

## Submitting Changes

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Build process or auxiliary tool changes
- `ci`: CI/CD changes

**Examples**:
```
feat(analysis): add support for RFC 8058 one-click unsubscribe

Implement detection and execution of one-click unsubscribe
according to RFC 8058 List-Unsubscribe-Post header.

Closes #42
```

```
fix(imap): handle connection timeout gracefully

Add 30-second timeout for IMAP connections and provide
clear error message when timeout occurs.
```

### Before Submitting

1. **Update from upstream**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run all checks**:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test
   cargo build --release
   ```

3. **Update documentation** if needed

4. **Add tests** for new functionality

## Pull Request Process

1. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Open a Pull Request** on GitHub

3. **Fill out the PR template** completely:
   - Description of changes
   - Motivation and context
   - Type of change (bug fix, feature, etc.)
   - Testing done
   - Screenshots (if applicable)

4. **Link related issues** using keywords:
   - `Fixes #123` - Closes issue when PR is merged
   - `Relates to #456` - References related issue

5. **Respond to review feedback**:
   - Address all comments
   - Push additional commits if needed
   - Request re-review when ready

6. **Wait for approval**:
   - At least one maintainer approval required
   - All CI checks must pass
   - No merge conflicts

7. **Squash and merge** (maintainer will handle this)

## Issue Guidelines

### Reporting Bugs

Use the bug report template and include:

- **Description**: Clear description of the bug
- **Steps to Reproduce**: Exact steps to trigger the bug
- **Expected Behavior**: What should happen
- **Actual Behavior**: What actually happens
- **Environment**:
  - OS and version
  - Rust version
  - UnsubMail version
- **Logs**: Relevant log output (use RUST_LOG=debug)
- **Screenshots**: If applicable

### Requesting Features

Use the feature request template and include:

- **Problem**: What problem does this solve?
- **Proposed Solution**: How should it work?
- **Alternatives**: Other solutions considered
- **Additional Context**: Any other relevant information

### Asking Questions

- Check existing issues and discussions first
- Use [GitHub Discussions](https://github.com/unsubmail/unsubmail/discussions) for questions
- Provide context and what you've tried

## Development Tips

### Debugging

```bash
# Run with full debug logging
RUST_LOG=unsubmail=trace cargo run

# Run with specific module logging
RUST_LOG=unsubmail::infrastructure::imap=debug cargo run

# Use rust-gdb or rust-lldb for debugging
rust-gdb target/debug/unsubmail
```

### Performance

- Profile with `cargo flamegraph`
- Use `cargo bench` for benchmarks
- Monitor memory usage with `valgrind` or `heaptrack`

### Documentation

```bash
# Generate and open documentation
cargo doc --open

# Check for broken links
cargo deadlinks
```

## Getting Help

- **Questions**: [GitHub Discussions](https://github.com/unsubmail/unsubmail/discussions)
- **Bugs**: [GitHub Issues](https://github.com/unsubmail/unsubmail/issues)
- **Chat**: (Add Discord/Slack if available)

## License

By contributing to UnsubMail, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to UnsubMail!
