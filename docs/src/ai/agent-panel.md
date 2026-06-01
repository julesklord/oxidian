---
title: AI Coding Agent - Zed Agent Panel
description: Generate, refactor, and debug code with tool calling, checkpoints, and multi-model support.
---

# Agent Panel

The Agent Panel facilitates interaction with AI agents that read, write, and execute code within projects. Use this panel for code generation, refactoring, debugging, and documentation.

Open the panel using the `agent::NewThread` action in the Command Palette or by clicking the status bar icon.

## Setup

The Agent Panel requires at least one LLM provider or external agent configuration.

1. **Hosted Models:** Subscribe to the Pro plan for access to Zed's hosted models.
2. **API Keys:** Configure custom keys for providers (e.g., Anthropic, OpenAI) or gateways (e.g., OpenRouter).
3. **External Agents:** Integrate CLI tools such as Gemini CLI or Claude Agent.

## Operations

Responses display indicators for active tool usage. External agents may not support all features, such as thread restoration, checkpoints, or token usage displays.

### Threads

The Agent Panel uses Zed's first-party agent by default.

- **New Thread:** Create a thread via the `agent::NewThread` action or the panel toolbar.
- **New From Summary:** Start a fresh thread seeded with a summary of the current conversation to manage context limits.
- **Terminal:** Open a shell session directly within the panel.
- **External Agent:** Use the `agent::NewExternalAgentThread` action to specify an external agent ID.

### Management

Run multiple independent threads simultaneously. The Threads Sidebar groups threads by project. Switch between recent threads using the thread switcher or archive inactive ones.

Isolate threads in Git worktrees via the title bar picker to prevent conflicting file edits.

### Interaction

- **Editing:** Modify and resubmit sent messages to adjust prompts or context.
- **Queueing:** The system queues messages sent during generation. Zed Agent sends queued messages at the next turn boundary; external agents send them after generation completes.
- **Checkpoints:** Restore the codebase to its state prior to specific messages using the "Restore Checkpoint" button.
- **Context Menu:** Right-click responses to copy selection, copy full text, scroll, or open threads as Markdown.

### Navigation

Scroll long conversations using arrow keys or panel buttons. Thread titles generate automatically based on content and support manual editing.

Follow agent activity by clicking the crosshair icon; the editor jumps to files as the agent accesses them. Submit messages with `cmd`/`ctrl` to enable this automatically.

### Notifications

Zed provides visual and audio notifications when background generations complete. Configure these via `agent.notify_when_agent_waiting` and `agent.play_sound_when_agent_done`.

## Reviewing Changes

The panel displays edited files and line counts. Use the "Review Changes" button to open a multi-buffer tab for accepting or rejecting individual hunks. Inline diffs are also available via the `agent.single_file_review` setting.

## Context

Add context by typing `@` to mention files, directories, symbols, previous threads, or diagnostics. Pasting code selections automatically formats them as @-mentions. Use the `agent::PasteRaw` action for plain text.

vision-capable models support image attachments via project search, drag-and-drop, or clipboard.

## Tools

The Agent Panel supports tool calling for agentic editing. Built-in tools handle codebase search, file modification, and terminal execution. MCP Servers provide additional tool extensions.

### Profiles

Group tools using profiles:
- **Write:** Enables file modification and terminal commands.
- **Ask:** Read-only tools for queries.
- **Minimal:** No tool access for general conversations.

Manage profiles via the Agent Profile modal or command palette.

### Permissions

Control tool approval through `agent.tool_permissions.default`:
- `confirm`: Prompts for every action.
- `allow`: Auto-approves actions.
- `deny`: Blocks actions.

Define per-tool defaults or pattern-based rules within the confirmation menu.

## Feedback

Rate responses using the thumbs up/down icons. Rating sends data to Zed's servers for system improvement.
