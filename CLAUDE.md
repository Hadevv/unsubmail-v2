# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

UnsubMail v2 is a modern, fast, and reliable Rust CLI tool designed to clean Gmail inboxes from newsletters and spam. The project follows clean architecture principles with clear separation of concerns.

## Key Design Principles

1. **Clean Architecture** - Separation between domain logic, application workflows, and infrastructure
2. **Type Safety** - Leveraging Rust's type system for reliability
3. **Performance** - Headers-only scanning, batch operations, async/await
4. **Security** - OAuth2 only, secure token storage via system keyring
5. **User Experience** - Minimal prompts, interactive TUI, clear feedback

## Project Structure

```
src/
├── cli/                    # CLI interface layer
│   ├── mod.rs             # Module exports
│   ├── accounts.rs        # Account management commands
│   ├── scan.rs            # Inbox scanning with progress bars
│   ├── select.rs          # Interactive sender selection (dialoguer)
│   └── actions.rs         # Cleanup action execution
│
├── domain/                 # Business logic (pure, no dependencies)
│   ├── mod.rs             # Module exports
│   ├── models.rs          # Core data structures (EmailAccount, SenderInfo, etc.)
│   ├── analysis.rs        # Email analysis & newsletter detection heuristics
│   └── planner.rs         # Action planning (unsubscribe vs block strategy)
│
├── infrastructure/         # External services & adapters
│   ├── mod.rs
│   ├── google/
│   │   ├── mod.rs
│   │   ├── auth.rs        # OAuth2 flow using yup-oauth2
│   │   ├── gmail_api.rs   # Gmail API client (list, get headers)
│   │   ├── filters.rs     # Gmail filter creation for blocking
│   │   └── delete.rs      # Batch message deletion
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── keyring.rs     # Secure token storage (keyring crate)
│   │   └── json_store.rs  # Account metadata storage (JSON files)
│   └── network/
│       ├── mod.rs
│       └── http_client.rs # One-click unsubscribe HTTP POST
│
├── application/            # Application orchestration
│   ├── mod.rs
│   └── workflow.rs        # Main workflows (add, scan, clean)
│
└── main.rs                # CLI entry point (clap commands)
```

## Architecture Layers

### Domain Layer (src/domain/)
- **No external dependencies** - Pure business logic
- **models.rs** - Core data structures
- **analysis.rs** - Newsletter detection using headers
  - List-Unsubscribe header detection
  - One-click unsubscribe detection
  - Email pattern matching (newsletter@, noreply@, etc.)
  - Heuristic scoring
- **planner.rs** - Strategy selection for cleanup actions

### Application Layer (src/application/)
- **workflow.rs** - Orchestrates domain logic with infrastructure
- Coordinates all operations end-to-end
- Error handling and user feedback

### Infrastructure Layer (src/infrastructure/)
- **Google API integration** - OAuth2 + Gmail API via google-gmail1
- **Storage** - keyring for tokens, JSON for metadata
- **Network** - reqwest for one-click unsubscribe

### CLI Layer (src/cli/)
- **User interaction** - dialoguer, indicatif for progress
- **Command handlers** - Thin wrappers around workflow

## Development Commands

```bash
# Build the project
cargo build

# Run with logging
RUST_LOG=unsubmail=debug cargo run -- clean email@gmail.com

# Run tests
cargo test

# Check without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Build release
cargo build --release
```

## Key Dependencies

- `clap` - CLI argument parsing
- `tokio` - Async runtime
- `google-gmail1` - Gmail API client
- `yup-oauth2` - OAuth2 authentication
- `dialoguer` - Interactive prompts
- `indicatif` - Progress bars
- `keyring` - Secure credential storage
- `reqwest` - HTTP client
- `hyper` + `hyper-rustls` - HTTP/TLS for Google APIs
- `serde` + `serde_json` - Serialization
- `anyhow` + `thiserror` - Error handling
- `tracing` - Structured logging

## Important Notes

### OAuth2 Flow
- Client ID and Secret must be set via environment variables
- Tokens cached in `~/.config/unsubmail/tokens/`
- Account metadata in `~/.config/unsubmail/accounts/`

### Gmail API Usage
- Headers-only fetching for speed
- Batch operations where possible
- Respects API rate limits (implicit via SDK)
- Uses `format="metadata"` for message fetching

### Newsletter Detection Heuristics
- **List-Unsubscribe header** - 0.4 score boost
- **List-Unsubscribe-Post: One-Click** - Enables one-click unsubscribe
- **Email keywords** - newsletter, noreply, no-reply, notification, promo, marketing
- **Message count** - >5 messages = 0.3, >20 = additional 0.5
- **Subject keywords** - "unsubscribe", "newsletter" = 0.1

### Cleanup Strategy
```
For each selected sender:
  if has_one_click_unsubscribe:
    POST to unsubscribe URL (HTTPS only)
    delete all messages
  else:
    create Gmail filter (auto-trash)
    delete all messages
```

### Security
- Only HTTPS URLs allowed for unsubscribe
- `mailto:` links explicitly rejected
- Tokens stored in OS keyring (Windows Credential Manager, macOS Keychain, Linux Secret Service)
- No passwords or IMAP - OAuth2 only

## TODO for Full Implementation

The project skeleton is complete and compiles. Key TODOs:

1. **OAuth2 flow completion** - Implement full authentication in `auth.rs`
2. **Parallel message fetching** - Use tokio tasks for batch header fetching
3. **RFC 2822 date parsing** - Add proper date header parsing
4. **Account removal** - Implement keyring token deletion
5. **Error recovery** - Better handling of API failures
6. **Rate limiting** - Add exponential backoff for API calls
7. **Dry-run mode** - Preview actions without executing
8. **Export results** - JSON/CSV output for scan results

## Testing Strategy

- Unit tests for domain logic (heuristics, scoring)
- Integration tests with mock Gmail API
- Manual testing with real Gmail accounts (test accounts)

## Commit Conventions

Use conventional commits:
- `feat:` - New features
- `fix:` - Bug fixes
- `refactor:` - Code refactoring
- `docs:` - Documentation changes
- `test:` - Test additions/changes
- `chore:` - Build/tooling changes

## Performance Considerations

- **Headers-only** - Never fetch full message bodies
- **Batch operations** - Use Gmail batch delete (max 1000 messages)
- **Pagination** - Handle large inboxes (2000+ messages)
- **Progress feedback** - Show progress bars for long operations
- **Async** - All I/O is async (tokio)

## Known Limitations

- Gmail only (no other providers yet)
- OAuth2 credentials must be manually created
- No undo functionality (deletions are permanent)
- One-click unsubscribe depends on sender support
- Windows requires Visual Studio Build Tools for ring crypto

## Contributing Guidelines

When adding features:
1. Follow the existing architecture
2. Keep domain logic pure
3. Add tests for business logic
4. Update this document if adding new patterns
5. Use `cargo fmt` and `cargo clippy`
6. Add doc comments for public APIs
