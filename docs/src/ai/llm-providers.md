# LLM Providers

Zed requires at least one configured language model provider for AI features. Configured providers work in the Agent Panel and Inline Assistant. 

Subscribe to a Zed plan or use existing API keys for supported providers. Use the Configuration guide for general setup.

## Use Your Own Keys

Add existing API keys for Anthropic, OpenAI, or other supported providers without a Zed subscription. 

Enter the key in the Agent Panel settings. Zed stores API keys in the OS secure credential storage instead of the settings file.

## Supported Providers

Zed supports these providers with individual API keys:

- Amazon Bedrock
- Anthropic
- ChatGPT Subscription
- DeepSeek
- GitHub Copilot Chat
- Google AI
- LM Studio
- Mistral
- Ollama
- OpenAI
- OpenAI API Compatible
- OpenCode
- OpenRouter
- Vercel AI Gateway
- xAI

### Amazon Bedrock

Amazon Bedrock supports tool use for models capable of streaming tools. Refer to the AWS documentation for compatibility details.

Zed requires AWS authentication. Assign these permissions to your credentials:

- `bedrock:InvokeModelWithResponseStream`
- `bedrock:InvokeModel`

Example IAM policy:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "bedrock:InvokeModel",
        "bedrock:InvokeModelWithResponseStream"
      ],
      "Resource": "*"
    }
  ]
}
```

Configure authentication using one of these three methods:

#### Authentication via Named Profile

Install and configure the AWS CLI with a named profile. Update `settings.json` to include the `bedrock` key under `language_models`.

```json [settings]
{
  "language_models": {
    "bedrock": {
      "authentication_method": "named_profile",
      "region": "your-aws-region",
      "profile": "your-profile-name"
    }
  }
}
```

#### Authentication via Static Credentials

Enter the AWS access key and secret in the Agent Panel settings UI. Named profiles offer better security. 

1. Create an IAM User in the IAM Console.
2. Generate and save security credentials.
3. Enter the Access Key ID, Secret Access Key, and Region in the Amazon Bedrock section of the Agent Configuration.

#### Authentication via Bedrock API Key

Amazon Bedrock supports direct API key authentication.

1. Create an API Key in the Amazon Bedrock Console.
2. Enter the key and Region in the Amazon Bedrock section of the Agent Configuration.

```json [settings]
{
  "language_models": {
    "bedrock": {
      "authentication_method": "api_key",
      "region": "your-aws-region"
    }
  }
}
```

The OS keychain stores the API key.

#### Cross-Region Inference

Zed uses Cross-Region inference to improve availability. This distributes traffic across multiple AWS Regions for higher throughput.

##### Regional vs Global Inference Profiles

Bedrock supports two cross-region inference profile types:

- Regional profiles: Zed routes requests within a geography. The `us-east-1` region uses the `us.*` profile to route across `us-east-1`, `us-east-2`, and `us-west-2`.
- Global profiles: Zed routes requests across all commercial AWS Regions for maximum availability.

Zed defaults to regional profiles to keep data within the same geography. Opt into global profiles by adding `"allow_global": true` to the configuration.

```json [settings]
{
  "language_models": {
    "bedrock": {
      "authentication_method": "named_profile",
      "region": "your-aws-region",
      "profile": "your-profile-name",
      "allow_global": true
    }
  }
}
```

Select models support global inference profiles. Refer to AWS documentation for the current list. Enabling `allow_global` may resolve regional availability issues.

AWS stores data in the source Region. Input prompts and output results might move outside the source Region during cross-Region inference. Encryption protects data during transmission.

Zed supports Cross-Region inference for specific models. Refer to the Bedrock crate source code for the implementation list. The AWS documentation lists currently supported regions and models.

#### Image Support

Bedrock models with vision capabilities process images in conversations and tool results. This includes Claude 3, Amazon Nova, Meta Llama 3.2 Vision, and Mistral Pixtral.

#### Guardrails

Apply a guardrail to Bedrock requests by adding `guardrail_identifier` to the configuration.

```json [settings]
{
  "language_models": {
    "bedrock": {
      "guardrail_identifier": "arn:aws:bedrock:us-east-1:123456789012:guardrail/abc123",
      "guardrail_version": "DRAFT"
    }
  }
}
```

The `guardrail_version` defaults to `"DRAFT"` if omitted.

### Anthropic

Select Anthropic models from the dropdown in the Agent Panel.

1. Sign up for Anthropic and generate an API key.
2. Add credits to the Anthropic account.
3. Enter the key in the Anthropic section of the settings view.

API usage requires credits even with a Claude Pro subscription. Zed recognizes the `ANTHROPIC_API_KEY` environment variable.

#### Custom Models

Add custom models to the Anthropic provider in the Zed settings file.

```json [settings]
{
  "language_models": {
    "anthropic": {
      "available_models": [
        {
          "name": "claude-3-5-sonnet-20240620",
          "display_name": "Sonnet 2024-June",
          "max_tokens": 128000,
          "max_output_tokens": 2560,
          "tool_override": "some-model-that-supports-toolcalling"
        }
      ]
    }
  }
}
```

Enable extended thinking for compatible models by setting the mode to `thinking`.

```json
{
  "name": "claude-sonnet-4-latest",
  "display_name": "claude-sonnet-4-thinking",
  "max_tokens": 200000,
  "mode": {
    "type": "thinking",
    "budget_tokens": 4096
  }
}
```

### ChatGPT Subscription

Access OpenAI models via an existing ChatGPT Plus or Pro subscription. This method requires no API key.

1. Sign in via the ChatGPT Subscription section in settings.
2. Authenticate through the browser.
3. Select models from the dropdown.

Model availability depends on the subscription tier. 

### DeepSeek

1. Generate an API key on the DeepSeek platform.
2. Enter the key in the DeepSeek section of the settings view.

The keychain stores the API key. Zed recognizes the `DEEPSEEK_API_KEY` environment variable.

#### Custom Models

Zed includes pre-configured settings for DeepSeek V4 Flash and Pro. Customize the API endpoint or add models in the settings file.

```json [settings]
{
  "language_models": {
    "deepseek": {
      "api_url": "https://api.deepseek.com",
      "available_models": [
        {
          "name": "deepseek-v4-flash",
          "display_name": "DeepSeek V4 Flash",
          "max_tokens": 1000000,
          "max_output_tokens": 384000
        },
        {
          "name": "deepseek-v4-pro",
          "display_name": "DeepSeek V4 Pro",
          "max_tokens": 1000000,
          "max_output_tokens": 384000
        }
      ]
    }
  }
}
```

### GitHub Copilot Chat

Select GitHub Copilot Chat from the model dropdown in the Agent Panel.

1. Sign in through the GitHub Copilot Chat section in settings.
2. Follow the modal instructions.

Zed also accepts an OAuth token via the `GH_COPILOT_TOKEN` environment variable. Enable specific models in GitHub settings if they do not appear in the dropdown. 

Configure Copilot Enterprise endpoints as described in the edit prediction documentation.

### Google AI

Select Gemini models from the dropdown in the Agent Panel.

1. Generate an API key in Google AI Studio.
2. Enter the key in the Google AI section of settings.

The keychain stores the API key. Zed recognizes the `GEMINI_API_KEY` environment variable.

#### Custom Models

Zed defaults to stable model versions. Specify specific versions or experimental models in the configuration. Use the `mode` setting to enable thinking mode and control reasoning token usage.

```json [settings]
{
  "language_models": {
    "google": {
      "available_models": [
        {
          "name": "gemini-3.1-pro-preview",
          "display_name": "Gemini 3.1 Pro",
          "max_tokens": 1000000,
          "mode": {
            "type": "thinking",
            "budget_tokens": 24000
          }
        },
        {
          "name": "gemini-3-flash-preview",
          "display_name": "Gemini 3 Flash (Thinking)",
          "max_tokens": 1000000,
          "mode": {
            "type": "thinking",
            "budget_tokens": 24000
          }
        }
      ]
    }
  }
}
```

### LM Studio

1. Install LM Studio.
2. Download a model using `cmd/ctrl-shift-m` or the CLI: `lms get qwen2.5-coder-7b`.
3. Start the server: `lms server start`.

Set LM Studio as a login item to automate server startup.

### Mistral

1. Generate an API key on the Mistral platform.
2. Enter the key in the Mistral section of settings.

The keychain stores the API key. Zed recognizes the `MISTRAL_API_KEY` environment variable.

#### Custom Models

Zed includes configurations for the latest Mistral models. Customize parameters or add models in the settings file.

```json [settings]
{
  "language_models": {
    "mistral": {
      "api_url": "https://api.mistral.ai/v1",
      "available_models": [
        {
          "name": "mistral-tiny-latest",
          "display_name": "Mistral Tiny",
          "max_tokens": 32000,
          "max_output_tokens": 4096,
          "max_completion_tokens": 1024,
          "supports_tools": true,
          "supports_images": false
        }
      ]
    }
  }
}
```

### Ollama

Install Ollama and confirm it runs using `ollama --version`.

1. Pull a model: `ollama pull mistral`.
2. Start the server via the app or `ollama serve`.
3. Select an Ollama model from the Agent Panel dropdown.

#### Ollama Autodiscovery

Zed discovers locally pulled Ollama models automatically. Disable this by setting `auto_discover` to `false` and specifying models manually.

```json [settings]
{
  "language_models": {
    "ollama": {
      "api_url": "http://localhost:11434",
      "auto_discover": false,
      "available_models": [
        {
          "name": "qwen2.5-coder",
          "display_name": "qwen 2.5 coder",
          "max_tokens": 32768,
          "supports_tools": true,
          "supports_thinking": true,
          "supports_images": true
        }
      ]
    }
  }
}
```

#### Ollama Context Length

Zed sends the context length to Ollama via the `num_ctx` parameter. The default length is 4096 tokens. Token counts in the Agent Panel are estimates.

Set a global context length using `context_window` or a per-model length using `max_tokens`.

```json [settings]
{
  "language_models": {
    "ollama": {
      "context_window": 8192
    }
  }
}
```

The `context_window` setting overrides per-model `max_tokens` values. Hardware memory limits may require smaller values. Monitor logs via `tail -f ~/.ollama/logs/ollama.log` or `journalctl -u ollama -f`.

The `keep_alive` setting controls how long the remote server retains the model in VRAM. Use an integer for seconds or a duration string like "5m".

Use `supports_tools` for models tagged with tools in the Ollama catalog. This enables the Ask and Write profiles. Minimal profiles work for models without tool tags.

Set `supports_thinking` for models that perform reasoning passes. Use `supports_images` for models with vision capabilities.

#### Ollama Authentication

Zed connects to local or remote Ollama instances. Remote instances require API keys.

Configure Ollama Turbo:

1. Sign in to Ollama and subscribe to Turbo.
2. Generate an API key in the Ollama settings.
3. Enter the key in the Ollama section of Zed settings.
4. Set the API URL to `https://ollama.com`.

