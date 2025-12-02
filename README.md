# UnsubMail v2

A fast, reliable CLI tool to clean your Gmail inbox from newsletters and spam.

## Features

- 🔐 **Secure OAuth2 Authentication** - Connect your Gmail account safely via Google OAuth2
- ⚡ **Fast Scanning** - Headers-only scanning for maximum speed
- 🎯 **Smart Detection** - Automatically detects newsletters and unwanted senders using List-Unsubscribe headers
- 🚀 **One-Click Unsubscribe** - Automatically unsubscribes when possible
- 🛡️ **Automatic Blocking** - Creates Gmail filters to block future emails
- 🗑️ **Batch Deletion** - Efficiently deletes all messages from unwanted senders
- 📊 **Interactive TUI** - Simple checkbox interface to select senders to clean

## Prerequisites

- Rust 1.70 or later
- A Google Cloud Project with Gmail API enabled
- OAuth2 credentials (Client ID and Client Secret)

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
./target/release/unsubmail --help
```

## Configuration

### 1. Create Google OAuth2 Credentials

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Enable the Gmail API
4. Create OAuth2 credentials (Desktop App type)
5. Download the credentials JSON file

### 2. Set Environment Variables

```bash
export GOOGLE_CLIENT_ID="your-client-id"
export GOOGLE_CLIENT_SECRET="your-client-secret"
```

Or add them to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.).

## Usage

### Add a Gmail Account

```bash
unsubmail add
```

This will open your browser to authenticate with Google and grant permissions.

### List Configured Accounts

```bash
unsubmail list
```

### Scan Inbox

```bash
unsubmail scan your-email@gmail.com
```

This will:
- Fetch up to 2000 messages (headers only)
- Analyze senders and detect newsletters
- Show top newsletter candidates with scores

### Clean Inbox (Scan + Select + Cleanup)

```bash
unsubmail clean your-email@gmail.com
```

This will:
1. Scan your inbox
2. Show an interactive selection interface
3. For each selected sender:
   - Attempt one-click unsubscribe (if available)
   - Block via Gmail filter (if unsubscribe fails)
   - Delete all messages from that sender (with confirmation)

### Remove an Account

```bash
unsubmail remove your-email@gmail.com
```

## Project Structure

```
src/
├── cli/                    # CLI commands
│   ├── accounts.rs         # Account management
│   ├── scan.rs             # Inbox scanning
│   ├── select.rs           # Interactive sender selection
│   └── actions.rs          # Cleanup actions
├── domain/                 # Business logic
│   ├── models.rs           # Core data models
│   ├── analysis.rs         # Email analysis & heuristics
│   └── planner.rs          # Action planning
├── infrastructure/         # External services
│   ├── google/
│   │   ├── auth.rs         # OAuth2 authentication
│   │   ├── gmail_api.rs    # Gmail API client
│   │   ├── filters.rs      # Filter management
│   │   └── delete.rs       # Batch deletion
│   ├── storage/
│   │   ├── keyring.rs      # Secure token storage
│   │   └── json_store.rs   # Account metadata storage
│   └── network/
│       └── http_client.rs  # Unsubscribe HTTP client
├── application/            # Workflow orchestration
│   └── workflow.rs         # Main workflow logic
└── main.rs                 # CLI entry point
```

## How It Works

### 1. Detection

UnsubMail analyzes email headers to detect newsletters:

- **List-Unsubscribe header** - Strong signal for newsletters
- **List-Unsubscribe-Post header** - Indicates one-click unsubscribe support
- **Email address patterns** - Detects common newsletter keywords
- **Message count** - High volume from same sender
- **Heuristic scoring** - Combines signals to prioritize senders

### 2. Cleanup Strategy

For each selected sender, UnsubMail follows this workflow:

```
If one-click unsubscribe available:
    → POST to unsubscribe URL
    → Delete all messages
Else:
    → Create Gmail filter to block future emails
    → Delete all messages
```

### 3. Safety

- Only HTTPS URLs are used for unsubscribe
- `mailto:` links are ignored
- Confirmation required before deletion
- All tokens stored securely in system keyring

## Development

### Build

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Run with Logging

```bash
RUST_LOG=unsubmail=debug cargo run -- clean your-email@gmail.com
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Disclaimer

This tool modifies your Gmail account (creates filters, deletes messages). Use at your own risk. Always review selections before confirming deletions.

## Roadmap

- [ ] Implement full OAuth2 flow
- [ ] Add parallel message fetching for better performance
- [ ] Implement proper RFC 2822 date parsing
- [ ] Add dry-run mode
- [ ] Support for multiple email providers (not just Gmail)
- [ ] Export scan results to JSON/CSV
- [ ] Machine learning-based sender classification
- [ ] Web UI

## Support

For issues, questions, or feature requests, please open an issue on GitHub.
