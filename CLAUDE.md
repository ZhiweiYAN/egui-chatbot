# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Environment Setup
Before running the application, set these environment variables:
```bash
export LLM_BASE_URL="https://api.deepseek.com"    # Default: https://api.deepseek.com
export LLM_API_KEY="your-api-key-here"            # Required: Your LLM API key
export LLM_MODEL="deepseek-chat"                  # Default: deepseek-chat
```

### Native Development
- `cargo run --release` - Run the application natively
- `cargo check --workspace --all-targets` - Check code for compilation errors
- `cargo fmt --all -- --check` - Check code formatting
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` - Run linting
- `cargo test --workspace --all-targets --all-features` - Run tests

### Web Development
- `rustup target add wasm32-unknown-unknown` - Add WASM target (one-time setup)
- `cargo install --locked trunk` - Install Trunk for web builds (one-time setup)
- `trunk serve` - Build and serve for web development at http://127.0.0.1:8080
- `trunk build --release` - Build for web deployment

### Quality Assurance
- `./check.sh` - Run comprehensive CI checks (format, clippy, tests, WASM build)
- `cargo check --workspace --all-features --lib --target wasm32-unknown-unknown` - Check WASM compatibility

### macOS App Bundle
- `./create_app_bundle.sh` - Create macOS .app bundle in `target/release/Pbot.app`

## Architecture Overview

This is an egui/eframe chatbot application with LLM integration, SQLite persistence, and a three-panel interface for managing conversations and memory.

### Application Structure
- **src/main.rs** - Entry point with platform-specific initialization (native vs WASM, Tokio runtime setup)
- **src/lib.rs** - Library root that exports `TemplateApp`
- **src/app.rs** - Main application state, data structures, LLM streaming logic, and business logic
- **src/chat_panel.rs** - Left panel: chat history with search and message management
- **src/digest_panel.rs** - Center panel: digested content with selection and summarization
- **src/long_mem_panel.rs** - Right panel: long-term memory items with selection and summarization
- **src/database.rs** - SQLite database layer for persistence (content deduplication, panel associations, assistant roles, system prompts)

### Key Components

#### Three-Panel UI Layout
1. **Chat Panel (Left)** - Displays conversation history with user/assistant messages
2. **Digest Panel (Center)** - Stores important excerpts from conversations for later reference
3. **Long-term Memory Panel (Right)** - Archives significant information across sessions

Each panel supports:
- Search functionality with real-time highlighting
- Content export to markdown
- Deletion and management of items
- Summarization via LLM (digest and memory panels)

#### Database Schema
The SQLite database uses a normalized schema with content deduplication:
- **content_items** - Stores unique content with role/source and timestamps
- **panel_associations** - Maps content to panels (chat/digest/longterm) with soft delete support
- **assistant_roles** - Defines different assistant personalities/roles
- **system_prompts** - Panel-specific system prompts for each assistant role

Database location varies by platform:
- Windows: `%APPDATA%\egui-chatbot\chat_data.db`
- macOS: `~/Library/Application Support/egui-chatbot/chat_data.db`
- Linux: `~/.local/share/egui-chatbot/chat_data.db`

#### LLM Integration
- Streaming responses via SSE (Server-Sent Events) from OpenAI-compatible API
- Asynchronous API calls using Tokio with `mpsc` channels for UI updates
- Role-based system prompts loaded from database per panel type
- Summary generation sends only selected content (not full chat history) to optimize tokens

#### State Management
- App state persisted via eframe's built-in storage (serde serialization)
- Database content loaded on-demand via "Load from DB" menu option
- Markdown rendering using `egui_commonmark`
- Icon integration via `egui-phosphor`

### Platform-Specific Features
- **Native**: Environment logging, custom window icon, async Tokio runtime
- **WASM**: Web logger, canvas-based rendering, wasm-bindgen futures
- **Font handling**: Automatic loading of system CJK fonts (Microsoft YaHei on Windows, PingFang on macOS, Noto Sans CJK on Linux from `/usr/share/fonts/noto-cjk/`)

### Dependencies
- **egui 0.32** / **eframe 0.32** - Immediate mode GUI framework
- **reqwest** - HTTP client for LLM API calls with streaming support
- **tokio** - Async runtime for concurrent operations
- **rusqlite** - SQLite database with bundled library
- **chrono** - Timestamp formatting
- **serde** / **serde_json** - Serialization for state and API communication
- **egui_commonmark** - Markdown rendering in panels
- **egui-phosphor** - Icon font integration

## Development Notes

- The package name is `eframe_template` but the app is branded as "Pbot" (see src/main.rs:22)
- Strict linting rules enforced via Cargo.toml workspace lints (unsafe code denied, many clippy warnings)
- The application expects LLM_API_KEY environment variable to be set for chat functionality
- Text highlighting for search results uses character-based matching to safely handle multi-byte Unicode (CJK support)
- Debug output prints full HTTP request payloads to LLM API (see app.rs:766, 878)
