# 💎 Oxidian

**The native, high-performance Markdown knowledge base for the [Zed](https://zed.dev) editor.**

[![Oxidian](https://img.shields.io/badge/Powered%20by-Zed-blue)](https://zed.dev)
[![License](https://img.shields.io/badge/License-GPL--3.0--or--later-green)](LICENSE-GPL)
[![Rust](https://img.shields.io/badge/Language-Rust-orange)](https://www.rust-lang.org)

Oxidian is a suite of tools that transforms the Zed editor into a powerful, sub-millisecond response-time knowledge management system. Built entirely in Rust and leveraging the GPUI rendering engine, it provides a fluid experience for managing large-scale Markdown vaults without the overhead of Electron-based alternatives.

## 🚀 Key Features

- **⚡ Native Performance:** Say goodbye to lag. Oxidian is built directly into the Zed architecture, ensuring that searching, indexing, and rendering are nearly instantaneous.
- **🔗 Backlinks Panel:** Discover and navigate relationships between your notes. The backlinks panel automatically identifies all notes that reference your current file.
- **📅 Daily Notes:** Streamline your workflow with a dedicated daily journaling system. Supports custom templates and automatic date-based file creation.
- **🏷️ Metadata & Tag Browser:** Deep integration with Markdown frontmatter (YAML). Browse your knowledge graph by tags and manage properties with ease.
- **📦 Vault Management:** Advanced indexing powered by SQLite and a background indexer. It keeps track of your vault's structure without getting in your way.
- **🐙 Git Integration:** Keep your knowledge base safe and versioned. Built-in status indicators help you stay on top of your conflicts and changes.
- **🤖 Agent Optimized:** Designed from the ground up to play nicely with AI agents and MCP (Model Context Protocol).

## 🏗️ Architecture

Oxidian is organized as a modular set of crates within the Zed monorepo:

- `oxidian_core`: Shared types and foundational logic.
- `oxidian_vault`: The engine for indexing, searching, and managing vault configurations.
- `oxidian_backlinks`: The UI and logic for the Backlinks dock panel.
- `oxidian_daily`: Workflow tools for daily note management.
- `oxidian_frontmatter`: YAML frontmatter parser and Tag Browser panel.
- `oxidian_git`: Git status monitoring for vaults.

## 🛠️ Getting Started

### Prerequisites

Since Oxidian builds as part of Zed, you will need the standard Rust toolchain and dependencies for your platform.

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/oxidian-org/oxidian.git
   cd oxidian
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the editor:
   ```bash
   ./target/release/zed
   ```

Refer to the [development documentation](./docs/src/development) for more specific platform instructions (Linux, macOS, Windows).

## 🗺️ Roadmap

- [x] Efficient Vault Indexing (SQLite)
- [x] Backlinks Panel
- [x] Daily Notes & Templates
- [x] Tag Browser & Metadata support
- [ ] 🚧 Wiki-link autocomplete (Marksman LSP integration)
- [ ] 🚧 Native GPUI Graph view
- [ ] 🚧 Canvas-like infinite board support

## 🤝 Contributing

We welcome contributions of all types! Whether it's a bug report, a feature request, or a pull request, please check out our [CONTRIBUTING.md](CONTRIBUTING.md) to get started.

## 📜 Licensing

Oxidian is licensed under the same terms as Zed: **GPL-3.0-or-later** or **Apache-2.0**. See the [LICENSE-GPL](LICENSE-GPL) and [LICENSE-APACHE](LICENSE-APACHE) files for details.

---

*Built with ❤️ and Rust.*
