---
title: Code Completions - Zed
description: Zed's code completions from language servers and edit predictions. Configure autocomplete behavior, snippets, and documentation display.
---

# Completions

Zed provides completions through two sources:

1. Language Servers (LSPs) installed automatically or through [Zed Language Extensions](languages.md) generate code completions.
2. The Zeta model or external providers like [GitHub Copilot](#github-copilot) generate edit predictions.

## Language Server Code Completions {#code-completions}

Active language servers suggest variables, functions, and symbols. Disable these suggestions in `settings.json`:

```json [settings]
"show_completions_on_input": false
```

Trigger completions with `ctrl-space` or the `editor::ShowCompletions` command.

MacOS users must disable the "Select the previous input source" shortcut in **System Settings** > **Keyboard** > **Keyboard Shortcuts** > **Input Sources** to use `ctrl-space`.

Reference these guides for more information:

- [Configuring Supported Languages](./configuring-languages.md)
- [List of Zed Supported Languages](./languages.md)

## Edit Predictions {#edit-predictions}

The [Zeta](https://huggingface.co/zed-industries/zeta) model predicts multiple edits at once. Press `tab` to accept these predictions.

The [edit predictions documentation](./ai/edit-prediction.md) contains setup and configuration instructions.