Zed recognize the `OLLAMA_API_KEY` environment variable.

### OpenAI

1. Generate an API key on the OpenAI platform.
2. Add credits to the OpenAI account.
3. Enter the key in the OpenAI section of settings.

The keychain stores the API key. Zed recognizes the `OPENAI_API_KEY` environment variable.

#### Custom Models

Zed includes configurations for current OpenAI models. Add preview releases or control parameters in the settings file.

```json [settings]
{
  "language_models": {
    "openai": {
      "available_models": [
        {
          "name": "gpt-5.2",
          "display_name": "gpt-5.2 high",
          "reasoning_effort": "high",
          "max_tokens": 272000,
          "max_completion_tokens": 20000
        },
        {
          "name": "gpt-5-nano",
          "display_name": "GPT-5 Nano",
          "max_tokens": 400000
        },
        {
          "name": "gpt-5.2-codex",
          "display_name": "GPT-5.2 Codex",
          "max_tokens": 128000,
          "capabilities": {
            "chat_completions": false
          }
        }
      ]
    }
  }
}
```

Specify the context window size using the `max_tokens` parameter. Refer to OpenAI documentation for values. Set `max_completion_tokens` for reasoning models to limit costs. 

Disable `chat_completions` for models that require the Responses endpoint.

### OpenAI API Compatible

