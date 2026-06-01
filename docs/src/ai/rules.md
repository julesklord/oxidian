# Rules

Rules provide prompts for the [Agent Panel](./agent-panel.md). Zed v1.4.0 replaced on-demand rules with [Skills](./skills.md). Use skills to package reusable instructions.

## `.rules` files

Zed reads project-level instructions from `.rules` files at the root. The Agent Panel includes these instructions in every interaction. Zed checks for these filenames in order:

- `.rules`
- `.cursorrules`
- `.windsurfrules`
- `.clinerules`
- `.github/copilot-instructions.md`
- `AGENT.md`
- `AGENTS.md`
- `CLAUDE.md`
- `GEMINI.md`

## Rules Library

The Rules Library manages legacy rules through an editor with syntax highlighting. Zed v1.4.0 replaced the rules library with [Skills](./skills.md).

### Opening the Rules Library

1. Open the Agent Panel.
2. Click the Agent menu (`...`).
3. Select `Rules...`.

The {#action agent::OpenRulesLibrary} action or {#kb agent::OpenRulesLibrary} keybinding also opens the library.

### Managing Rules

Users edit selected rules in the built-in editor and change titles in the title bar. The editor includes buttons to duplicate, delete, or add rules to the default set.

### Creating Rules

Click the `+` button in the `Rules Library` to create a rule file. Zed stores these files locally.

Refer to provider documentation for prompt engineering guidance:
- [Anthropic: Prompt Engineering](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/overview)
- [OpenAI: Prompt Engineering](https://platform.openai.com/docs/guides/prompt-engineering)

### Using Rules

Mention rules in the Agent Panel using the `@` symbol. This inserts reusable prompts without manual typing.

#### Default Rules

The rule editor includes a paper clip icon to set a rule as default. Zed inserts default rules into the context for all new Agent Panel interactions.

## Migrating to Skills

Zed v1.4.0 automatically migrates existing Rules.

- Zed converts non-default rules into global skills in `~/.agents/skills/` and sets `disable-model-invocation: true`. Users invoke these via `/skill-name` or `@` mentions.
- Zed appends default rules and Git commit prompt customizations to the global `AGENTS.md` file. This file resides in `~/.config/zed/AGENTS.md` on macOS and Linux, or `%APPDATA%\Zed\AGENTS.md` on Windows.

The Rules Library content remains on disk. Downgrading to an earlier version restores access to existing rules.
