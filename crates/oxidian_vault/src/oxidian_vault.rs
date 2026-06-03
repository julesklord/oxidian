use anyhow::{Context as _, Result};
use async_channel as channel;
use collections::HashMap;
use db::sqlez::domain::Domain;
use db::sqlez_macros::sql;
use db::static_connection;
use fs::Fs;
use gpui::{App, AppContext as _, BorrowAppContext, Context, Entity, EventEmitter, Global, Task};
use oxidian_core::{NoteId, VaultConfig, WikiLink, WikiLinkResolver};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use worktree::{PathChange, UpdatedEntriesSet, WorktreeId};

// OXIDIAN BEGIN — vault database domain

/// SQLite domain for the Oxidian vault index.
pub struct VaultDatabase(db::sqlez::thread_safe_connection::ThreadSafeConnection);

impl Domain for VaultDatabase {
    const NAME: &str = "oxidian_vault";

    const MIGRATIONS: &[&str] = &[sql!(
        CREATE TABLE IF NOT EXISTS vault_notes (
            note_id     TEXT PRIMARY KEY,
            title       TEXT NOT NULL,
            path        TEXT NOT NULL,
            modified_at INTEGER NOT NULL
        ) STRICT;

        CREATE TABLE IF NOT EXISTS vault_links (
            from_note   TEXT NOT NULL,
            to_target   TEXT NOT NULL,
            alias       TEXT,
            heading     TEXT,
            line        INTEGER NOT NULL,
            FOREIGN KEY (from_note) REFERENCES vault_notes(note_id) ON DELETE CASCADE
        ) STRICT;

        CREATE INDEX IF NOT EXISTS idx_vault_links_to_target
            ON vault_links(to_target);

        CREATE TABLE IF NOT EXISTS vault_tags (
            note_id TEXT NOT NULL,
            tag     TEXT NOT NULL,
            PRIMARY KEY (note_id, tag),
            FOREIGN KEY (note_id) REFERENCES vault_notes(note_id) ON DELETE CASCADE
        ) STRICT;

        CREATE INDEX IF NOT EXISTS idx_vault_tags_tag
            ON vault_tags(tag);
    )];
}

static_connection!(VaultDatabase, []);

// OXIDIAN END

// OXIDIAN BEGIN — vault queries

impl VaultDatabase {
    db::query! {
        pub async fn upsert_note(
            note_id: String,
            title: String,
            path: String,
            modified_at: i64
        ) -> Result<()> {
            INSERT OR REPLACE INTO vault_notes(note_id, title, path, modified_at)
            VALUES ((?), (?), (?), (?))
        }
    }

    db::query! {
        pub async fn delete_note(note_id: String) -> Result<()> {
            DELETE FROM vault_notes WHERE note_id = (?)
        }
    }

    db::query! {
        pub async fn delete_links_from(note_id: String) -> Result<()> {
            DELETE FROM vault_links WHERE from_note = (?)
        }
    }

    db::query! {
        pub async fn insert_link(
            from_note: String,
            to_target: String,
            alias: Option<String>,
            heading: Option<String>,
            line: i64
        ) -> Result<()> {
            INSERT INTO vault_links(from_note, to_target, alias, heading, line)
            VALUES ((?), (?), (?), (?), (?))
        }
    }

    db::query! {
        pub fn get_backlinks(target: &str) -> Result<Vec<(String, Option<String>, i64)>> {
            SELECT from_note, alias, line FROM vault_links WHERE to_target = (?)
        }
    }

    db::query! {
        pub fn all_note_ids() -> Result<Vec<String>> {
            SELECT note_id FROM vault_notes ORDER BY note_id
        }
    }

    db::query! {
        pub fn resolve_note_path(note_id: &str) -> Result<Option<String>> {
            SELECT path FROM vault_notes WHERE note_id = (?)
        }
    }

    db::query! {
        pub async fn upsert_tag(note_id: String, tag: String) -> Result<()> {
            INSERT OR IGNORE INTO vault_tags(note_id, tag) VALUES ((?), (?))
        }
    }

    db::query! {
        pub async fn delete_tags_for_note(note_id: String) -> Result<()> {
            DELETE FROM vault_tags WHERE note_id = (?)
        }
    }

