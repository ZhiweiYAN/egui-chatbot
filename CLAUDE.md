# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

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

## Architecture Overview

This is an egui/eframe GUI application template that supports both native and web deployment.

### Core Structure
- **src/main.rs** - Entry point with platform-specific initialization (native vs WASM)
- **src/lib.rs** - Library root that exports the main app struct
- **src/app.rs** - Main application logic implementing the `eframe::App` trait

### Key Components
- **TemplateApp** - Main application struct with state persistence via serde
- **Native vs Web** - Conditional compilation supports both targets seamlessly
- **State Management** - Automatic persistence using eframe's storage system
- **UI Layout** - Uses egui panels (TopBottomPanel, CentralPanel) for responsive design

### Dependencies
- **egui 0.32** - Immediate mode GUI framework
- **eframe 0.32** - Application framework with native/web support
- **serde** - Serialization for state persistence
- **trunk** - Web build tool for WASM deployment

## Development Notes

- The project name is currently "eframe_template" - update Cargo.toml, main.rs, index.html, and assets/sw.js when renaming
- Web builds require the wasm32-unknown-unknown target
- State is automatically persisted between sessions when persistence feature is enabled
- The app supports both light and dark themes with global theme switching
- CI-like checks are available via the check.sh script