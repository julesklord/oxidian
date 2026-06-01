# Model Context Protocol

Zed uses the Model Context Protocol (MCP) to interact with context servers.

> The Model Context Protocol (MCP) is an open protocol for connecting LLM applications to external tools and data sources through a standard interface.

## Supported Features

Zed supports MCP Tools and Prompts. Contribute to the Zed repository to advance coverage for Discovery, Sampling, and Elicitation.

Zed handles `notifications/tools/list_changed` notifications from MCP servers. When a server modifies available tools, Zed reloads the tool list without a restart.

## Installing MCP Servers

### As Extensions

Expose MCP servers as extensions. The MCP Server Extensions page explains how to create them.

Find MCP servers in the extension library:

1. [Zed website](https://zed.dev/extensions?filter=context-servers)
2. Open the Command Palette and run {#action zed::Extensions}
3. Select "View Server Extensions" in the Agent Panel menu

Available extensions include:

- [Context7](https://zed.dev/extensions/mcp-server-context7)
- [GitHub](https://zed.dev/extensions/mcp-server-github)
- [Puppeteer](https://zed.dev/extensions/mcp-server-puppeteer)
- [Gem](https://zed.dev/extensions/gem)
- [Brave Search](https://zed.dev/extensions/mcp-server-brave-search)
- [Prisma](https://github.com/aqrln/prisma-mcp-zed)
- [Framelink Figma](https://zed.dev/extensions/mcp-server-figma)
- [Resend](https://zed.dev/extensions/mcp-server-resend)

### As Custom Servers

Connect MCP servers by adding commands to the settings file ([edit instructions](../configuring-zed.md#settings-files)):

```json [settings]
{
  "context_servers": {
    "local-mcp-server": {
      "command": "some-command",
      "args": ["arg-1", "arg-2"],
      "env": {}
    },
    "remote-mcp-server": {
      "url": "custom",
      "headers": { "Authorization": "Bearer <token>" }
    },
    "remote-mcp-server-with-oauth": {
      "url": "https://mcp.example.com/mcp"
    }
  }
}
```

Alternatively, add a custom server through the Agent Panel Settings ({#action agent::OpenSettings}). Click "Add Custom Server" in the configuration modal.

Zed prompts for authentication via the standard MCP OAuth flow if a remote server lacks an "Authorization" header.

## Using MCP Servers

### Configuration Check

MCP servers usually require configuration after installation.

Zed displays a setup modal after extension installation. The GitHub extension, for example, requires a Personal Access Token.

Check provider documentation for custom server commands, arguments, and environment variables.

Monitor server status in the Agent Panel settings. A green indicator and "Server is active" tooltip signal correct configuration. Other colors indicate connection or runtime issues.

### Agent Panel Usage

Open the Agent Panel to start prompting after installation.

Tool invocation reliability depends on the model. Use the server name in prompts to guide tool selection.

Create a custom profile to ensure the agent uses specific MCP tools. Disable built-in tools and enable the desired MCP server tools.

The Dagger team suggests this configuration for the [Container Use server](https://zed.dev/extensions/mcp-server-container-use):

```json [settings]
"agent": {
  "profiles": {
    "container-use": {
      "name": "Container Use",
      "tools": {
        "fetch": true,
        "thinking": true,
        "copy_path": false,
        "find_path": false,
        "delete_path": false,
        "create_directory": false,
        "list_directory": false,
        "diagnostics": false,
        "read_file": false,
        "move_path": false,
        "grep": false,
        "edit_file": false,
        "terminal": false
      },
      "enable_all_context_servers": false,
      "context_servers": {
        "container-use": {
          "tools": {
            "environment_create": true,
            "environment_add_service": true,
            "environment_update": true,
            "environment_run_cmd": true,
            "environment_open": true,
            "environment_file_write": true,
            "environment_file_read": true,
            "environment_file_list": true,
            "environment_file_delete": true,
            "environment_checkpoint": true
          }
        }
      }
    }
  }
}
```

### Tool Permissions

Zed v0.224.0 and above use `agent.tool_permissions.default` for tool approval. Earlier versions used `agent.always_allow_tool_actions` (default `false`).

The `agent.tool_permissions.default` setting defines the approval behavior:

- `"confirm"` (default): Prompt for approval before tool execution.
- `"allow"`: Approve tool actions automatically.
- `"deny"`: Block tool actions.

Configure per-tool rules for granular control. MCP tools use the format `mcp:<server>:<tool_name>`. Example: `mcp:github:create_issue`.

Use the `default` key for MCP tools. Pattern-based rules match against empty strings for MCP tools, which usually prevents matches.

Read the [tool permissions documentation](./tool-permissions.md) for customization details.

### External Agents

Zed forwards configured MCP servers to [external agents](./external-agents.md) via the Agent Client Protocol. External agents also access servers through native configuration files.

Review [Configuration Boundaries](./external-agents.md#configuration-boundaries) for shared data details.

### Error Handling

The agent receives an error message and the operation fails when a server error occurs. Scenarios include:

- Invalid tool parameters
- Server-side failures like database connection issues or rate limits
- Unsupported operations or missing resources

The agent response displays the server error message. Check server logs or documentation for error code details.