    db::query! {
        pub fn notes_with_tag(tag: &str) -> Result<Vec<String>> {
            SELECT note_id FROM vault_tags WHERE tag = (?)
        }
    }

    db::query! {
        pub fn all_tags_with_counts() -> Result<Vec<(String, i64)>> {
            SELECT tag, COUNT(note_id) FROM vault_tags GROUP BY tag ORDER BY tag ASC
        }
    }
}

// OXIDIAN END

// OXIDIAN BEGIN — vault index

/// Events emitted by `VaultIndex` to notify the UI layer.
#[derive(Debug, Clone)]
pub enum VaultEvent {
    /// A note was added or re-indexed.
    NoteIndexed(NoteId),
    /// A note was removed from the vault.
    NoteRemoved(NoteId),
    /// The full vault scan completed.
    InitialScanComplete,
}

/// The vault index: watches the vault directory and keeps the SQLite index up to date.
pub struct VaultIndex {
    pub config: VaultConfig,
    db: VaultDatabase,
    /// All currently known note IDs, mapped to their absolute paths.
    notes: HashMap<NoteId, PathBuf>,
    _watcher_task: Task<()>,
}

impl EventEmitter<VaultEvent> for VaultIndex {}

impl VaultIndex {
    /// Creates and starts a new vault index for the given config.
    /// Immediately starts a background scan of the vault directory.
    pub fn new(config: VaultConfig, fs: Arc<dyn Fs>, cx: &mut Context<Self>) -> Self {
        let db = VaultDatabase::global(cx);

        let vault_root = config.root.clone();
        let (scan_sender, scan_receiver) = channel::unbounded::<PathBuf>();

        let watcher_task = cx.spawn({
            let fs = fs.clone();
            async move |this, cx| {
                // Initial scan — enumerate all .md files in the vault
                if let Ok(entries) = Self::scan_vault_directory(&vault_root, &*fs).await {
                    for path in entries {
                        scan_sender.send(path).await.ok();
                    }
                }

                while let Ok(path) = scan_receiver.recv().await {
                    let note_id = Self::note_id_for_path(&vault_root, &path);
                    let Some(note_id) = note_id else { continue };

                    match this.update(cx, |vault, cx| vault.index_note(note_id.clone(), path, cx)) {
                        Ok(task) => cx
                            .background_spawn(async move {
                                if let Err(err) = task.await {
                                    log::error!("VaultIndex: failed to index note: {err}");
                                }
                            })
                            .detach(),
                        Err(err) => {
                            log::error!("VaultIndex: entity gone while indexing {note_id}: {err}");
                            break;
                        }
                    }
                }

                this.update(cx, |_, cx| {
                    cx.emit(VaultEvent::InitialScanComplete);
                })
                .ok();
            }
        });

        Self {
            config,
            db,
            notes: HashMap::default(),
            _watcher_task: watcher_task,
        }
    }

    /// Returns the absolute path for a given `NoteId`, if it's in the index.
    pub fn resolve_note(&self, note_id: &NoteId) -> Option<&PathBuf> {
        self.notes.get(note_id)
    }

    /// Fuzzy-resolves a wiki-link target string to the best matching `NoteId`.
    /// Tries exact match first, then basename match, then prefix match.
    pub fn resolve_wiki_link(&self, target: &str) -> Option<NoteId> {
        let normalized = target.trim();

        // Exact match
        let candidate = NoteId::from_relative_path(normalized);
        if self.notes.contains_key(&candidate) {
            return Some(candidate);
        }

        // Basename match (target matches the last component of the path)
        for note_id in self.notes.keys() {
            let basename = note_id
                .as_str()
                .rsplit('/')
                .next()
                .unwrap_or(note_id.as_str());
            if basename.eq_ignore_ascii_case(normalized) {
                return Some(note_id.clone());
            }
        }

        None
    }

    /// Indexes a single note: extracts wiki-links, updates the DB.
    fn index_note(
        &mut self,
        note_id: NoteId,
        path: PathBuf,
        cx: &mut Context<Self>,
    ) -> Task<Result<()>> {
        let db = self.db.clone();
        let note_id_str = note_id.as_str().to_owned();
        let path_str = path.to_string_lossy().into_owned();

        self.notes.insert(note_id, path.clone());

        cx.background_spawn(async move {
            let metadata = std::fs::metadata(&path).context("reading note metadata")?;
            let modified_at = metadata
                .modified()
                .context("reading modification time")?
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let content = std::fs::read_to_string(&path).context("reading note content")?;

            let title = extract_title(&content).unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&note_id_str)
                    .to_owned()
            });

