# Changelog

All notable changes to UnsubMail will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Interactive loop mode: Continue cleaning from same account or switch accounts
- Comprehensive README.md with installation, configuration, and usage guide
- CONTRIBUTING.md with development guidelines and coding standards
- GitHub CI/CD workflow with multi-platform testing (Linux, macOS, Windows)
- GitHub issue templates (bug report, feature request)
- GitHub pull request template
- Comprehensive module-level documentation for all layers
- Integration tests for domain logic
- Example programs: `simple_scan.rs` and `batch_cleanup.rs`
- Improved .gitignore with coverage and build artifacts

### Changed
- Enhanced interactive mode with account switching capability
- Updated documentation structure across all modules
- Improved error messages and user feedback

## [0.1.0] - 2024-12-02 - Initial Implementation

### Added
- Clean architecture implementation with 4 layers (domain, application, infrastructure, CLI)
- OAuth2 authentication flow with automatic browser opening
- IMAP-based Gmail inbox scanning
- Newsletter detection using multiple heuristics:
  - RFC 2369 List-Unsubscribe header detection
  - RFC 8058 One-click unsubscribe support
  - Email pattern matching (newsletter@, noreply@, etc.)
  - Message volume analysis
  - Heuristic scoring system
- Interactive terminal UI with progress indicators
- Batch message operations (scan, delete, move to spam)
- Secure token storage in OS keyring
- One-click HTTP POST unsubscribe
- Parallel message fetching with semaphore-based rate limiting

### Features
- **Smart Detection**: Advanced heuristics for newsletter identification
- **One-Click Unsubscribe**: RFC 8058 automated unsubscribe
- **OAuth2 Security**: No password storage, uses OS keyring
- **Fast Scanning**: Headers-only, parallel fetching
- **Interactive TUI**: Beautiful terminal interface with `inquire` and `indicatif`
- **Batch Operations**: Efficiently process hundreds of emails

### Technical Implementation
- Async/await with Tokio runtime
- IMAP via `async-imap` crate
- OAuth2 via `oauth2` crate
- Clean architecture with dependency inversion
- Type-safe domain models
- Comprehensive error handling with `anyhow` and `thiserror`

### Performance
- Parallel message fetching (up to 10 concurrent requests)
- Exponential backoff for rate limiting
- Headers-only scanning (no message bodies downloaded)
- Expected scan time:
  - 500 messages: ~5-10 seconds
  - 2000 messages: ~20-40 seconds

### Security
- OAuth2 authentication only (no passwords)
- Tokens stored securely in OS keyring:
  - macOS: Keychain
  - Windows: Credential Manager
  - Linux: Secret Service
- HTTPS-only unsubscribe links
- mailto: links explicitly rejected

### Dependencies
- `tokio` - Async runtime
- `async-imap` - Gmail IMAP client
- `oauth2` - OAuth2 authentication
- `inquire` - Interactive prompts
- `indicatif` - Progress bars
- `serde` + `serde_json` - Serialization
- `anyhow` + `thiserror` - Error handling
- `mailparse` - RFC 2822 date parsing
- `reqwest` - HTTP client for unsubscribe
- `regex` - Pattern matching
- `chrono` - Date/time handling

### Known Limitations
- Gmail only (no other providers)
- OAuth2 credentials must be manually created
- No undo functionality
- One-click unsubscribe depends on sender support
- Windows requires Visual Studio Build Tools for ring crypto

### Documentation
- CLAUDE.md with architecture and design principles
- Inline documentation for all public APIs
- Examples for testing and debugging

## [Pre-release] - 2024-12-01

### Initial Development
- Project structure setup
- Domain model design
- Infrastructure layer implementation
- Basic IMAP connection tests

---

## Versioning Strategy

- **Major version (X.0.0)**: Breaking changes, major architecture changes
- **Minor version (0.X.0)**: New features, backward-compatible changes
- **Patch version (0.0.X)**: Bug fixes, documentation updates

## Release Process

1. Update CHANGELOG.md with release notes
2. Update version in Cargo.toml
3. Run full test suite: `cargo test`
4. Build release: `cargo build --release`
5. Tag release: `git tag -a vX.Y.Z -m "Release X.Y.Z"`
6. Push tag: `git push origin vX.Y.Z`
7. Create GitHub release with binaries

## Future Roadmap

### v0.2.0 (Planned)
- [ ] Dry-run mode (preview without executing)
- [ ] Export results to JSON/CSV
- [ ] Improved error recovery and retry logic
- [ ] Configuration file support
- [ ] Account management improvements

### v0.3.0 (Planned)
- [ ] Support for other email providers (Outlook, Yahoo)
- [ ] Web UI for remote management
- [ ] Scheduled cleanup jobs
- [ ] Email template recognition
- [ ] Advanced filtering rules

### v1.0.0 (Future)
- [ ] Production-ready stability
- [ ] Comprehensive test coverage (>80%)
- [ ] Full documentation and guides
- [ ] Multi-language support
- [ ] Enterprise features

---

For the latest updates, see [GitHub Releases](https://github.com/unsubmail/unsubmail/releases).
