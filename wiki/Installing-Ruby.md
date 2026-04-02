# Installing Ruby

## How installation works

Rubynaut downloads pre-built Ruby binaries from [ruby/ruby-builder](https://github.com/ruby/ruby-builder), the same source that powers GitHub Actions' `setup-ruby`. No compilation is needed -- installs take seconds, not minutes.

## Available versions

- Ruby 4.0.x through 1.9.3
- Versions are fetched from the GitHub API and cached for 1 hour
- If the API is unavailable (rate limited, offline), a built-in fallback list is used

## Installation steps

1. Go to the **Install** tab
2. Browse or scroll to find your desired version
3. Click **Install** on the version card
4. Watch the progress bar: Download > Extract > Fix paths > Verify
5. Done -- the version appears on your Dashboard

## What happens during install

1. **Download**: Fetches the `.tar.gz` binary for your platform from GitHub
2. **Extract**: Unpacks to `~/.rubies/<version>/`
3. **Fix library paths** (macOS): Uses `install_name_tool` to fix hardcoded dylib references in the `ruby` binary and all `.bundle` files
4. **Fix shebangs**: Rewrites `#!/Users/runner/...` in `gem`, `bundle`, `irb`, etc. to point to the actual Ruby path
5. **Verify**: Runs `ruby --version` to confirm the binary works

## Platform support

| Platform | Binary source | Notes |
|----------|--------------|-------|
| macOS ARM64 | `darwin-arm64` | Apple Silicon (M1+) |
| macOS x64 | `darwin-x64` | Intel Macs |
| Linux x64 | `ubuntu-22.04-x64` | glibc-based distros |
| Linux ARM64 | `ubuntu-22.04-arm64` | Raspberry Pi, AWS Graviton |
| Windows | Not yet supported | Use WSL for now |

## Troubleshooting

**"Ruby binary failed verification"**: The pre-built binary has hardcoded library paths. Rubynaut fixes these automatically, but if it fails, run Doctor to check for missing libraries (libyaml, OpenSSL, etc.).

**"Download failed: 404"**: The specific version may not have pre-built binaries for your platform. Older versions (< 2.3) may only have x64 builds.
