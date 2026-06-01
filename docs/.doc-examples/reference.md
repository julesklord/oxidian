---
title: AI Agent Tools - Zed
description: Built-in tools for Zed's AI agent including file editing, code search, terminal commands, web search, and diagnostics.
---

# Tools

Zed's built-in agent reads, searches, and edits code using these tools. You interact with them in the [Agent Panel](./agent-panel.md).

You configure permissions to approve, deny, or prompt for confirmation on specific tool actions. See [Tool Permissions](./tool-permissions.md) for details.

Add custom tools using [MCP servers](./mcp.md).

## Read & Search Tools {#read-search-tools}

### `diagnostics` {#diagnostics}

Gets errors and warnings for a specific file or the entire project. Passing a path retrieves diagnostics for that file. Omitting the path returns a summary count for the project.

### `fetch` {#fetch}

Fetches a URL and returns the content as Markdown.

### `find_path` {#find-path}

Finds files matching glob patterns and returns paths alphabetically.

### `grep` {#grep}

Searches project files using regular expressions. This locates symbols when exact paths remain unknown.

### `list_directory` {#list-directory}

Lists files and directories at a specified path.

### `read_file` {#read-file}

Reads the content of a specified project file.

### `search_web` {#search-web}

Searches the web and returns results with text snippets and links.

## Edit Tools {#edit-tools}

### `copy_path` {#copy-path}

Copies a file or directory recursively.

### `create_directory` {#create-directory}

Creates a directory at the specified path. It builds required parent directories automatically.

### `delete_path` {#delete-path}

Deletes a file or directory and its contents at the specified path.

### `edit_file` {#edit-file}

Replaces specific text within a file with new content.

### `move_path` {#move-path}

Moves or renames a file or directory.

### `write_file` {#write-file}

Creates a new file or overwrites an existing file with new content.

### `terminal` {#terminal}

Executes shell commands and returns the combined output. It creates a distinct shell process for each invocation.

## Other Tools {#other-tools}

### `spawn_agent` {#spawn-agent}

Spawns a subagent with a dedicated context window to perform a delegated task. Subagents run parallel investigations or self-contained research. Each subagent accesses the same tools as the parent agent.

## See Also {#see-also}

- [Agent Panel](./agent-panel.md): Interface for AI agents.
- [Tool Permissions](./tool-permissions.md): Configure tool approval rules.
- [MCP Servers](./mcp.md): Add custom tools via Model Context Protocol.
