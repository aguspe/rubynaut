# Rubynaut vs rbenv vs rvm: Which Ruby Version Manager Should You Use?

## What is a Ruby Version Manager?

A Ruby version manager lets you install multiple versions of Ruby on the same computer and switch between them. This is essential because different projects often require different Ruby versions — a Rails 7 app might need Ruby 3.3, while an older project might need Ruby 2.7.

Think of it like nvm for Node.js or pyenv for Python.

## Feature Comparison

| Feature | Rubynaut | rbenv | rvm |
|---------|----------|-------|-----|
| **GUI (desktop app)** | Yes | No | No |
| **CLI** | Yes (32 commands) | Yes | Yes |
| **Install speed** | Seconds (pre-built) | 5-15 min (compiles) | 5-15 min (compiles) |
| **Compile from source** | No | Yes | Yes |
| **Custom compile flags** | No | Yes | Yes |
| **Shell support** | bash, zsh, fish, PowerShell | bash, zsh | bash, zsh |
| **JRuby / TruffleRuby** | Built-in | Via plugin | JRuby only |
| **.ruby-version** | Yes | Yes | Yes |
| **.tool-versions (asdf/mise)** | Yes | No | No |
| **Gemfile ruby constraint** | Yes (detection) | No | No |
| **Gemsets** | No (use Bundler) | No (use Bundler) | Built-in |
| **Environment diagnostics** | Doctor tab with auto-fix | No | Basic |
| **Offline install** | Yes (--from-file) | No | No |
| **Self-update** | Built-in | Via package manager | `rvm get stable` |
| **Version aliases** | Yes | No | Yes |
| **Proxy / mirror support** | Yes (config) | Env vars | Env vars |
| **Default gems** | Yes (~/.rubies/default-gems) | Via plugin | No |
| **Project tracking** | GUI dashboard | No | No |
| **Auto-install on cd** | Yes (with prompt) | Via plugin | Via .rvmrc |
| **Platforms** | macOS, Linux | macOS, Linux | macOS, Linux |
| **Windows** | No (WSL works) | No (WSL works) | No (WSL works) |
| **Maturity** | v0.1.0 (new) | 10+ years | 10+ years |

## Decision Flowchart

**Are you new to Ruby?**
- Yes -> **Use Rubynaut.** It installs in seconds, has a GUI, and guides you through setup.

**Do you need to compile Ruby with custom flags (--enable-yjit, --with-jemalloc)?**
- Yes -> **Use rbenv + ruby-build.** Rubynaut only installs pre-built binaries.

**Are you on Alpine Linux, FreeBSD, or an unusual platform?**
- Yes -> **Use rbenv.** It compiles from source and works anywhere with a C compiler.

**Does your team already use rbenv?**
- Yes -> **Either works.** Rubynaut reads `.ruby-version` files, so it's fully compatible.

**Do you need gemsets?**
- Yes -> **Use rvm.** (But consider Bundler instead — most modern Ruby projects use it.)

**Do you want a visual overview of your Ruby environment?**
- Yes -> **Use Rubynaut.** It's the only option with a GUI.

## Use Rubynaut If...

- You're **learning Ruby for the first time** and don't want to debug compiler errors
- You're a **bootcamp student or instructor** and need fast, reliable setup
- You want a **desktop app** with visual version management and diagnostics
- You're setting up a **new machine** and want one command (`rubynaut init`) to get everything working
- You need to manage **multiple Ruby engines** (CRuby, JRuby, TruffleRuby)
- You work in a **corporate environment** with proxy/mirror requirements

## Use rbenv If...

- You're an **experienced Rubyist** comfortable with the terminal
- You need to **compile Ruby with custom flags** for performance tuning
- You use **Alpine Linux, FreeBSD**, or other platforms without pre-built binaries
- You want a **minimal footprint** — rbenv is just shell shims
- You need the **plugin ecosystem** (rbenv-vars, rbenv-each, etc.)

## Use rvm If...

- You **need gemsets** (isolated gem environments per project)
- You have **existing scripts** that depend on rvm commands
- You prefer **all-in-one** over modular (rvm bundles everything)

## Can I Switch?

Yes. All three tools use `.ruby-version` files, so your projects are compatible regardless of which tool you use. Ruby installations go in different directories (`~/.rubies` for Rubynaut, `~/.rbenv/versions` for rbenv, `~/.rvm/rubies` for rvm), so they don't conflict.

### Migrating from rbenv to Rubynaut

1. Install Rubynaut
2. Run `rubynaut init` to install and configure Ruby
3. Your existing `.ruby-version` files work automatically
4. Optionally remove rbenv: `brew uninstall rbenv ruby-build`

### Migrating from rvm to Rubynaut

1. Install Rubynaut
2. Run `rubynaut init` to install and configure Ruby
3. Your existing `.ruby-version` files work automatically
4. Optionally remove rvm: `rvm implode`

## When to Graduate from Rubynaut to rbenv

As you gain experience with Ruby, you might outgrow Rubynaut if you need:

- Custom compile flags for production optimization
- Building Ruby from a git branch or patch
- Platform support beyond macOS and Linux x64/ARM64
- A plugin ecosystem for advanced workflows

This is normal and expected. Rubynaut is designed to get you started quickly and remove friction — not to be the last tool you ever use.
