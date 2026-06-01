# Parallel Agents

Run agent and terminal threads simultaneously in the Threads Sidebar. Threads maintain independent agents, context windows, and histories. The sidebar integrates terminal and agent threads for quick switching.

Use {#kb multi_workspace::ToggleWorkspaceSidebar} to open the Threads Sidebar.

Zed version 0.233.0 and later places the Agent Panel and Threads Sidebar on the left. The Project and Git panels reside on the right. Right-click any panel icon to rearrange them.

## Threads Sidebar

The sidebar groups threads by project. Each project section displays titles, status indicators, and active agents. Threads in linked Git worktrees remain with their parent project. See [Worktree Isolation](#worktree-isolation).

Terminal threads display a terminal icon in the sidebar. Select an entry to switch focus. See [terminal threads](./agent-panel.md#terminal-threads).

Focus the sidebar with {#kb multi_workspace::FocusWorkspaceSidebar}. Search threads using {#kb agents_sidebar::FocusSidebarFilter} while the sidebar is active.

### Switching Threads

Click any sidebar entry to switch threads. The Agent Panel updates immediately to the selected conversation. Cycle through recent threads with {#kb agents_sidebar::ToggleThreadSwitcher}. Hold `Shift` with the binding to cycle backward.

### Thread History

Archive threads by clicking the archive icon on hover or pressing {#kb agent::ArchiveSelectedThread}. Active threads must finish before archiving. Toggle Thread History via {#kb agents_sidebar::ToggleThreadHistory} or the clock icon in the sidebar.

Restore threads by selecting them in the History view. Zed moves the thread to the active list and reopens the conversation. History restoration automatically rebuilds associated Git worktrees. 

Use the trash icon in History to delete threads permanently. This action removes conversation history and associated worktree data. History search uses fuzzy matching on titles.

### Importing External Agent Threads

Zed detects existing threads from external agents and prompts for import. Open Thread History and click the import button to select agents. External agent support varies. Zed does not currently support Cursor or Gemini CLI imports.

## Running Multiple Threads

Threads run independently. Users can start tasks in separate threads simultaneously. Scope new threads by clicking the `+` button on a project header or using {#action agents_sidebar::NewThreadInGroup}. Each thread supports different agents.

## Multiple Projects

The sidebar supports multiple projects in dedicated groups. Use the **Add Project** button to include more folders. The popover displays recent projects and options for local or remote folders.

### Multi-Root Folder Projects

Multi-root projects allow agents to access multiple folders in one thread. 

* **Sidebar**: Select **Add Local Folders** from the **Add Project** menu.
* **Title bar**: Click the project picker, hover a local entry, and select **Add Folder to this Project**.
* **Project panel**: Right-click a root or empty space and choose **Add Folders to Project**.

## Worktree Isolation

Prevent file conflicts by running threads in separate [Git worktrees](../git.md#git-worktrees). Manage worktrees through the title bar picker. New worktrees start in a detached HEAD state to prevent accidental branch sharing. 

Use the branch picker to create or check out branches. If a branch is active in another worktree, the current worktree remains in detached HEAD. 

The `create_worktree` [Task hook](../tasks.md#hooks) automates setup. `ZED_WORKTREE_ROOT` identifies the new path. `ZED_MAIN_GIT_WORKTREE` references the original repository. 

Review and merge changes through standard Git workflows. Archiving a thread removes the worktree from disk while preserving the Git state. Restoring the thread recreates the worktree.

## See Also

* [Agent Panel](./agent-panel.md): Manage individual threads and configure the agent
* [External Agents](./external-agents.md): Use Claude Code, Gemini CLI, and other agents
* [Tools](./tools.md): Built-in tools available in each thread