            let wiki_links = extract_wiki_links_from_text(&content);
            let tags = extract_tags_from_frontmatter(&content);

            db.upsert_note(note_id_str.clone(), title, path_str, modified_at)
                .await?;
            db.delete_links_from(note_id_str.clone()).await?;
            db.delete_tags_for_note(note_id_str.clone()).await?;

            for (line, link) in wiki_links {
                db.insert_link(
                    note_id_str.clone(),
                    link.target.to_string(),
                    link.alias.map(|s| s.to_string()),
                    link.heading.map(|s| s.to_string()),
                    line as i64,
                )
                .await?;
            }

            for tag in tags {
                db.upsert_tag(note_id_str.clone(), tag).await?;
            }

            Ok(())
        })
    }

    /// Removes a single note from the index and updates the DB.
    fn remove_note(&mut self, note_id: NoteId, cx: &mut Context<Self>) -> Task<Result<()>> {
        let db = self.db.clone();
        let note_id_str = note_id.as_str().to_owned();

        self.notes.remove(&note_id);

        cx.background_spawn(async move {
            db.delete_note(note_id_str.clone()).await?;
            db.delete_links_from(note_id_str.clone()).await?;
            db.delete_tags_for_note(note_id_str.clone()).await?;
            Ok(())
        })
    }

    /// Handles worktree updates in real-time, re-indexing or removing notes as appropriate.
    pub fn handle_worktree_updated_entries(
        &mut self,
        _worktree_id: WorktreeId,
        entries: &UpdatedEntriesSet,
        cx: &mut Context<Self>,
    ) {
        let vault_root = self.config.root.clone();
        for (rel_path, _, change) in entries.iter() {
            let abs_path = vault_root.join(rel_path.as_ref());
            if abs_path.extension().is_some_and(|ext| ext == "md") {
                let name = abs_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.starts_with('.') {
                    continue;
                }

                let note_id = Self::note_id_for_path(&vault_root, &abs_path);
                let Some(note_id) = note_id else { continue };

                match change {
                    PathChange::Removed => {
                        let task = self.remove_note(note_id.clone(), cx);
                        let note_id_clone = note_id.clone();
                        cx.background_spawn(async move {
                            if let Err(err) = task.await {
                                log::error!("VaultIndex: failed to remove note: {err}");
                            }
                        })
                        .detach();
                        cx.emit(VaultEvent::NoteRemoved(note_id_clone));
                    }
                    PathChange::Added
                    | PathChange::Updated
                    | PathChange::AddedOrUpdated
                    | PathChange::Loaded => {
                        let task = self.index_note(note_id.clone(), abs_path, cx);
                        let note_id_clone = note_id.clone();
                        cx.background_spawn(async move {
                            if let Err(err) = task.await {
                                log::error!("VaultIndex: failed to index note: {err}");
                            }
                        })
                        .detach();
                        cx.emit(VaultEvent::NoteIndexed(note_id_clone));
                    }
                }
            }
        }
    }

    async fn scan_vault_directory(root: &Path, _fs: &dyn Fs) -> Result<Vec<PathBuf>> {
        let root = root.to_path_buf();
        // Run the directory scan on the thread pool since std::fs is synchronous
        smol::unblock(move || {
            let mut markdown_files = Vec::new();
            let mut stack = vec![root];

            while let Some(dir) = stack.pop() {
                let entries = std::fs::read_dir(&dir).context("scanning vault directory")?;

                for entry in entries {
                    let entry = entry.context("reading directory entry")?;
                    let path = entry.path();
                    let file_name = entry.file_name();
                    let name = file_name.to_string_lossy();

                    // Skip hidden directories (including .oxidian, .git, etc.)
                    if name.starts_with('.') {
                        continue;
                    }

                    let metadata = entry.metadata().context("reading entry metadata")?;

                    if metadata.is_dir() {
                        stack.push(path);
                    } else if path.extension().is_some_and(|ext| ext == "md") {
                        markdown_files.push(path);
                    }
                }
            }

            Ok(markdown_files)
        })
        .await
    }

    fn note_id_for_path(vault_root: &Path, path: &Path) -> Option<NoteId> {
        let relative = path.strip_prefix(vault_root).ok()?;
        let relative_str = relative.to_str()?;
        Some(NoteId::from_relative_path(relative_str))
    }
}

