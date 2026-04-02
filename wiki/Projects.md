# Projects

The Projects tab lets you open project folders, detect their Ruby version requirements, and manage dependencies.

## Opening a project

1. Click **Open Project Folder**
2. Select a directory in the native file picker
3. Rubynaut scans for Ruby version info and adds it to your tracked projects

## Detection sources (in priority order)

1. **`.ruby-version`** - Single line with the version number (e.g., `3.3.6`)
2. **`.tool-versions`** - asdf/mise format, reads the `ruby` line
3. **`Gemfile`** - Parses `ruby "3.3.6"` or `ruby "~> 3.3.0"` constraints

## Tracked projects list

All opened projects are remembered in `~/.rubies/config.json`. Each project shows:

- **Project name** (folder name)
- **Path**
- **Ruby version** + source file
- **Status badge**: Ready (green), Not Installed (red), No Version (yellow), Folder Missing (grey)

## Actions per project

- **Gems** - View gems from `Gemfile.lock` (top-level dependencies only, not transitive)
- **Bundle** - Run `bundle install` with the correct Ruby version and environment
- **Install** - Download the required Ruby version if it's not installed
- **Remove** - Stop tracking this project

## Bundle Install

When you click **Bundle**, Rubynaut runs `bundle install` in the project directory with:
- The correct Ruby binary from `~/.rubies/<version>/bin/ruby`
- `GEM_HOME` and `GEM_PATH` set correctly
- `RUBYLIB` set to fix load path issues with pre-built binaries
- `DYLD_FALLBACK_LIBRARY_PATH` (macOS) or `LD_LIBRARY_PATH` (Linux) for shared libraries

The output streams in a log panel below the project.

## Project gems from Gemfile.lock

The Gems view parses `Gemfile.lock` and shows all top-level gem dependencies with their locked versions. Sub-dependencies (transitive) are excluded for clarity.
