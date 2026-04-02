# Architecture

## Stack

- **Backend**: Rust (via Tauri v2)
- **Frontend**: Vanilla HTML/CSS/JS (no framework, no build step)
- **UI framework**: Tauri v2 (uses OS-native webview, not Electron/Chromium)
- **Binary size**: ~5-10 MB (vs ~150 MB for Electron apps)

## Directory layout on disk

```
~/.rubies/
├── 4.0.2/                  # Ruby installation
│   ├── bin/ruby, gem, bundle, irb, ...
│   ├── lib/ruby/4.0.0/    # Standard library
│   └── include/
├── 3.3.6/                  # Another version
│   └── ...
├── gems/
│   ├── 4.0.2/             # Gems for Ruby 4.0.2
│   │   ├── gems/
│   │   ├── specifications/
│   │   └── bin/
│   └── 3.3.6/             # Gems for Ruby 3.3.6
├── config.json             # Global version + tracked projects
└── versions_cache.json     # Cached available versions list
```

## Version switching

Ruby version switching works via environment variables. There are no shims (unlike rbenv).

When a version is activated:
```
PATH=~/.rubies/<version>/bin:~/.rubies/gems/<version>/bin:$PATH
GEM_HOME=~/.rubies/gems/<version>
GEM_PATH=$GEM_HOME:~/.rubies/<version>/lib/ruby/gems/<api_version>
RUBYNAUT_VERSION=<version>
```

The shell hook (`rubynaut_switch`) runs on every `cd` and:
1. Walks up the directory tree looking for `.ruby-version`
2. Falls back to the global version from `config.json`
3. If the version changed, swaps the environment variables

## Pre-built binary fixes

Ruby binaries from ruby-builder have hardcoded paths from the GitHub Actions build environment. Rubynaut fixes three layers at install time:

1. **`libruby.dylib` references** in `bin/ruby` and all `.bundle` files -- fixed with `install_name_tool -change` (macOS) or `patchelf --set-rpath` (Linux)
2. **Shebangs** in `bin/gem`, `bin/bundle`, etc. -- rewritten from `#!/Users/runner/...` to the actual Ruby path
3. **`$LOAD_PATH`** -- cannot be patched in the binary; fixed at runtime by setting `RUBYLIB` environment variable

## IPC between frontend and backend

The frontend calls Rust functions via Tauri's invoke mechanism:
```js
const result = await invoke('command_name', { arg1: value1 });
```

Events flow back from Rust to the frontend for real-time updates:
```rust
window.emit("event-name", payload)?;
```
```js
listen('event-name', (event) => { ... });
```

## Config format

`~/.rubies/config.json`:
```json
{
  "global_version": "4.0.2",
  "projects": [
    { "path": "/Users/me/myapp", "name": "myapp" }
  ]
}
```

Backward compatible: old configs with only `global_version` work fine (the `projects` field defaults to `[]`).
