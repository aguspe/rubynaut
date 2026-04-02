# Dashboard

The Dashboard is the main screen showing your Ruby environment at a glance.

## Stat cards

- **Installed** - Number of Ruby versions installed
- **Active Version** - The currently active global Ruby version
- **Available** - Number of versions available to download

## Installed versions list

Each installed Ruby shows:

- **Version number** (e.g., `4.0.2`)
- **Badges**: "Active" (currently resolving in shell), "Global" (the default)
- **Gems button** - Toggle to see default and user-installed gems for this version
- **Projects button** - Toggle to see which tracked projects use this version
- **Use dropdown**:
  - **Set as Global Default** - Makes this version the default in all terminals
  - **Set as Local (Project)** - Opens a folder picker, writes `.ruby-version` to the selected directory
- **Remove** - Uninstalls this Ruby version and its gems

## Inline panels

Clicking **Gems** or **Projects** expands an inline panel directly below the version row. Click again to collapse. Only one panel per version can be open at a time.

### Gems panel
- Shows all default gems (shipped with Ruby) and user-installed gems
- Filter by name, toggle default/user visibility
- Install new gems with name + optional version
- Remove user-installed gems

### Projects panel
- Shows tracked projects that use this Ruby version
- Each project shows its name, path, and detection source (`.ruby-version`, `.tool-versions`, `Gemfile`)
- Remove projects from tracking
