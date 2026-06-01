---
title: AI Configuration
description: Manage LLM providers, model settings, and the Agent Panel.
---

# Configuration

Zed users customize LLM providers, model parameters, and Agent Panel behavior in the settings file.

## LLM Providers

Zed connects to three types of language model sources:

- Authenticated users access [Zed's hosted models](./subscription.md).
- [Individual API keys](./llm-providers.md) bypass Zed's hosting infrastructure.
- [External agents](./external-agents.md) like Claude Agent offer alternative workflows.

## Settings

The configuration file defines [model parameters](./agent-settings.md#model-settings) and [panel interactions](./agent-settings.md#agent-panel-settings).

## Disable AI

To deactivate all AI features, add this entry to the settings:

```json [settings]
{
  "disable_ai": true
}
```

[Zed's blog](https://zed.dev/blog/disable-ai-features) provides details on this setting.
