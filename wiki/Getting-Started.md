# Getting Started

## Installation

### Download a release

Download the latest release for your platform from [Releases](https://github.com/aguspe/rubynaut/releases):

| Platform | Format | Notes |
|----------|--------|-------|
| macOS (Apple Silicon) | `.dmg` | M1/M2/M3/M4 |
| macOS (Intel) | `.dmg` | Pre-2020 Macs |
| Linux (Debian/Ubuntu) | `.deb` | `sudo dpkg -i rubynaut.deb` |
| Linux (Fedora/RHEL) | `.rpm` | `sudo rpm -i rubynaut.rpm` |
| Linux (Other) | `.AppImage` | `chmod +x` and run |
| Windows | `.msi` | Standard installer |

### macOS: Unsigned app warning

Since Rubynaut is not yet notarized by Apple, macOS may show "app is damaged" or block it. To allow it:

```bash
xattr -cr /Applications/Rubynaut.app
```

Or: System Settings > Privacy & Security > click "Open Anyway" after the first launch attempt.

## First-time setup

1. **Open Rubynaut**
2. **Install a Ruby version**: Go to the Install tab, pick a version (e.g., 4.0.2), click Install
3. **Set as global default**: On the Dashboard, click "Use" > "Set as Global Default"
4. **Install shell hook**: Go to Settings, find your shell (Zsh/Bash/Fish), click "Install Hook"
5. **Restart your terminal**
6. **Verify**: `ruby -v` should show your selected version

## File locations

| What | Where |
|------|-------|
| Ruby installations | `~/.rubies/<version>/` |
| Gems per version | `~/.rubies/gems/<version>/` |
| Config (global version, tracked projects) | `~/.rubies/config.json` |
| Version cache | `~/.rubies/versions_cache.json` |
| Shell hook | Appended to your `~/.zshrc`, `~/.bashrc`, or Fish config |