Connect to hosted services like Together AI or local models using OpenAI-compatible APIs. Specify a custom `api_url` and `available_models`.

Add models via the UI modal or the settings file.

```json [settings]
{
  "language_models": {
    "openai_compatible": {
      "Together AI": {
        "api_url": "https://api.together.xyz/v1",
        "available_models": [
          {
            "name": "mistralai/Mixtral-8x7B-Instruct-v0.1",
            "display_name": "Together Mixtral 8x7B",
            "max_tokens": 32768,
            "capabilities": {
              "tools": true,
              "images": false,
              "parallel_tool_calls": false,
              "prompt_cache_key": false
            }
          }
        ]
      }
    }
  }
}
```

Default capabilities:
- `tools`: true
- `images`: false
- `parallel_tool_calls`: false
- `prompt_cache_key`: false
- `chat_completions`: true
- `interleaved_reasoning`: false

Set `interleaved_reasoning` to `true` for models that expect a dedicated `reasoning_content` field. Disable `chat_completions` for models using the Responses API.

Set the API key in environment variables using the `<PROVIDER_NAME>_API_KEY` format.

### OpenCode

OpenCode provides model access through Zen, Zen Free, or Go subscriptions.

1. Create an account in the OpenCode Console.
2. Generate an API key.
3. Enter the key in the OpenCode section of settings.

The keychain stores the API key. Zed recognizes the `OPENCODE_API_KEY` environment variable.

Hide irrelevant subscriptions in the UI or settings.

```json [settings]
{
  "language_models": {
    "opencode": {
      "show_zen_models": true,
      "show_go_models": false,
      "show_free_models": false
    }
  }
}
```

