---
title: AI Code Completion in Zed - Zeta, Copilot, Codestral, Mercury Coder
description: Set up AI code completions in Zed with Zeta (built-in), GitHub Copilot, Codestral, or Mercury Coder. Multi-line predictions on every keystroke.
---

# Edit Prediction

Zed's AI code completions use an LLM to predict code as you type. Every keystroke triggers a request to the prediction provider. Accept multi-line suggestions by pressing `tab`.

The default provider is Zeta. Zed also supports GitHub Copilot, Mercury Coder, and Codestral.

## Configuring Zeta

Sign in to use Zeta. Open the Settings Editor (`Cmd+,` or `Ctrl+,`) and search for `edit_predictions` to verify the provider is `Zed AI`.

Check your settings.json:

```json [settings]
{
  "edit_predictions": {
    "provider": "zed"
  }
}
```

The Z icon in the status bar indicates Zeta is active.

### Pricing and Plans

Free plans include 2,000 Zeta predictions per month. The Pro plan removes this limit. See the Zed pricing page for details.

### Switching Modes {#switching-modes}

Edit Prediction has two display modes:

1. `eager` (default): Zed displays predictions inline unless they conflict with language server completions.
2. `subtle`: Predictions appear inline only while you hold a modifier key (`alt` by default).

Toggle modes via the status bar menu or the `mode` key:

```json [settings]
"edit_predictions": {
  "mode": "eager" // or "subtle"
},
```

## Default Key Bindings

Users on macOS and Windows accept predictions with `alt-tab`. Linux uses `alt-l` by default to avoid window manager conflicts.

In `eager` mode, `tab` accepts predictions unless the completion menu is open. If the menu is visible, `tab` accepts LSP completions. Dismiss a prediction with {#kb editor::Cancel} to insert whitespace with `tab`.

{#action editor::AcceptNextWordEditPrediction} ({#kb editor::AcceptNextWordEditPrediction}) accepts the prediction up to the next word boundary.
{#action editor::AcceptNextLineEditPrediction} ({#kb editor::AcceptNextLineEditPrediction}) accepts the prediction up to the next line.

## Configuring Edit Prediction Keybindings {#edit-predictions-keybinding}

### Always Use Tab

To prioritize edit predictions over the LSP completions menu, add this to `keymap.json`:

```json [keymap]
[
  {
    "context": "Editor && edit_prediction",
    "bindings": {
      "tab": "editor::AcceptEditPrediction"
    }
  }
]
```

Use {#kb editor::ComposeCompletion} to accept LSP completions instead.

### Always Use Alt-Tab

To stop `tab` from accepting predictions, unbind it in the eager edit prediction context:

```json [keymap]
[
  {
    "context": "Editor && edit_prediction",
    "unbind": {
      "tab": "editor::AcceptEditPrediction"
    }
  }
]
```

`alt-tab` and `alt-l` remain active for accepting predictions.

### Rebind Both Tab and Alt-Tab

Unbind the defaults and add a new binding:

```json [keymap]
[
  {
    "context": "Editor && edit_prediction",
    "unbind": {
      "alt-tab": "editor::AcceptEditPrediction",
      "tab": "editor::AcceptEditPrediction"
    },
    "bindings": {
      "ctrl-enter": "editor::AcceptEditPrediction"
    }
  }
]
```

### Cleaning Up Older Keymap Entries

If you configured bindings before version `v0.229.0`, remove redundant entries. Use the `unbind` key rather than copying default non-prediction bindings. Zed automatically migrates the `edit_prediction_conflict` context to `edit_prediction && (showing_completions || in_leading_whitespace)`.

## Disabling Automatic Edit Prediction

Disable predictions globally or for specific files. Use Subtle Mode to reduce visual noise.

### On Buffers

Hide all prediction indicators by updating your settings:

```json [settings]
{
  "show_edit_predictions": false
}
```

Trigger predictions manually with {#action editor::ShowEditPrediction} or {#kb editor::ShowEditPrediction}.

### For Specific Languages

Disable predictions for a single language:

```json [settings]
{
  "languages": {
    "Python": {
      "show_edit_predictions": false
    }
  }
}
```

### In Specific Directories

Specify files or paths to ignore:

```json [settings]
{
  "edit_predictions": {
    "disabled_globs": ["~/.config/zed/settings.json"]
  }
}
```

### Turning Off Completely

Disable edit prediction across all providers:

```json [settings]
{
  "edit_predictions": {
    "provider": "none"
  }
}
```

## Configuring Other Providers {#other-providers}

### GitHub Copilot {#github-copilot}

Set the provider to `copilot` in your settings:

```json [settings]
{
  "edit_predictions": {
    "provider": "copilot"
  }
}
```

Click the Copilot icon in the status bar to sign in. Copy the device code and follow the GitHub verification link.

#### Using GitHub Copilot Enterprise

Enter your enterprise instance URL in settings:

```json [settings]
{
  "edit_predictions": {
    "copilot": {
      "enterprise_uri": "https://your.enterprise.domain"
    }
  }
}
```

Zed routes requests through your enterprise endpoint and redirects authentication to that URL.

Navigate multiple Copilot alternatives:

- {#action editor::NextEditPrediction} ({#kb editor::NextEditPrediction}): Cycle to the next prediction.
- {#action editor::PreviousEditPrediction} ({#kb editor::PreviousEditPrediction}): Cycle to the previous prediction.

### Mercury Coder {#mercury-coder}

Enter your API key in the Mercury section of the Configure Providers menu. Select it from the status bar or update your settings:

```json [settings]
{
  "edit_predictions": {
    "provider": "mercury"
  }
}
```

### Codestral {#codestral}

Add your API key from the Codestral dashboard to the Configure Providers menu. Update your settings:

```json [settings]
{
  "edit_predictions": {
    "provider": "codestral"
  }
}
```

### Local and self-hosted models

Zed supports local and self-hosted models through Ollama or any server using the OpenAI completion API.

#### Ollama

Configure the local model and URL:

```json [settings]
{
  "edit_predictions": {
    "provider": "ollama",
    "ollama": {
      "api_url": "http://localhost:11434",
      "model": "qwen2.5-coder:7b-base",
      "prompt_format": "infer",
      "max_output_tokens": 512
    }
  }
}
```

#### OpenAI-compatible servers

Set the API endpoint and model:

```json [settings]
{
  "edit_predictions": {
    "provider": "open_ai_compatible_api",
    "open_ai_compatible_api": {
      "api_url": "http://localhost:8080/v1/completions",
      "model": "deepseek-coder-6.7b-base",
      "prompt_format": "deepseek_coder",
      "max_output_tokens": 512
    }
  }
}
```

The `prompt_format` setting dictates code context formatting. Use `"infer"` for automatic detection based on the model name or select a format:

- `zeta`
- `zeta2`
- `zeta2_1`
- `code_llama`
- `star_coder`
- `deepseek_coder`
- `qwen`
- `code_gemma`
- `codestral`
- `glm`
- `infer`

Zed uses Zeta 2 format for models named `zeta2` and Zeta 2.1 for `zeta2.1` when using `infer`.

The server must implement the OpenAI `/v1/completions` endpoint. Edit predictions send POST requests containing the model name, prompt, token limits, and temperature.

## See also

- [Agent Panel](./agent-panel.md): Agentic editing with file access and terminal integration.
- [Inline Assistant](./inline-assistant.md): Prompt-driven transformations on selected code.
