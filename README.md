# UnsubMail v2

A fast, reliable **interactive CLI** tool to clean your Gmail inbox from newsletters and spam.

## Features

- 💬 **Interactive Mode** - Guided menu with questions (default mode)
- 🔐 **Secure OAuth2 Authentication** - Connect your Gmail account safely via Google OAuth2
- ⚡ **Fast Scanning** - Headers-only scanning for maximum speed
- 🎯 **Smart Detection** - Automatically detects newsletters and unwanted senders using List-Unsubscribe headers
- 🚀 **One-Click Unsubscribe** - Automatically unsubscribes when possible
- 🛡️ **Automatic Blocking** - Creates Gmail filters to block future emails
- 🗑️ **Batch Deletion** - Efficiently deletes all messages from unwanted senders
- 📊 **Interactive Selection** - Simple checkbox interface to select senders to clean
- ⚙️ **Command Mode** - Also available for scripting and automation

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

### 🎯 Interactive Mode (Recommended)

Simply run without arguments to launch the interactive menu:

```bash
cargo run
# or
cargo run -- --interactive
# or
cargo run -- -i
```

You'll see a guided menu:

```
🔹 UnsubMail - Nettoyez votre Gmail

Que voulez-vous faire ?
❯ ➕ Ajouter un compte Gmail
  🔍 Scanner une boîte mail
  🧹 Nettoyer une boîte mail
  📋 Lister les comptes
  🚪 Quitter
```

**Navigate** with arrow keys (↑↓), **select** with Enter, **check boxes** with Space.

The interactive mode will:
- ✅ Guide you step by step
- ✅ Ask for account selection when needed
- ✅ Show clear prompts and confirmations
- ✅ Provide visual feedback (✓ ✗ →)

---

### ⚙️ Command Mode (For Scripts)

You can also use direct commands:

#### Add a Gmail Account

```bash
cargo run -- add
```

Opens browser for OAuth2 authentication.

#### List Configured Accounts

```bash
cargo run -- list
```

#### Scan Inbox

```bash
cargo run -- scan your-email@gmail.com
```

Shows top newsletter candidates with scores.

#### Clean Inbox (Scan + Select + Cleanup)

```bash
cargo run -- clean your-email@gmail.com
```

Full cleanup workflow with interactive selection.

#### Remove an Account

```bash
cargo run -- remove your-email@gmail.com
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
# or optimized release
cargo build --release
```

### Run Tests

```bash
cargo test
```

### Run with Logging

```bash
# Interactive mode with debug logs
RUST_LOG=unsubmail=debug cargo run

# Command mode with debug logs
RUST_LOG=unsubmail=debug cargo run -- clean your-email@gmail.com
```

### Quick Commands

Use the development helpers:

**Windows:**
```powershell
.\dev.ps1 run        # Interactive mode
.\dev.ps1 all        # Run all checks
```

**Linux/Mac:**
```bash
./dev.sh run         # Interactive mode
make all             # Run all checks
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
