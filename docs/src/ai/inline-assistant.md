---
title: Inline AI Code Editing - Zed
description: Transform code inline with AI in Zed. Send selections to any LLM for refactoring, generation, or editing with multi-cursor support.
---

# Inline Assistant

Use {#kb assistant::InlineAssist} to open the Inline Assistant in editors, the rules library, channel notes, and the terminal panel. The tool sends selections or lines to a language model and replaces them with the response.

## Getting Started

New users must configure an LLM provider or external agent. Access methods include:

1. Subscribing to the Pro plan for hosted models.
2. Inputting API keys from providers like Anthropic or gateways like OpenRouter.

Existing LLM configurations for the Agent Panel apply to the Inline Assistant. External agents do not yet support generating changes within the Inline Assistant.

## Adding Context

Add context using the same methods as the Agent Panel:

- @-mention files, directories, threads, rules, and symbols.
- Paste images from the clipboard.

Referencing an Agent Panel thread via `@thread` allows for targeted refinements without repeating context.

## Parallel Generations

The Inline Assistant generates multiple changes simultaneously.

### Multiple Cursors

Pressing {#kb assistant::InlineAssist} with multiple cursors sends the prompt to each cursor position. Changes generate in parallel. This functionality integrates with multibuffer excerpts.

### Multiple Models

Send one prompt to multiple models simultaneously. Modify the settings file to enable this functionality:

```json [settings]
{
  "agent": {
    "default_model": {
      "provider": "zed.dev",
      "model": "claude-sonnet-4-5"
    },
    "inline_alternatives": [
      {
        "provider": "zed.dev",
        "model": "gpt-4-mini"
      }
    ]
  }
}
```

The UI displays buttons to cycle between model outputs. Specified alternatives run alongside the default model.

The following configuration generates three outputs using the default model and two alternatives:

```json [settings]
{
  "agent": {
    "default_model": {
      "provider": "zed.dev",
      "model": "claude-sonnet-4-5"
    },
    "inline_alternatives": [
      {
        "provider": "zed.dev",
        "model": "gpt-4-mini"
      },
      {
        "provider": "zed.dev",
        "model": "gemini-3-flash"
      }
    ]
  }
}
```

## Inline Assistant vs. Edit Prediction

Both features generate inline code. Their workflows differ.

- Inline Assistant: Manual prompts and selections define the transformation. The user controls context.
- Edit Prediction: Zed suggests edits based on recent changes and cursor position without prompts.

Inline Assistant requires explicit prompts. Edit Prediction operates automatically through context inference.

## Prefilling Prompts

Add a custom keybinding to prefill prompts in the keymap:

```json [keymap]
[
  {
    "context": "Editor && mode == full",
    "bindings": {
      "ctrl-shift-enter": [
        "assistant::InlineAssist",
        { "prompt": "Build a snake game" }
      ]
    }
  }
]
```