// OXIDIAN END

// OXIDIAN BEGIN — wiki-link extraction from raw text

/// Extracts all wiki-links from raw Markdown text, returning (line_number, WikiLink) pairs.
/// This runs before pulldown-cmark to capture `[[...]]` syntax.
pub fn extract_wiki_links_from_text(text: &str) -> Vec<(usize, WikiLink)> {
    let mut results = Vec::new();

    for (line_index, line) in text.lines().enumerate() {
        let mut search_start = 0;

        while let Some(open) = line[search_start..].find("[[") {
            let open_abs = search_start + open;
            let after_open = open_abs + 2;

            if let Some(close_rel) = line[after_open..].find("]]") {
                let close_abs = after_open + close_rel;
                let inner = &line[after_open..close_abs];

                // Skip empty links and links containing newlines (shouldn't happen in single line)
                if !inner.is_empty() {
                    results.push((line_index, WikiLink::parse(inner)));
                }

                search_start = close_abs + 2;
            } else {
                // No closing ]] on this line — skip rest of line
                break;
            }
        }
    }

    results
}

// OXIDIAN END

// OXIDIAN BEGIN — frontmatter and title helpers

/// Extracts the first `# Heading` as the note title, or the first non-empty line.
fn extract_title(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("# ") {
            return Some(stripped.trim().to_owned());
        }
        // Skip frontmatter delimiters and empty lines
        if !trimmed.is_empty() && trimmed != "---" && trimmed != "+++" {
            return Some(trimmed.to_owned());
        }
    }
    None
}

/// Extracts tags from YAML frontmatter `tags:` field.
/// Supports both `tags: [a, b]` and `tags:\n  - a\n  - b` formats.
fn extract_tags_from_frontmatter(content: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut in_frontmatter = false;
    let mut in_tags_block = false;
    let mut first_line = true;

    for line in content.lines() {
        let trimmed = line.trim();

        if first_line && trimmed == "---" {
            in_frontmatter = true;
            first_line = false;
            continue;
        }
        first_line = false;

        if !in_frontmatter {
            break;
        }

        if trimmed == "---" || trimmed == "+++" {
            break;
        }

        if let Some(rest) = trimmed.strip_prefix("tags:") {
            let value = rest.trim();
            if value.starts_with('[') && value.ends_with(']') {
                // Inline array: tags: [rust, zed]
                let inner = &value[1..value.len() - 1];
                for tag in inner.split(',') {
                    let tag = tag.trim().trim_matches('"').trim_matches('\'');
                    if !tag.is_empty() {
                        tags.push(tag.to_owned());
                    }
                }
                in_tags_block = false;
            } else if !value.is_empty() {
                let tag = value.trim_matches('"').trim_matches('\'');
                if !tag.is_empty() {
                    tags.push(tag.to_owned());
                }
                in_tags_block = false;
            } else if value.is_empty() {
                // Block array follows
                in_tags_block = true;
            }
            continue;
        }

        if in_tags_block {
            if let Some(tag) = trimmed.strip_prefix("- ") {
                let tag = tag.trim().trim_matches('"').trim_matches('\'');
                if !tag.is_empty() {
                    tags.push(tag.to_owned());
                }
            } else if !trimmed.is_empty() {
                in_tags_block = false;
            }
        }
    }

    tags
}

// OXIDIAN END

// OXIDIAN BEGIN — vault detection

/// Returns true if the given directory contains an `.oxidian` marker (our vault),
/// or an `.obsidian` directory (an Obsidian vault we can read).
pub fn is_vault_root(path: &Path) -> bool {
    path.join(".oxidian").exists() || path.join(".obsidian").exists()
}

