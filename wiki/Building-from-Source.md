# Building from Source

## Prerequisites

- [Rust](https://rustup.rs/) 1.77.2+
- Platform-specific Tauri dependencies:

### macOS
```bash
xcode-select --install
```

### Linux (Debian/Ubuntu)
```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

### Linux (Fedora)
```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file \
  libxdo-devel libappindicator-gtk3-devel librsvg2-devel
```

### Windows
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
- Visual Studio Build Tools with C++ workload

## Development

```bash
# Clone the repository
git clone https://github.com/aguspe/rubynaut.git
cd rubynaut

# Run in dev mode (hot-reload for frontend changes)
cd src-tauri
cargo tauri dev

# Run tests
cargo test --all

# Build for release
cargo tauri build
```

## Project structure

```
rubynaut/
├── frontend/dist/           # Frontend (no build step needed)
│   ├── index.html           # App layout, all tabs
│   ├── styles.css           # Ruby-themed CSS design system
│   ├── app.js               # All frontend logic + Tauri IPC
│   └── icons/               # SVG logo and nav icons
├── src-tauri/
│   ├── src/
│   │   ├── main.rs          # Desktop entry point
│   │   ├── lib.rs           # Plugin init + command registration
│   │   └── commands/
│   │       ├── mod.rs        # Module declarations
│   │       ├── platform.rs   # OS, arch, package manager, shell detection
│   │       ├── ruby.rs       # Core: install, uninstall, version switching, gems,
│   │       │                 #   projects, shell hooks, Gemfile.lock parser
│   │       └── doctor.rs     # Diagnostics: library checks, auto-fix
│   ├── Cargo.toml
│   ├── tauri.conf.json       # Window config, CSP, bundle settings
│   ├── capabilities/         # Tauri v2 permission grants
│   └── icons/                # App icons (.icns, .ico, .png)
├── .github/workflows/        # CI/CD pipeline
├── wiki/                     # GitHub wiki content
├── LICENSE                   # MIT
├── CONTRIBUTING.md
└── README.md
```

## Running tests

```bash
cd src-tauri

# Run all tests
cargo test --all

# Run with output
cargo test --all -- --nocapture

# Run a specific test
cargo test test_parse_gemfile_lock

# Run tests with coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out html
```
