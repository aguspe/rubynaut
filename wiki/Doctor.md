# Doctor

The Doctor tab runs diagnostics on your Ruby environment and offers one-click fixes.

## Checks performed

| Check | What it verifies | Fix |
|-------|-----------------|-----|
| **Rubies directory** | `~/.rubies` exists | Install a Ruby version |
| **Installed Rubies** | At least one Ruby is installed | Go to Install tab |
| **Shell integration** | Rubynaut hook in your shell RC file | Go to Settings |
| **PATH resolution** | `ruby` resolves to a Rubynaut-managed binary | Install shell hook + restart terminal |
| **Conflicting managers** | No rbenv/rvm/mise/asdf/chruby detected | Remove or disable conflicting managers |
| **C compiler** | `cc`/`gcc`/`clang` available | `xcode-select --install` (macOS) or install build-essential (Linux) |
| **libyaml** | YAML parsing library for `psych.bundle` | `brew install libyaml` / `apt install libyaml-dev` |
| **OpenSSL** | TLS/SSL for HTTPS connections | `brew install openssl` / `apt install libssl-dev` |
| **libffi** | Foreign function interface for `fiddle` | `brew install libffi` / `apt install libffi-dev` |
| **GMP** | Arbitrary precision arithmetic for OpenSSL | `brew install gmp` / `apt install libgmp-dev` |
| **GEM_HOME override** | No conflicting `GEM_HOME` env var | Check shell config |

## Auto-fix

Checks with a **Fix** button can be resolved with one click. The fix command runs directly (e.g., `brew install libyaml`). After the fix, Doctor automatically re-runs to verify.

## Shared library details

Ruby pre-built binaries from ruby-builder link against system libraries. These must be present:

### macOS
Libraries are expected in Homebrew paths (`/opt/homebrew/lib/` for ARM64, `/usr/local/lib/` for Intel). Rubynaut sets `DYLD_FALLBACK_LIBRARY_PATH` automatically when running Ruby commands.

### Linux
Libraries are expected in standard system paths (`/usr/lib/x86_64-linux-gnu/`, `/usr/lib64/`, etc.). Rubynaut sets `LD_LIBRARY_PATH` when running Ruby commands. The Doctor detects the correct package manager (apt, dnf, pacman, apk) and generates the appropriate install command.

### Why libraries go missing
- Removing a Ruby version manager (e.g., rbenv) via Homebrew can auto-remove shared libraries that were dependencies
- Minimal Docker images or fresh server installs may not have these libraries
- Alpine Linux uses musl instead of glibc -- ruby-builder binaries are incompatible
