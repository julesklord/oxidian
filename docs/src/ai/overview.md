---
title: AI Code Editor Documentation - Zed
description: AI integration in Zed. Supports autonomous agents, inline edits, and code completion.
---

# AI in Zed

Zed integrates AI into a native Rust application to provide autonomous agents, inline edits, and code completion.

## Technical Architecture

Zed executes AI features within a GPU-accelerated environment.

- **Open Source:** Zed provides the source code for the editor and AI implementation.
- **Model Support:** Connects to Anthropic, OpenAI, Google, and Ollama via cloud or local APIs.
- **External Agents:** The Agent Client Protocol enables integration with CLI tools like Gemini CLI and Claude Agent.
- **Privacy:** Zed shares data only when users opt in. Zero-data retention agreements govern API key usage.

## Agent Workflows

The Threads Sidebar manages agent tasks for reading, modifying, and executing code. Users can run multiple agent and terminal threads simultaneously across projects.

The Agent Panel controls prompts, code reviews, and context. Model Context Protocol (MCP) servers and tool permissions define agent capabilities.

The Inline Assistant transforms code by rewriting selections or terminal commands based on user descriptions.

## Code Completions

Edit Prediction provides real-time code suggestions. The editor inserts suggestions from configured providers when the user presses tab.

Zeta, an open-source model developed by Zed, serves as the default provider. Users can also configure GitHub Copilot or Codestral.

## Resources
- [Configuration](./configuration.md): Set up LLM providers.
- [Parallel Agents](./parallel-agents.md): Manage multiple threads.
- [External Agents](./external-agents.md): Integrate CLI-based agents.
- [Subscription](./subscription.md): Hosted model details.
- [Privacy and Security](./privacy-and-security.md): Data handling policies.
