---

title: Configuring Zed - Settings and Preferences
description: Configure Zed with the Settings Editor, JSON files, and project-specific overrides. Covers settings options.

---

# Configuring Zed

Zed configures settings through the Settings Editor, JSON configuration files, and project-specific overrides.

See [Appearance](./appearance.md) for visual customization like themes, fonts, and icons.

## Settings Editor {#settings-editor}

The **Settings Editor** ({#kb zed::OpenSettings}) configures Zed. It provides a searchable interface to browse settings, view current values, and apply changes.

Open the Settings Editor by pressing {#kb zed::OpenSettings} or running {#action zed::OpenSettings} from the command palette.

Type in the search box to display matching settings with descriptions and modification controls. Zed saves changes to your settings file.

> **Note:** The Settings Editor omits some settings. Edit the JSON file directly to configure advanced options like language formatters.

## Settings Files {#settings-files}

### User Settings {#user-settings}

Your user settings apply across projects. Open the file with {#kb zed::OpenSettingsFile} or run {#action zed::OpenSettingsFile} from the command palette.

The file locations are:

- macOS: `~/.config/zed/settings.json`
- Linux: `~/.config/zed/settings.json` (or `$XDG_CONFIG_HOME/zed/settings.json`)
- Windows: `%APPDATA%\Zed\settings.json`

The syntax uses JSON and supports `//` comments.

### Default Settings {#default-settings}

Run {#action zed::OpenDefaultSettings} from the command palette to view settings and their default values. This read-only reference helps you edit your settings.

### Project Settings {#project-settings}

Create a `.zed/settings.json` file in your project root to override user settings for a specific project. Run {#action zed::OpenProjectSettings} to create this file.

Project settings override user settings for that project.

```json [settings]
// .zed/settings.json
{
  "tab_size": 2,
  "formatter": "prettier",
  "format_on_save": "on"
}
```

Add settings files in subdirectories for granular control.

> **Note:** Project settings only apply to editor behavior and language tooling options like `tab_size`, `formatter`, and `format_on_save`. Global settings like `theme` or `vim_mode` require user settings.

## How Settings Merge {#how-settings-merge}

Zed applies settings in layers:

1. **Default settings** apply built-in defaults.
2. **User settings** apply global preferences.
3. **Project settings** apply project-specific overrides.

Later layers override earlier layers. Object settings like `terminal` merge properties instead of replacing the entire object.

## Per-Release Channel Overrides {#release-channel-overrides}

Add top-level channel keys to apply different settings for Stable, Preview, or Nightly builds:

```json [settings]
{
  "theme": "One Dark",
  "vim_mode": false,
  "nightly": {
    "theme": "Rosé Pine",
    "vim_mode": true
  },
  "preview": {
    "theme": "Catppuccin Mocha"
  }
}
```

This configuration applies the following settings:

- **Stable** uses One Dark with vim mode disabled.
- **Preview** uses Catppuccin Mocha with vim mode disabled.
- **Nightly** uses Rosé Pine with vim mode enabled.

The Settings Editor applies changes across channels.

## Settings Deep Links {#deep-links}

Zed deep links open specific settings:

```
zed://settings/theme
zed://settings/vim_mode
zed://settings/buffer_font_size
```

Share these links to provide configuration tips or reference settings in documentation.

## Example Configuration {#example-configuration}

```json [settings]
{
  "theme": {
    "mode": "system",
    "light": "One Light",
    "dark": "One Dark"
  },
  "buffer_font_family": "JetBrains Mono",
  "buffer_font_size": 14,
  "tab_size": 2,
  "format_on_save": "on",
  "autosave": "on_focus_change",
  "vim_mode": false,
  "terminal": {
    "font_family": "JetBrains Mono",
    "font_size": 14
  },
  "languages": {
    "Python": {
      "tab_size": 4
    }
  }
}
```

## See Also {#see-also}

- [Appearance](./appearance.md) configures themes, fonts, and visual settings.
- [Key bindings](./key-bindings.md) customizes keyboard shortcuts.
- [AI Configuration](./ai/configuration.md) sets up AI providers, models, and agent settings.
- [All Settings](./reference/all-settings.md) provides a complete settings reference.
