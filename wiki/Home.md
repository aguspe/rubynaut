# Rubynaut Wiki

Welcome to the Rubynaut wiki -- the cross-platform GUI tool for installing and managing Ruby versions.

## Pages

- [Getting Started](Getting-Started) - Installation and first-time setup
- [Dashboard](Dashboard) - Managing installed Ruby versions
- [Installing Ruby](Installing-Ruby) - How to download and install Ruby versions
- [Projects](Projects) - Track projects and auto-detect Ruby versions
- [Gems](Gems) - Managing gems per Ruby version and per project
- [Doctor](Doctor) - Diagnosing and fixing environment issues
- [Settings](Settings) - Shell integration and configuration
- [Architecture](Architecture) - How Rubynaut works under the hood
- [Building from Source](Building-from-Source) - Development setup and contributing
- [App Signing](App-Signing) - Distributing signed builds for macOS and Windows

## How it works

Rubynaut downloads pre-built Ruby binaries from [ruby/ruby-builder](https://github.com/ruby/ruby-builder) (the same source that powers GitHub Actions' `setup-ruby`). Binaries are extracted to `~/.rubies/<version>/` and activated via environment variables.

Version switching uses a shell hook that reads `.ruby-version` files and adjusts `PATH` on `cd`.