/// Builds a `VaultConfig` for the given root, importing Obsidian settings if present.
pub fn load_vault_config(root: PathBuf) -> VaultConfig {
    let obsidian_config = root.join(".obsidian").join("app.json");

    let mut config = VaultConfig::default_for_root(root);

    if obsidian_config.exists() {
        if let Ok(content) = std::fs::read_to_string(&obsidian_config) {
            import_obsidian_config(&mut config, &content);
        }
    }

    import_oxidian_features(&mut config);

    config
}

/// Reads relevant fields from Obsidian's `app.json` and applies them to the config.
fn import_obsidian_config(config: &mut VaultConfig, app_json: &str) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(app_json) else {
        return;
    };

    if let Some(attachment_folder) = value.get("attachmentFolderPath").and_then(|v| v.as_str()) {
        config.templates_dir = config.root.join(attachment_folder);
    }

    // Obsidian's newFileFolderPath is intentionally ignored until Oxidian has a
    // first-class default note folder setting.
}

/// Lee el campo `"features"` de `.oxidian/config.json` y aplica los valores
/// encontrados sobre los defaults de `OxidianFeatureFlags`.
fn import_oxidian_features(config: &mut VaultConfig) {
    let oxidian_config_path = config.root.join(".oxidian").join("config.json");
    let Ok(content) = std::fs::read_to_string(&oxidian_config_path) else {
        return; // Sin config.json — se usan defaults
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
        log::warn!("Oxidian: .oxidian/config.json no es JSON válido");
        return;
    };
    let Some(features) = value.get("features") else {
        return; // Sin sección "features" — se usan defaults
    };

    if let Some(v) = features.get("backlinks_panel").and_then(|v| v.as_bool()) {
        config.features.backlinks_panel = v;
    }
    if let Some(v) = features.get("daily_notes_panel").and_then(|v| v.as_bool()) {
        config.features.daily_notes_panel = v;
    }
    if let Some(v) = features.get("frontmatter_panel").and_then(|v| v.as_bool()) {
        config.features.frontmatter_panel = v;
    }
    if let Some(v) = features.get("vim_mode").and_then(|v| v.as_bool()) {
        config.features.vim_mode = v;
    }
    if let Some(v) = features.get("git_panel").and_then(|v| v.as_bool()) {
        config.features.git_panel = v;
    }
}

// OXIDIAN END

// OXIDIAN BEGIN — note path helpers

fn note_path_for_target(vault_root: &Path, target: &str) -> Option<PathBuf> {
    let normalized = target.trim().trim_end_matches(".md").replace('\\', "/");
    let relative = Path::new(&normalized);

    if normalized.is_empty() || relative.is_absolute() {
        return None;
    }

    if relative.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return None;
    }

    Some(vault_root.join(relative).with_extension("md"))
}

// OXIDIAN END

// OXIDIAN BEGIN — GPUI global registration

/// GPUI global holding the active vault index, if any.
pub struct ActiveVault(pub Option<Entity<VaultIndex>>);

impl Global for ActiveVault {}