Zed includes configurations for long-term OpenCode Free models. Configure temporary free models as custom models using parameters from the OpenCode website.

#### Custom Models

Add newer or custom endpoint models in the settings file.

```json [settings]
{
  "language_models": {
    "opencode": {
      "available_models": [
        {
          "name": "my-custom-model",
          "display_name": "My Custom Model",
          "max_tokens": 123456,
          "max_output_tokens": 98765,
          "protocol": "openai_chat",
          "reasoning_effort_levels": ["low", "medium", "high"],
          "interleaved_reasoning": false,
          "subscription": "go",
          "custom_model_api_url": "https://example.com/zen"
        }
      ]
    }
  }
}
```

Configuration requirements:
- `name`: OpenCode model id.
- `max_tokens`: Context window size.
- `protocol`: API protocol (`anthropic`, `openai_responses`, `openai_chat`, or `google`).

### OpenRouter

OpenRouter offers access to various models through one API. 

1. Create an OpenRouter account.
2. Generate an API key.
3. Enter the key in the OpenRouter section of settings.

The keychain stores the API key. Zed recognizes the `OPENROUTER_API_KEY` environment variable. Specify a default model in `settings.json`.

```json [settings]
{
  "agent": {
    "default_model": {
      "provider": "openrouter",
      "model": "openrouter/auto"
    }
  }
}
```

The `openrouter/auto` model routes requests to available models automatically.

#### Custom Models

Add custom models to the OpenRouter provider in the settings file.

```json [settings]
{
  "language_models": {
    "open_router": {
      "api_url": "https://openrouter.ai/api/v1",
      "available_models": [
        {
          "name": "google/gemini-2.0-flash-thinking-exp",
          "display_name": "Gemini 2.0 Flash (Thinking)",
          "max_tokens": 200000,
          "max_output_tokens": 8192,
          "supports_tools": true,
          "supports_images": true,
          "mode": {
            "type": "thinking",
            "budget_tokens": 8000
          }
        }
      ]
    }
  }
}
```

Refer to the OpenRouter models page for identifiers and specifications.

#### Provider Routing

Control request routing among upstream providers via the `provider` object.

```json [settings]
{
  "language_models": {
    "open_router": {
      "api_url": "https://openrouter.ai/api/v1",
      "available_models": [
        {
          "name": "openrouter/auto",
          "display_name": "Auto Router (Tools Preferred)",
          "max_tokens": 2000000,
          "supports_tools": true,
          "provider": {
            "order": ["anthropic", "openai"],
            "allow_fallbacks": true,
            "require_parameters": true,
            "only": ["anthropic", "openai", "google"],
            "ignore": ["cohere"],
            "quantizations": ["int8"],
            "sort": "price",
            "data_collection": "allow"
          }
        }
      ]
    }
  }
}
```

These controls adjust cost and reliability without changing the UI selection.

### Vercel AI Gateway

Vercel AI Gateway provides access to models through an OpenAI-compatible endpoint.

1. Generate an API key on the Vercel AI Gateway page.
2. Enter the key in the Vercel AI Gateway section of settings.

The keychain stores the API key. Zed recognizes the `VERCEL_AI_GATEWAY_API_KEY` environment variable. Set a custom endpoint in the settings file if needed.

```json [settings]
{
  "language_models": {
    "vercel_ai_gateway": {
      "api_url": "https://ai-gateway.vercel.sh/v1"
    }
  }
}
```

### xAI

Access Grok models using the dedicated xAI provider.

1. Generate an API key in the xAI Console.
2. Enter the key in the xAI section of settings.

The keychain stores the API key. Zed recognizes the `XAI_API_KEY` environment variable.

#### Custom Models

Zed includes configurations for Grok models. Customize parameters or add models in the settings file.

```json [settings]
{
  "language_models": {
    "x_ai": {
      "api_url": "https://api.x.ai/v1",
      "available_models": [
        {
          "name": "grok-1.5",
          "display_name": "Grok 1.5",
          "max_tokens": 131072,
          "max_output_tokens": 8192
        },
        {
          "name": "grok-1.5v",
          "display_name": "Grok 1.5V (Vision)",
          "max_tokens": 131072,
          "max_output_tokens": 8192,
          "supports_images": true
        }
      ]
    }
  }
}
```

## Custom Provider Endpoints

Use custom API endpoints for Anthropic, Google, Ollama, and OpenAI providers. Add the `api_url` to the settings file.

```json
{
  "language_models": {
    "some-provider": {
      "api_url": "http://localhost:11434"
    }
  }
}
```

This infrastructure also supports OpenAI-compatible models.
