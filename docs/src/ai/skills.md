---
title: Agent Skills
description: Add specialized knowledge to Zed using skill files.
---

# Skills {#skills}

Skills provide the agent with instruction packages for specific tasks. You use these for test-driven development, document processing, or team coding standards.

Each skill folder contains a `SKILL.md` file with metadata and instructions. The agent catalogs installed skills and loads them when needed. You invoke skills using slash commands in the message editor.

## Adding Skills {#adding-skills}

### Create your own {#create-your-own}

Zed provides a `create-skill` tool. Type `/create-skill` to start the setup process.

Open the Skill Creator from the Agent Panel via {#kb agent::OpenRulesLibrary} or the `...` menu. Alternatively, use the {#action agent::OpenSkillCreator} action in the command palette. Enter the name, description, and instructions in the window.

Import skills from GitHub Markdown files. Select {#action agent::CreateSkillFromUrl} in the command palette. Zed fetches the content if your clipboard contains a GitHub URL.

### From the skills.sh Registry {#from-the-registry}

[skills.sh](https://skills.sh) hosts community skills for frameworks and tools.

Copy skill folders to `~/.agents/skills/` for global access or `.agents/skills/` within your project for local use.

## Managing Skills {#managing-skills}

Navigate to **AI > Skills** in the Settings Editor or visit [agent.skills](zed://settings/agent.skills).

The **User** tab lists global skills. The **Project** tab lists local project skills.

For each skill:

- **Copy Share Link**: copies a `zed://skill` link with embedded data.
- **Open**: opens `SKILL.md` in the editor.
- **Delete**: removes the folder.

The **Create a Skill** button opens the creator if you have no skills installed.

## Sharing Skills {#sharing-skills}

Share skills directly with teammates. Click the **link** icon in the Skills settings to copy a `zed://skill?data=…` URL. This link embeds the `SKILL.md` content.

Opening the link launches the Skill Creator with pre-filled fields. The recipient reviews the content and clicks **Save** to install. Zed only writes to disk after the user saves.

## Using Skills {#using-skills}

The agent identifies relevant skills by scanning the catalog of names and descriptions. It invokes the `skill` tool when a task matches a description.

Zed requests permission when the agent invokes a skill. You configure automatic approvals in [Tool Permissions](./tool-permissions.md).

### Manual Invocation {#manual-invocation}

Load skills manually:

- **Slash command**: Type `/` and select a skill.
- **@-mention**: Type `@skill` and choose from the menu.

These actions add the skill instructions to the context. Click the button in the thread to open the file.

### Preventing Autonomous Invocation {#disable-model-invocation}

Add `disable-model-invocation: true` to the frontmatter to prevent automatic use. The skill remains available via slash commands.

Use this for deployment or release procedures.

```yaml
---
name: deploy
description: Deploy the current branch to production.
disable-model-invocation: true
---
```

## Skill Format {#skill-format}

### Folder Structure {#folder-structure}

A skill folder contains these items:

```
my-skill/
├── SKILL.md          # Metadata and instructions
├── scripts/          # Scripts for the agent
├── references/       # Documentation
└── assets/           # Templates and files
```

The folder name and the `name` field in `SKILL.md` must match.

### SKILL.md format {#skill-md-format}

Put YAML frontmatter at the top of `SKILL.md`, followed by instructions.

**Minimal example:**

```markdown
---
name: my-skill
description: What this skill does and when to use it.
---

## Instructions

Step-by-step instructions for the agent...
```

#### Frontmatter Fields {#frontmatter-fields}

| Field                      | Required | Description                                                                                  |---
title: Agent Skills - Zed
description: Extend the Zed agent with skill files for specialized tasks.
---

# Skills {#skills}

Extend the Zed agent with skills. These folders contain a `SKILL.md` file with instructions for tasks like test-driven development, document processing, or database integration. You can invoke skills from the message editor using a slash command. The agent catalog lists all installed skills and loads them on demand.

## Adding Skills {#adding-skills}

### Create your own {#create-your-own}

Invoke the built-in `create-skill` skill with `/create-skill`. Zed guides you through the process.

Open the Skill Creator from the Agent Panel using {#kb agent::OpenRulesLibrary}. You can also click `...` and select **Skills**. The {#action agent::OpenSkillCreator} action in the command palette opens a window to define the name, description, and scope.

Create a skill from a GitHub Markdown URL using the {#action agent::CreateSkillFromUrl} action. Zed fetches the content if your clipboard contains a supported URL.

### From the skills.sh Registry {#from-the-registry}

[skills.sh](https://skills.sh) hosts community skills for frameworks and tools:

- [`find-skills`](https://skills.sh/vercel-labs/skills/find-skills): installs skills from the ecosystem
- [`frontend-design`](https://skills.sh/anthropics/skills/frontend-design): builds production-grade interfaces
- [`pdf`](https://skills.sh/anthropics/skills/pdf): extracts text and handles PDF forms

Install a skill by copying its folder into `~/.agents/skills/` for global use. Use the project `.agents/skills/` folder for local tasks.

## Managing Skills {#managing-skills}

Navigate to **AI > Skills** in the Settings Editor (`Cmd+,` or `Ctrl+,`), or go to [agent.skills](zed://settings/agent.skills).

The **User** tab manages global skills. The **Project** tab manages local ones.

Use these options for each skill:

- **Copy Share Link**: creates a `zed://skill` link (see [Sharing Skills](#sharing-skills))
- **Open**: opens the `SKILL.md` file
- **Delete**: removes the skill folder

The **Create a Skill** button appears if you have no skills installed.

## Sharing Skills {#sharing-skills}

Share skills without external hosting. Click the **link** icon in Skills settings to copy a `zed://skill?data=…` link. This self-contained link embeds the `SKILL.md` content.

When you open this link, Zed pre-fills the Skill Creator. You can review the instructions and click **Save** to install it. Zed only writes to disk after you confirm the save.

## Using Skills {#using-skills}

The agent selects skills autonomously. It identifies relevant skills from the catalog and calls the `skill` tool. Zed prompts you to allow or deny these calls. You can configure defaults in [Tool Permissions](./tool-permissions.md).

### Manual Invocation {#manual-invocation}

Load skills manually:

- **Slash command**: type `/` and select a skill
- **@-mention**: type `@skill` and choose from the menu

Both methods inject instructions into the thread. Click the crease button in the thread to open the skill file.

### Preventing Autonomous Invocation {#disable-model-invocation}

Set `disable-model-invocation: true` in the frontmatter to stop the agent from selecting a skill. You can still trigger these skills manually with a slash command. Use this for sensitive workflows like releases.

```yaml
---
name: deploy
description: Deploy the current branch to production.
disable-model-invocation: true
---
```

## Skill Format {#skill-format}

### Folder Structure {#folder-structure}

Each skill requires a folder containing a `SKILL.md` file. The folder name must match the `name` field in the frontmatter.

```
my-skill/
├── SKILL.md          # metadata and instructions
├── scripts/          # optional scripts
├── references/       # documentation
└── assets/           # templates
```

### SKILL.md format {#skill-md-format}

Start `SKILL.md` with YAML frontmatter.

```markdown
---
name: my-skill
description: Describe the task and when to use the skill.
---

## Instructions

Provide step-by-step instructions.
```

#### Frontmatter Fields {#frontmatter-fields}

| Field | Required | Description |
| :--- | :--- | :--- |
| `name` | Yes | Use lowercase letters, numbers, and hyphens. Max 64 characters. |
| `description` | Yes | Define what the skill does. Max 1024 characters. |
| `disable-model-invocation` | No | Set to `true` to restrict usage to slash commands. |

Write descriptions that help the agent identify the skill. Mention specific tasks and keywords.

#### Name Validation {#name-validation}

The `name` field must follow these rules:

- Use lowercase letters (`a-z`), numbers, and hyphens
- Do not start or end with a hyphen
- Do not use consecutive hyphens
- Keep the length between 1 and 64 characters

Invalid names prevent skills from loading.

### Bundled Resources {#bundled-resources}

Keep `SKILL.md` under 500 lines. Link to files in the `references/` folder for detailed material.

```markdown
See the [reference guide](references/REFERENCE.md) for API details.

Run this script:
scripts/extract.py
```

The agent uses `read_file` and `list_directory` to load these files. It can access global skills in `~/.agents/skills/` even if they sit outside the project.

### Writing Effective Instructions {#writing-instructions}

The agent sees the name and description first. It only loads the full body after activation. 

- Place critical instructions at the top
- Keep `SKILL.md` under 500 lines
- Move detailed data to `references/`
- Put executable code in `scripts/`

## Where Skills Live {#where-skills-live}

Zed loads skills from these locations:

| Scope | Path | Application |
| :--- | :--- | :--- |
| Global | `~/.agents/skills/` | Every project |
| Project-local | `<worktree>/.agents/skills/` | Current project only |

Nesting skills in subfolders is not supported.

### Project-local Skills and Trust {#project-local-trust}

Zed only loads project-local skills from [trusted worktrees](../worktree-trust.md). Review the project before granting trust to prevent unauthorized instructions.

### Override Behavior {#override-behavior}

Project-local skills take precedence over global skills with the same name. Use this to customize global workflows for specific projects.

### Editing Skill Files {#editing-skill-files}

The agent cannot modify `SKILL.md` or bundled resources without authorization. This protects the instructions governing the agent.

## Limitations {#limitations}

- **Flat layout**: Skills must reside directly in the skills root.
- **50KB catalog budget**: Zed caps the total size of names and descriptions at 50KB.
- **No remote registry**: Install skills into the local paths manually.
- **Live reload**: Changes to `SKILL.md` take effect.

## See also

- [Agent Panel](./agent-panel.md)
- [Tool Permissions](./tool-permissions.md)
- [Agent Skills specification](https://agentskills.io/specification)
