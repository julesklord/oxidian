description: Zed is a text editor that supports lots of Git features
title: Zed Editor Git integration documentation

---

# Git

Zed includes Git support for version control. The Git Panel displays the working tree, staging area, and branch information. Zed reflects command line changes instantly.

Access the integrated terminal for operations Zed does not support natively.

## Git Panel {#git-panel}

The Git Panel displays the working tree and staging area.

Open the Git Panel with {#action git_panel::ToggleFocus} or the status bar icon.

The panel shows the active repository, branch, changed files, and staging states.

Zed updates when you change the repository via the command line.

### Configuration {#configuration}

Configure Git in the Settings Editor (`Cmd+,` on macOS, `Ctrl+,` on Linux/Windows). Settings reside on two pages:

- **Panels > Git Panel**: covers position, tree views, and status styles.
- **Version Control**: covers gutter indicators, inline blame, and hunk styles.

#### Moving the Git Panel

The Git Panel docks on the left. Adjust the position in **Panels > Git Panel > Git Panel Dock**.

#### Switching to Tree View

The Git Panel defaults to a flat list. Toggle **Tree View** in the context menu or **Panels > Git Panel** for a folder hierarchy.

#### Inline Blame

Zed displays Git blame on the current line. Configure visibility and delays in **Version Control > Inline Git Blame**.

#### Hiding the Gutter Indicators

Hide the colored gutter bars for added, modified, or deleted lines in **Version Control > Git Gutter > Visibility**.

#### Commit Message Line Length

Zed wraps commit messages at 72 characters. Adjust **Preferred Line Length** in the Git Commit settings.

## Project Diff {#project-diff}

View Git changes in the Project Diff ({#kb git::Diff}). Access this through the Command Palette or the Git Panel.

Project Diff excerpts function as editable multibuffers.

Stage or unstage hunks and files using tab bar buttons or keybindings.

### Word Diff Highlighting {#word-diff}

Zed highlights changed words within modified lines. Disable this in **Settings > Languages & Tools > Miscellaneous > Word Diff Enabled**.

To disable word diff for specific languages, update settings.json:

```json
{
  "languages": {
    "Markdown": {
      "word_diff_enabled": false
    }
  }
}
```

## File History {#file-history}

File History lists commit authors, timestamps, and messages for a file. Selecting a commit opens a filtered diff showing changes from that commit.

To view File History:

- Right-click a file in the Project Panel and select "View File History".
- Right-click a file in the Git Panel and select "View File History".
- Right-click an editor tab and select "View File History".
- Search "file history" in the Command Palette.

## Fetch, Push, and Pull {#fetch-push-pull}

Execute fetch, push, or pull from the Git Panel or Command Palette actions: {#action git::Fetch}, {#action git::Push}, and {#action git::Pull}.

### Push Configuration {#push-configuration}

Zed follows Git push configurations in this order:

1. `pushRemote` for the current branch.
2. `remote.pushDefault` in Git config.
3. The branch's tracking remote.

Zed uses existing `pushRemote` or `pushDefault` settings from your `.gitconfig`.

## Remotes {#remotes}

The Git Panel displays a remote selector for repositories with multiple remotes. Select a remote for push or pull operations.

## Staging Workflow {#staging-workflow}

Stage changes through the Project Diff or the Git Panel.

### Using the Project Diff {#staging-project-diff}

Focus on hunks in the Project Diff to stage them via tab bar buttons or {#action git::StageAndNext} ({#kb git::StageAndNext}).

Stage all hunks with {#action git::StageAll} ({#kb git::StageAll}) and commit using {#action git::Commit} ({#kb git::Commit}).

### Using the Git Panel {#staging-git-panel}

Type a message in the Git Panel and click commit or use {#action git::Commit}. This stages tracked files marked with `[·]` and commits them.

Stage individual entries using checkboxes. Stage all changes with the panel button or {#action git::StageAll}.

## Committing {#committing}

Zed provides two commit textareas.

1. The Git Panel footer textarea. {#kb git::Commit} commits staged changes.
2. The commit editor, accessible via {#action git::ExpandCommitEditor} or {#kb git::ExpandCommitEditor} while focused in the Git Panel.

### Undoing a Commit {#undo-commit}

After committing, the Git Panel displays the recent commit below the message area. The "Uncommit" button runs `git reset HEAD~ --soft`.

### Configuring Commit Line Length

Zed defaults the commit line length to `72`. Configure this in `settings.json`.

See the [Configuration](#configuration) section for details on `preferred-line-length`.

## Branch Management {#branch-management}

### Creating and Switching Branches {#create-switch-branches}

Create branches with {#action git::Branch}. Switch branches with {#action git::Switch} or {#action git::CheckoutBranch}.

### Deleting Branches {#delete-branches}

Delete branches through the {#action git::Switch} menu. Zed requires confirmation before deletion.

> **Note:** You cannot delete the active branch. Switch to a different branch first.

## Merge Conflicts {#merge-conflicts}

Zed highlights merge, rebase, or pull conflicts and provides resolution buttons.

### Viewing Conflicts {#viewing-conflicts}

Conflicting files display a warning icon in the Git Panel. The Project Diff highlights conflict regions:

- Green: Changes from the current branch.
- Blue: Changes from the incoming branch.

### Resolving Conflicts {#resolving-conflicts}

Conflict resolution offers three options:

- **Use [branch-name]**: Retain changes from that specific branch.
- **Use [other-branch]**: Retain changes from the incoming branch.
- **Use Both**: Retain both sets of changes.

Clicking a button resolves the conflict and removes markers. Stage and commit the file after resolving all conflicts.

> **Tip:** Edit complex conflicts manually. Remove markers like `<<<<<<<` and `=======` while keeping the desired content.

## Stashing {#stashing}

Git stash saves uncommitted changes and cleans the working directory.

### Creating Stashes {#creating-stashes}

Stash all changes with {#action git::StashAll}. This saves staged and unstaged changes.

### Managing Stashes {#managing-stashes}

Access the stash picker through {#action git::ViewStash} or the Git Panel overflow menu. The picker supports these tasks:

- **View stash list**: Browse saved stashes with descriptions.
- **Open diffs**: Inspect changes in a stash.
- **Apply stashes**: Apply changes while keeping the stash entry.
- **Pop stashes**: Apply changes and remove the entry.
- **Drop stashes**: Delete entries without applying them.

### Quick Stash Operations {#quick-stash}

Zed provides direct actions for the recent stash:

- **Apply latest stash**: {#action git::StashApply} applies the recent stash.
- **Pop latest stash**: {#action git::StashPop} applies and removes the recent stash.

### Stash Diff View {#stash-diff-view}

Select a stash in the picker and press {#kb stash_picker::ShowStashItem} to view its content. Use these keybindings in the diff view:

| Action                               | Keybinding                   |
| ------------------------------------ | ---------------------------- |
| Apply stash                          | {#kb git::ApplyCurrentStash} |
| Pop stash (apply and remove)         | {#kb git::PopCurrentStash}   |
| Drop stash (remove without applying) | {#kb git::DropCurrentStash}  |

## AI Support in Git {#ai-support}

Zed generates commit messages using LLMs. Click the pencil icon in the Git Panel editor or use {#action git::GenerateCommitMessage} ({#kb git::GenerateCommitMessage}).

> **Note:** Configure an LLM provider via API keys or Zed's hosted models. See the [AI configuration page](./ai/configuration.md).

Set a preferred model in the `commit_message_model` agent setting. See [Feature-specific models](./ai/agent-settings.md#feature-specific-models).

```json [settings]
{
  "agent": {
    "commit_message_model": {
      "provider": "anthropic",
      "model": "claude-3-5-haiku"
    }
  }
}
```

Modify the "Commit message" rule in {#action agent::OpenRulesLibrary} to customize generation formats.

The model incorporates commit message instructions from [Rules files](./ai/rules.md).

## Git Integrations {#git-integrations}

Zed links commit hashes and references for Issues, Pull Requests, and Merge Requests to Git hosting services.

Supported services include GitHub, GitLab, Bitbucket, SourceHut, and Codeberg.

### Self-Hosted Instances {#self-hosted}

Zed identifies providers by remote URL keywords. If the URL lacks keywords, configure `git_hosting_providers` in settings:

```json [settings]
{
  "git_hosting_providers": [
    {
      "provider": "gitlab",
      "name": "Corp GitLab",
      "base_url": "https://git.example.corp"
    }
  ]
}
```

Supported `provider` values include `github`, `gitlab`, `bitbucket`, `gitea`, `forgejo`, and `sourcehut`. `base_url` defines the server root.

### Permalinks {#permalinks}

The Copy Permalink feature generates permanent links to code snippets. Use the Command Palette, custom keybindings for `editor::CopyPermalinkToLine`, or the right-click menu to generate permalinks for selected lines.

## Diff Hunk Keyboard Shortcuts {#diff-hunks}

Zed displays expandable diff hunks for changed files:

- **Expand all diff hunks**: {#action editor::ExpandAllDiffHunks} ({#kb editor::ExpandAllDiffHunks})
- **Collapse all diff hunks**: Press `Escape` ({#action editor::Cancel})
- **Toggle selected diff hunks**: {#action editor::ToggleSelectedDiffHunks} ({#kb editor::ToggleSelectedDiffHunks})
- **Navigate between hunks**: {#action editor::GoToHunk} and {#action editor::GoToPreviousHunk}

> **Tip:** Press `Escape` to collapse all expanded hunks.

## Action Reference {#action-reference}

| Action                                    | Keybinding                            |
| ----------------------------------------- | ------------------------------------- |
| {#action git::Add}                        | {#kb git::Add}                        |
| {#action git::StageAll}                   | {#kb git::StageAll}                   |
| {#action git::UnstageAll}                 | {#kb git::UnstageAll}                 |
| {#action git::ToggleStaged}               | {#kb git::ToggleStaged}               |
| {#action git::StageAndNext}               | {#kb git::StageAndNext}               |
| {#action git::UnstageAndNext}             | {#kb git::UnstageAndNext}             |
| {#action git::Commit}                     | {#kb git::Commit}                     |
| {#action git::ExpandCommitEditor}         | {#kb git::ExpandCommitEditor}         |
| {#action git::Push}                       | {#kb git::Push}                       |
| {#action git::ForcePush}                  | {#kb git::ForcePush}                  |
| {#action git::Pull}                       | {#kb git::Pull}                       |
| {#action git::PullRebase}                 | {#kb git::PullRebase}                 |
| {#action git::Fetch}                      | {#kb git::Fetch}                      |
| {#action git::Diff}                       | {#kb git::Diff}                       |
| {#action git::Restore}                    | {#kb git::Restore}                    |
| {#action git::RestoreFile}                | {#kb git::RestoreFile}                |
| {#action git::Branch}                     | {#kb git::Branch}                     |
| {#action git::Switch}                     | {#kb git::Switch}                     |
| {#action git::CheckoutBranch}             | {#kb git::CheckoutBranch}             |
| {#action git::Blame}                      | {#kb git::Blame}                      |
| {#action git::StashAll}                   | {#kb git::StashAll}                   |
| {#action git::StashPop}                   | {#kb git::StashPop}                   |
| {#action git::StashApply}                 | {#kb git::StashApply}                 |
| {#action git::ViewStash}                  | {#kb git::ViewStash}                  |
| {#action editor::ToggleGitBlameInline}    | {#kb editor::ToggleGitBlameInline}    |
| {#action editor::ExpandAllDiffHunks}      | {#kb editor::ExpandAllDiffHunks}      |
| {#action editor::ToggleSelectedDiffHunks} | {#kb editor::ToggleSelectedDiffHunks} |

> **Note:** Assign custom keybindings to actions without defaults in your [user keymap](./key-bindings.md#user-keymaps).

## Git CLI Configuration {#cli-configuration}

Use Zed as a command-line Git editor with `zed --wait`:

```sh
git config --global core.editor "zed --wait"
```

Or update your shell environment (`~/.zshrc`, `~/.bashrc`):

```sh
export GIT_EDITOR="zed --wait"
```
