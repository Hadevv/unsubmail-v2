# UnsubMail

> Clean your Gmail inbox from newsletters and spam with ease

[![CI](https://github.com/unsubmail/unsubmail/workflows/CI/badge.svg)](https://github.com/unsubmail/unsubmail/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/unsubmail.svg)](https://crates.io/crates/unsubmail)

UnsubMail is a modern, fast, and reliable Rust CLI tool designed to help you reclaim your inbox. It automatically detects newsletters and promotional emails, offers one-click unsubscribe when available, and helps you bulk clean your Gmail account.

## Features

- **Smart Detection** - Advanced heuristics identify newsletters and promotional emails
- **One-Click Unsubscribe** - Automatic unsubscribe via RFC 8058 List-Unsubscribe-Post
- **OAuth2 Authentication** - Secure Gmail access without passwords
- **Fast & Efficient** - Headers-only scanning for maximum performance
- **Interactive TUI** - Beautiful terminal interface with progress indicators
- **Batch Operations** - Clean hundreds of emails in seconds
- **Safe** - Preview before deletion, secure token storage in OS keyring
- **Multi-Account** - Manage multiple Gmail accounts
- **Loop Mode** - Clean multiple senders or switch accounts seamlessly

## Installation

### From Crates.io (Recommended)

```bash
cargo install unsubmail
```

### From Source

```bash
git clone https://github.com/unsubmail/unsubmail.git
cd unsubmail
cargo install --path .
```

### Requirements

- Rust 1.70 or later
- Gmail account with OAuth2 credentials (see [Configuration](#configuration))

## Quick Start

1. **Set up OAuth2 credentials** (see [Configuration](#configuration))

2. **Run UnsubMail**:
   ```bash
   unsubmail
   ```

3. **Follow the interactive prompts**:
   - Enter your Gmail address
   - Authenticate via OAuth2 (browser opens automatically)
   - Review detected newsletters
   - Select senders to clean
   - Choose cleanup actions (unsubscribe, block, or delete)

4. **Done!** Your inbox is now cleaner.

## Configuration

### OAuth2 Setup

UnsubMail requires OAuth2 credentials to access Gmail. Here's how to set them up:

1. **Create a Google Cloud Project**:
   - Go to [Google Cloud Console](https://console.cloud.google.com/)
   - Create a new project or select an existing one

2. **Enable Gmail API**:
   - Navigate to "APIs & Services" > "Library"
   - Search for "Gmail API" and enable it

3. **Create OAuth2 Credentials**:
   - Go to "APIs & Services" > "Credentials"
   - Click "Create Credentials" > "OAuth client ID"
   - Application type: "Desktop app"
   - Download the JSON file

4. **Set Environment Variables**:
   Create a `.env` file in your home directory or project directory:
   ```bash
   GOOGLE_CLIENT_ID=your_client_id_here.apps.googleusercontent.com
   GOOGLE_CLIENT_SECRET=your_client_secret_here
   ```

   Or export them in your shell:
   ```bash
   export GOOGLE_CLIENT_ID="your_client_id"
   export GOOGLE_CLIENT_SECRET="your_client_secret"
   ```

### Token Storage

- Access tokens are securely stored in your OS keyring:
  - **macOS**: Keychain
  - **Windows**: Credential Manager
  - **Linux**: Secret Service (gnome-keyring, KWallet, etc.)

- Account metadata is stored in: `~/.config/unsubmail/accounts/`

## How It Works

### Newsletter Detection

UnsubMail uses multiple heuristics to identify newsletters and promotional emails:

1. **List-Unsubscribe Header** (RFC 2369): Strong signal (+0.5 score)
2. **Email Patterns**: `newsletter@`, `noreply@`, `marketing@`, etc. (+0.3 score)
3. **Message Volume**: High message count from single sender (+0.2-0.5 score)
4. **One-Click Support** (RFC 8058): Detects automated unsubscribe capability

Senders with a heuristic score >= 0.6 OR with an unsubscribe header are presented for cleanup.

### Cleanup Strategy

For each selected sender:

```
if has_one_click_unsubscribe:
    POST to unsubscribe URL (HTTPS only)
    optionally delete all messages
else:
    create Gmail filter (auto-trash future messages)
    OR move existing messages to spam
    optionally delete all messages
```

### Security

- **OAuth2 Only**: No passwords or IMAP credentials stored
- **HTTPS Only**: Unsubscribe links must use HTTPS
- **No mailto**: `mailto:` unsubscribe links are rejected (require manual action)
- **Secure Storage**: Tokens stored in OS-native secure storage

## Architecture

UnsubMail follows clean architecture principles:

```
src/
├── cli/                    # CLI interface layer
│   ├── interactive.rs      # Interactive TUI mode
│   └── mod.rs
│
├── domain/                 # Business logic (pure)
│   ├── models.rs           # Core data structures
│   ├── analysis.rs         # Newsletter detection heuristics
│   └── planner.rs          # Action planning
│
├── infrastructure/         # External services
│   ├── imap/               # IMAP client (Gmail)
│   ├── storage/            # Keyring & JSON storage
│   └── network/            # HTTP client (unsubscribe)
│
└── application/            # Orchestration
    └── workflow.rs         # Main workflows
```

See [CLAUDE.md](CLAUDE.md) for detailed architecture documentation.

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, coding standards, and how to contribute.

### Quick Commands

```bash
# Build
cargo build

# Run with debug logging
RUST_LOG=unsubmail=debug cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Build release
cargo build --release
```

## Examples

See the [examples/](examples/) directory for code examples:

- `simple_scan.rs`: Scan inbox and print results
- `batch_cleanup.rs`: Non-interactive batch cleanup

## Roadmap

- [ ] Support for other email providers (Outlook, Yahoo, etc.)
- [ ] Dry-run mode (preview without executing)
- [ ] Export results to JSON/CSV
- [ ] Email template unsubscribe (handle mailto links)
- [ ] Undo functionality
- [ ] GUI application

## FAQ

**Q: Is my data safe?**
A: Yes. UnsubMail uses OAuth2 for authentication and stores tokens securely in your OS keyring. No passwords are stored. The tool only reads email headers and metadata, never message bodies.

**Q: Will this delete important emails?**
A: UnsubMail only presents senders with high newsletter scores or explicit unsubscribe headers. Personal emails are filtered out. You always review and confirm before any deletion.

**Q: Does it work with Google Workspace accounts?**
A: Yes, as long as IMAP access is enabled and you configure OAuth2 credentials.

**Q: What if one-click unsubscribe fails?**
A: The tool will report the failure and let you choose to block the sender or delete messages manually.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

UnsubMail is licensed under the [MIT License](LICENSE).

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [async-imap](https://github.com/async-email/async-imap) for Gmail IMAP access
- Inspired by the need for inbox sanity

## Support

- **Issues**: [GitHub Issues](https://github.com/unsubmail/unsubmail/issues)
- **Discussions**: [GitHub Discussions](https://github.com/unsubmail/unsubmail/discussions)

---

Made with heart by the UnsubMail contributors
