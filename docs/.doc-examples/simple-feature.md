<!--
  GOLD STANDARD EXAMPLE: Simple Feature / Overview Documentation

  This example demonstrates documentation for a feature overview or navigation guide.

  Key patterns to note:
  - Anchor IDs on sections
  - Opening paragraph explains the scope
  - Concise sections (1-2 paragraphs max)
  - Links to detailed documentation
  - Quick reference table at the end
  - {#kb ...} syntax for keybindings
-->

---

title: Finding and Navigating Code - Zed
description: Navigate your codebase in Zed with file finder, project search, go to definition, symbol search, and the command palette.

---

# Finding & Navigating

Zed provides several tools to navigate your codebase.

## Command Palette {#command-palette}

The Command Palette ({#kb command_palette::Toggle}) provides access to Zed features. Type a few characters to filter commands. Press Enter to execute.

[Learn more about the Command Palette →](./command-palette.md)

## File Finder {#file-finder}

Open a file in your project with {#kb file_finder::Toggle}. Type part of the filename or path to narrow results.

## Project Search {#project-search}

Search across files with {#kb pane::DeploySearch}. Results appear in a [multibuffer](./multibuffers.md). You can edit matches in place.

## Go to Definition {#go-to-definition}

Jump to a symbol definition with {#kb editor::GoToDefinition} (or `Cmd+Click` / `Ctrl+Click`). Multiple definitions open in a multibuffer.

## Go to Symbol {#go-to-symbol}

- **Current file:** {#kb outline::Toggle} opens a symbol outline for the active file
- **Entire project:** {#kb project_symbols::Toggle} searches symbols across files

## Outline Panel {#outline-panel}

The Outline Panel ({#kb outline_panel::ToggleFocus}) shows a persistent tree view of symbols in the current file. Use it with [multibuffers](./multibuffers.md) to navigate search results or diagnostics.

[Learn more about the Outline Panel →](./outline-panel.md)

## Tab Switcher {#tab-switcher}

Switch between open tabs with {#kb tab_switcher::Toggle}. Zed sorts tabs by recent use. Keep holding Ctrl and press Tab to cycle through them.

[Learn more about the Tab Switcher →](./tab-switcher.md)

## Quick Reference {#quick-reference}

| Task              | Keybinding                       |
| ----------------- | -------------------------------- |
| Command Palette   | {#kb command_palette::Toggle}    |
| Open file         | {#kb file_finder::Toggle}        |
| Project search    | {#kb pane::DeploySearch}         |
| Go to definition  | {#kb editor::GoToDefinition}     |
| Find references   | {#kb editor::FindAllReferences}  |
| Symbol in file    | {#kb outline::Toggle}            |
| Symbol in project | {#kb project_symbols::Toggle}    |
| Outline Panel     | {#kb outline_panel::ToggleFocus} |
| Tab Switcher      | {#kb tab_switcher::Toggle}       |

## See Also {#see-also}

- [Command Palette](./command-palette.md) - Full command palette documentation
- [Multibuffers](./multibuffers.md) - Edit multiple files simultaneously
- [Outline Panel](./outline-panel.md) - Symbol tree view
- [Tab Switcher](./tab-switcher.md) - Switch between open files
