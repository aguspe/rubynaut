<p align="center">
  <img src="frontend/dist/icons/rubynaut-logo.svg" width="128" height="128" alt="Rubynaut">
</p>

<h1 align="center">Rubynaut</h1>

<p align="center">
  <strong>A cross-platform GUI tool for installing and managing Ruby versions.</strong>
</p>

<p align="center">
  <a href="#features">Features</a> &middot;
  <a href="#installation">Installation</a> &middot;
  <a href="#usage">Usage</a> &middot;
  <a href="#development">Development</a> &middot;
  <a href="#contributing">Contributing</a>
</p>

---

## Features

- **One-click Ruby installs** - Downloads pre-built binaries from [ruby-builder](https://github.com/ruby/ruby-builder). No compilation, no waiting. Ruby 1.9.3 through 4.0.x.
- **Version switching** - Set global defaults or pin per-project via `.ruby-version` files. Shell hook auto-switches on `cd`.
- **Gem management** - Browse, install, and remove gems per Ruby version. See default vs user-installed gems at a glance.
- **Project tracking** - Open project folders to detect their Ruby version. Track multiple projects and see which Ruby each uses.
- **Bundle install** - Run `bundle install` from the GUI with the correct Ruby environment.
- **Doctor diagnostics** - Detect missing libraries (libyaml, OpenSSL, libffi, GMP), conflicting version managers, broken PATH, and more. One-click fixes.
- **Shell integration** - Auto-detect and install hooks for Zsh, Bash, Fish, and PowerShell.
- **Cross-platform** - macOS (ARM64 + Intel), Linux (x64 + ARM64). Windows via WSL.

## Installation

### Download

Download the latest release for your platform from [Releases](https://github.com/aguspe/rubynaut/releases).

| Platform | Format |
|----------|--------|
| macOS (Apple Silicon) | `.dmg` |
| macOS (Intel) | `.dmg` |
| Linux (Debian/Ubuntu) | `.deb` |
| Linux (Fedora/RHEL) | `.rpm` |
| Linux (Other) | `.AppImage` |
| Windows | `.msi` |

### Build from source

```bash
# Prerequisites: Rust 1.77+, platform-specific Tauri dependencies (see CONTRIBUTING.md)
cd src-tauri
cargo tauri build
```

## Usage

1. **Install Ruby** - Go to the Install tab, pick a version, click Install. Takes seconds.
2. **Set as global** - On the Dashboard, click "Use" > "Set as Global Default" on any installed version.
3. **Pin to project** - Click "Use" > "Set as Local (Project)" and pick a folder. Writes a `.ruby-version` file.
4. **Shell hook** - Go to Settings and click "Install Hook" for your shell. Restart your terminal.
5. **Verify** - Open a terminal: `ruby -v` should show your selected version.

## How it works

Rubynaut downloads pre-built Ruby binaries from the [ruby/ruby-builder](https://github.com/ruby/ruby-builder) project (the same source that powers GitHub Actions' `setup-ruby`). Binaries are extracted to `~/.rubies/<version>/` and activated via environment variables (`PATH`, `GEM_HOME`, `GEM_PATH`).

Version switching uses a small shell hook (installed in your `.zshrc`/`.bashrc`/etc.) that reads `.ruby-version` files and adjusts `PATH` on `cd`. No shims, no overhead.

## Development

```bash
# Run in development mode (hot-reload for frontend)
cd src-tauri
cargo tauri dev

# Run tests
cargo test --all

# Build for release
cargo tauri build
```

### Project structure

```
rubynaut/
├── frontend/dist/       # HTML/CSS/JS frontend (no build step)
│   ├── index.html       # App layout and tabs
│   ├── styles.css       # Ruby-themed design system
│   ├── app.js           # Frontend logic and Tauri IPC
│   └── icons/           # App icon and SVGs
├── src-tauri/
│   ├── src/
│   │   ├── main.rs      # Desktop entry point
│   │   ├── lib.rs       # Command registration
│   │   └── commands/
│   │       ├── platform.rs   # OS/arch/package manager detection
│   │       ├── ruby.rs       # Ruby install/uninstall/gems/projects/shell hooks
│   │       └── doctor.rs     # Environment diagnostics and auto-fix
│   ├── Cargo.toml
│   └── tauri.conf.json
├── .github/workflows/   # CI pipeline
├── LICENSE              # MIT
└── CONTRIBUTING.md
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions and guidelines.

## License

MIT - See [LICENSE](LICENSE) for details.
