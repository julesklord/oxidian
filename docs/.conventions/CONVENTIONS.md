# Zed Documentation Conventions

This guide details Zed documentation structure: content selection, organization, and page creation.

The brand-voice/ directory contains style guidelines:

* SKILL.md: Voice principles and workflow.
* rubric.md: Quality scoring criteria.
* taboo-phrases.md: Patterns to avoid.
* voice-examples.md: Transformation examples.

## Content Selection

### Document
* User-facing features: Interactive elements.
* Configuration: Setting keys, types, defaults, and examples.
* Keybindings: Use {#action ...} and {#kb ...} syntax.
* Actions: Document all actions.
* AI capabilities: Agent tools, providers, and workflows.
* Integrations: LLM providers, MCP servers, and external agents.
* Tools: Agent, MCP, and built-in tools.
* UI: Panels and views.
* Extension APIs: For developers.
* Breaking changes: Document all modifications regardless of complexity.
* Version updates: Use callouts for version-specific behavior (e.g., "In Zed v0.224.0").

### Skip
Skip internal refactors, bug fixes without documentation errors, performance improvements without user impact, and test or CI changes.

## Organization

### New Pages
Create a page for major features with sub-features, extensive configuration examples, high search-volume topics, or new categories.

### New Sections
Add sections to existing pages for settings, keybindings, minor enhancements, or options for existing features.

### Examples

| Change | Action |
| :--- | :--- |
| Git Stash feature | Add section to `git.md` |
| Remote Development | Create `remote-development.md` |
| New git setting | Add to Git config section |
| New AI provider | Add section to `llm-providers.md` |
| New agent tool category | Create page based on scope |

## Document Structure

### Frontmatter
Include YAML frontmatter on every page:

```yaml
---
title: Feature Name - Zed
description: One sentence describing this page.
---
```

Follow these SEO guidelines:
* Assign one primary keyword per page.
* Write unique titles (50-60 characters) that state intent.
* Write descriptions (140-160 characters) summarizing reader actions.
* Use the primary keyword in the title and opening paragraph.
* Use simple single-line entries for compatibility.

### Section Ordering
1. Title (# Feature Name).
2. Opening paragraph: Definition and purpose (1-2 sentences).
3. Usage: Access or enablement instructions.
4. Core functionality: Main features and workflows.
5. Configuration: Settings and JSON examples.
6. Keybindings: Reference tables.
7. See Also: Links to related docs.

### Section Depth
Use ## for main sections and ### for subsections. Avoid deeper levels.

### Anchor IDs
Add explicit IDs like {#getting-started} for common reference targets or stable links.

## Formatting

### Code and Symbols
Use inline `code` for setting names, keybindings, commands, file paths, action names, and values.

### Action and Keybinding References
Use Zed syntax for dynamic rendering:
* {#action git::Commit}: Renders the action name.
* {#kb git::Commit}: Renders the keybinding.

### JSON Examples
Annotate blocks with [settings] or [keymap]:

```json [settings]
{
  "vim_mode": true
}
```

```json [keymap]
{
  "context": "Editor",
  "bindings": {
    "ctrl-s": "workspace::Save"
  }
}
```

### Text Layout
* Tables: Use for references and comparisons. Keep cells concise.
* Paragraphs: Limit to three sentences. Focus on one idea per paragraph.
* Pronouns: Repeat nouns instead of using vague words like "it" or "this".
* Callouts: Use blockquotes for tips, notes, and warnings.
* Version Notes: Specify version numbers for behavioral changes.

## Linking

### Internal and External
* Use relative paths for internal links.
* Link to zed.dev or upstream docs for integrations.
* Include a "See also" section with related links.

### SEO Linking
* Link each page to the documentation tree.
* Include three internal links on non-reference pages.
* Use descriptive link text.
* Link to relevant zed.dev marketing pages for main features.

## Specialized Documentation

### Languages
Follow this structure for language docs in `src/languages/`:
1. Language name and description.
2. Setup instructions.
3. Server configuration.
4. Formatting configuration.
5. Language settings.
6. Limitations.

### Settings
1. Show the Settings Editor (UI) approach first.
2. Provide JSON as a secondary option.
3. Include the setting key in code formatting.
4. Provide a one-sentence description.
5. Show types and defaults.
6. Include a JSON example.

#### File Locations
* macOS/Linux: `~/.config/zed/settings.json` and `keymap.json`.
* Windows: `%AppData%\Zed\settings.json` and `keymap.json`.

## Terminology

| Use | Instead of |
| :--- | :--- |
| folder | directory |
| project | workspace |
| Settings Editor | settings UI |
| command palette | command bar |
| panel | sidebar (e.g., "Project Panel") |

## Requirements
Format all documentation with Prettier at an 80-character line width. Verify changes with `npx prettier --check src/`.

## Quality Checklist
* Frontmatter includes title and description.
* One primary keyword per page.
* Opening paragraph explains the topic.
* Settings show UI options before JSON.
* Actions use {#action ...} and {#kb ...} syntax.
* All actions recorded.
* Anchor IDs added to sections.
* Version callouts included.
* No orphan pages.
* Passes Prettier formatting.
* Follows the brand voice rubric.

## Reference
Consult `docs/AGENTS.md` for automation rules, safety constraints, and output formats.
