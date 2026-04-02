# Contributing to Rubynaut

Thank you for your interest in contributing to Rubynaut! This project is built for the Ruby community, by the Ruby community.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (1.77.2+)
- [Node.js](https://nodejs.org/) (18+) — only needed if you add a build step to the frontend
- Platform-specific Tauri dependencies:
  - **macOS:** Xcode Command Line Tools (`xcode-select --install`)
  - **Linux:** `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`
  - **Windows:** [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) and Visual Studio Build Tools

### Building

```bash
cd src-tauri
cargo tauri dev
```

### Project Structure

```
rubynaut/
├── frontend/dist/       # HTML/CSS/JS frontend (no build step needed)
│   ├── index.html
│   ├── styles.css
│   └── app.js
├── src-tauri/
│   ├── src/
│   │   ├── main.rs      # Desktop entry point
│   │   ├── lib.rs        # App setup and command registration
│   │   └── commands/
│   │       ├── platform.rs   # OS/arch/package manager detection
│   │       ├── ruby.rs       # Ruby install/uninstall/version switching
│   │       └── doctor.rs     # Environment diagnostics
│   ├── Cargo.toml
│   └── tauri.conf.json
├── LICENSE
└── CONTRIBUTING.md
```

## How to Contribute

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Test on your platform
5. Submit a pull request

## Areas Where Help is Needed

- **Windows testing** — WSL integration, RubyInstaller support
- **Linux distro coverage** — testing on Fedora, Arch, Alpine
- **Shell hooks** — Nushell, Elvish, and other shell support
- **Pre-built binary sources** — expanding version coverage
- **Accessibility** — keyboard navigation, screen reader support
- **Translations** — i18n for the UI

## Code Style

- **Rust:** Follow `cargo clippy` recommendations. Run `cargo fmt` before committing.
- **Frontend:** Vanilla JS, no framework. Keep it simple and accessible.

## Reporting Issues

When filing a bug, please include:
- Your OS and architecture
- The output of "Doctor" diagnostics
- Steps to reproduce

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
