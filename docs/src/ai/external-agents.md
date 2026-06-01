---
title: Use Claude Agent, Gemini CLI, and Codex in Zed
description: Run AI coding agents in Zed via the Agent Client Protocol (ACP).
---

# External Agents

Run agents in Zed using the [Agent Client Protocol (ACP)](https://agentclientprotocol.com). Zed interacts with [Gemini CLI](https://github.com/google-gemini/gemini-cli), [Claude Agent](https://platform.claude.com/docs/en/agent-sdk/overview), [Codex](https://developers.openai.com/codex), and [GitHub Copilot](https://github.com/github/copilot-language-server-release).

Zed provides a UI for these agents. Your billing and legal agreements exist directly with the provider.

## Gemini CLI {#gemini-cli}

Run [Gemini CLI](https://github.com/google-gemini/gemini-cli) in the [agent panel](./agent-panel.md). Zed runs the CLI in the background and communicates via ACP.

### Getting Started

Open the agent panel with {#kb agent::ToggleFocus}. Start a Gemini CLI thread with the agent selector or the `+` button.

Map a shortcut in your `keymap.json` file:

```json [keymap]
[
  {
    "bindings": {
      "cmd-alt-g": ["agent::NewExternalAgentThread", { "agent": "gemini" }]
    }
  }
]
```

#### Installation

Zed installs [@google/gemini-cli](https://github.com/google-gemini/gemini-cli) when you create your first thread. Zed manages updates automatically.

#### Authentication

Log in when prompted. Click "Login" to use your Google account or [Vertex AI](https://cloud.google.com/vertex-ai) credentials. Zed does not see your tokens.

Zed automatically passes the `GEMINI_API_KEY` or `GOOGLE_AI_API_KEY` environment variables if you set them in your system or [provider settings](./llm-providers.md#google-ai).

### Usage

Gemini CLI generates code, refactors, debugs, and answers questions. Add context by mentioning files, threads, or symbols with @.

## Claude Agent

Run [Claude Agent](https://platform.claude.com/docs/en/agent-sdk/overview) in the agent panel. Zed uses the Claude Agent SDK to communicate over ACP.

### Getting Started

Open the agent panel with {#kb agent::ToggleFocus}. Start a Claude Agent thread using the selector or the `+` button.

Map a shortcut in your `keymap.json`:

```json [keymap]
[
  {
    "bindings": {
      "cmd-alt-c": ["agent::NewExternalAgentThread", { "agent": "claude-acp" }]
    }
  }
]
```

### Authentication

Claude Agent authentication happens independently of Zed settings. Run `/login` in a new thread to use an API key or your Claude Pro subscription.

#### Installation

Zed installs [@zed-industries/claude-agent-acp](https://github.com/zed-industries/claude-agent-acp) during your first thread creation. Zed manages updates.

Override the executable by setting `CLAUDE_CODE_EXECUTABLE` in your settings:

```json
{
  "agent_servers": {
    "claude-acp": {
      "type": "registry",
      "env": {
        "CLAUDE_CODE_EXECUTABLE": "/path/to/alternate-claude-code-executable"
      }
    }
  }
}
```

### Usage

Claude Agent handles standard workflows. Mention files, diagnostics, or symbols to add context.

- Slash commands work through merged skills.
- Subagents work.
- Teams and hooks lack support.

#### CLAUDE.md

Claude Agent reads `CLAUDE.md` files in the project root or subdirectories. Use the `init` command to create one.

## Codex CLI

Run [Codex CLI](https://github.com/openai/codex) in the agent panel. Zed communicates via the [Codex adapter](https://github.com/zed-industries/codex-acp).

### Getting Started

Start a Codex thread from the agent panel selector.

Map a shortcut in your `keymap.json`:

```json
[
  {
    "bindings": {
      "cmd-alt-c": ["agent::NewExternalAgentThread", { "agent": "codex-acp" }]
    }
  }
]
```

### Authentication

Codex authentication happens independently of Zed settings. Choose a method in the thread prompt:

1. Login with ChatGPT.
2. `CODEX_API_KEY`.
3. `OPENAI_API_KEY`.

Use `/logout` to change methods. Configure third-party providers in [config.toml](https://github.com/openai/codex/blob/main/docs/config.md#model-selection).

#### Installation

Zed installs [codex-acp](https://github.com/zed-industries/codex-acp) when you start your first thread.

### Usage

Codex performs standard agent tasks. Use @ to add file or symbol context.

## Add More Agents {#add-more-agents}

### ACP Registry

Install agents from the [ACP Registry](https://agentclientprotocol.com/registry). This replaces Agent Server extensions.

Open the registry with {#action zed::AcpRegistry}. Install your preferred agent to see it in the thread menu.

### Custom Agents

Define custom agents in your `settings.json` file:

```json [settings]
{
  "agent_servers": {
    "My Custom Agent": {
      "type": "custom",
      "command": "node",
      "args": ["~/projects/agent/index.js", "--acp"],
      "env": {}
    }
  }
}
```

Customize registry agents by using their names with `"type": "registry"`.

## Debugging Agents

View communication logs with {#action dev::OpenAcpLogs}. Use this data for bug reports.

## Configuration Boundaries {#configuration-boundaries}

Zed communicates with separate agent processes via ACP. This separates Zed settings from agent configuration.

### Data Shared via ACP

Zed sends these settings to external agents:

| Setting | Configuration |
| :--- | :--- |
| Model | `agent_servers.<agent>.default_model` |
| Mode | `agent_servers.<agent>.default_mode` |
| Environment | `agent_servers.<agent>.env` |
| MCP servers | `context_servers` |
| Directory | Project root |

Zed does not forward [profiles](./agent-panel.md#profiles), [tool permissions](./tool-permissions.md), or [rules files](./rules.md).

### Native Agent Configuration {#native-config}

Agents read their own configuration files. Zed does not block this access.

#### Claude Agent

Claude Agent reads:
- `~/.claude/` settings and memory.
- `CLAUDE.md` files.
- Skills.
- Native MCP servers.

#### Codex

Codex reads:
- `~/.codex/config.toml`.
- Native MCP servers.
- `CODEX_API_KEY` and `OPENAI_API_KEY` environment variables.

### MCP Server Access {#mcp-server-access}

Zed forwards `context_servers` to Claude Agent and Codex.

- Local stdio MCP servers work.
- Remote OAuth MCP servers have [known issues](https://github.com/zed-industries/zed/issues/54410).

### Troubleshooting {#troubleshooting}

**Agent lacks MCP tools**
Verify settings in `context_servers`. Check logs via {#action dev::OpenAcpLogs}.

**Existing setup fails in Zed**
Authenticate via `/login` or the thread prompt. Zed handles authentication separately from global installs.

**Profiles do not affect agents**
Profiles only apply to Zed's native agent. External agents use their own tool sets.