/// Registers Oxidian vault integration with the GPUI App.
/// Call this from `zed/src/main.rs` during initialization.
pub fn init(fs: Arc<dyn Fs>, cx: &mut App) {
    cx.set_global(ActiveVault(None));
    cx.set_global(oxidian_core::MarksmanBinaryPath(None));

    cx.set_global(WikiLinkResolver(Arc::new(move |target, _window, cx| {
        let active_vault = cx.try_global::<ActiveVault>().and_then(|av| av.0.clone())?;
        let note_id = active_vault.read(cx).resolve_wiki_link(target);
        if let Some(note_id) = note_id {
            active_vault.read(cx).resolve_note(&note_id).cloned()
        } else {
            // Note does not exist yet! Create it under the vault root
            let vault_root = active_vault.read(cx).config.root.clone();
            let note_path = note_path_for_target(&vault_root, target)?;
            if !note_path.exists() {
                if let Some(parent) = note_path.parent() {
                    if let Err(err) = std::fs::create_dir_all(parent) {
                        log::error!("Oxidian: failed to create note directory {parent:?}: {err}");
                        return None;
                    }
                }
                let title = note_path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or(target);
                if let Err(err) = std::fs::write(&note_path, format!("# {title}\n\n")) {
                    log::error!("Oxidian: failed to create note {note_path:?}: {err}");
                    return None;
                }
            }
            Some(note_path)
        }
    })));

    cx.observe_new(move |workspace: &mut workspace::Workspace, _window, cx| {
        let Some(worktree) = workspace.visible_worktrees(cx).next() else {
            return;
        };
        let root_path = worktree.read(cx).abs_path().to_path_buf();

        if is_vault_root(&root_path) {
            let config = load_vault_config(root_path);
            cx.set_global(oxidian_core::MarksmanBinaryPath(
                config.marksman_binary.clone(),
            ));
            let index = cx.new(|cx| VaultIndex::new(config, fs.clone(), cx));

            // Subscribe to project events to re-index notes in real-time!
            let project = workspace.project().clone();
            let index_weak = index.downgrade();
            cx.subscribe(&project, move |_, _, event, cx| {
                if let Some(index) = index_weak.upgrade() {
                    match event {
                        project::Event::WorktreeUpdatedEntries(worktree_id, entries) => {
                            index.update(cx, |vault, cx| {
                                vault.handle_worktree_updated_entries(*worktree_id, entries, cx);
                            });
                        }
                        _ => {}
                    }
                }
            })
            .detach();

            cx.update_global::<ActiveVault, _>(|active, _| {
                active.0 = Some(index);
            });
        }
    })
    .detach();
}

// OXIDIAN END

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_wiki_links_simple() {
        let text = "See [[My Note]] for details.\nAlso [[Another Note|Display]] here.";
        let links = extract_wiki_links_from_text(text);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].0, 0);
        assert_eq!(links[0].1.target.as_ref(), "My Note");
        assert_eq!(links[1].0, 1);
        assert_eq!(links[1].1.target.as_ref(), "Another Note");
        assert_eq!(links[1].1.alias.as_deref(), Some("Display"));
    }

    #[test]
    fn test_extract_wiki_links_multiple_on_same_line() {
        let text = "[[Note A]] and [[Note B]] and [[Note C]]";
        let links = extract_wiki_links_from_text(text);
        assert_eq!(links.len(), 3);
    }

    #[test]
    fn test_extract_wiki_links_with_heading() {
        let text = "See [[My Note#Introduction]] for context.";
        let links = extract_wiki_links_from_text(text);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].1.heading.as_deref(), Some("Introduction"));
    }

    #[test]
    fn test_extract_tags_inline_array() {
        let content = "---\ntags: [rust, zed, notes]\n---\n# My Note";
        let tags = extract_tags_from_frontmatter(content);
        assert_eq!(tags, vec!["rust", "zed", "notes"]);
    }

    #[test]
    fn test_extract_tags_block_array() {
        let content = "---\ntags:\n  - rust\n  - zed\n---\n# My Note";
        let tags = extract_tags_from_frontmatter(content);
        assert_eq!(tags, vec!["rust", "zed"]);
    }

    #[test]
    fn test_extract_tags_scalar() {
        let content = "---\ntags: rust\n---\n# My Note";
        let tags = extract_tags_from_frontmatter(content);
        assert_eq!(tags, vec!["rust"]);
    }

    #[test]
    fn test_extract_title_from_heading() {
        let content = "# My Great Note\n\nSome content here.";
        assert_eq!(extract_title(content), Some("My Great Note".to_owned()));
    }

    #[test]
    fn test_is_vault_root_oxidian() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".oxidian")).unwrap();
        assert!(is_vault_root(dir.path()));
    }

    #[test]
    fn test_is_vault_root_obsidian() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".obsidian")).unwrap();
        assert!(is_vault_root(dir.path()));
    }

    #[test]
    fn test_is_not_vault_root() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!is_vault_root(dir.path()));
    }

    #[test]
    fn test_note_path_for_target_accepts_nested_notes() {
        let root = Path::new("/vault");
        assert_eq!(
            note_path_for_target(root, "projects/alpha"),
            Some(PathBuf::from("/vault/projects/alpha.md"))
        );
    }

    #[test]
    fn test_note_path_for_target_rejects_escape_paths() {
        let root = Path::new("/vault");
        assert_eq!(note_path_for_target(root, "../secret"), None);
        assert_eq!(note_path_for_target(root, "/tmp/secret"), None);
    }
}
