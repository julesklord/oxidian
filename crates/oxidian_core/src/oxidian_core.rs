use std::path::PathBuf;
use std::sync::Arc;

/// A note's stable identifier — its path relative to the vault root, without extension.
/// For example, a note at `<vault>/projects/alpha.md` has id `projects/alpha`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NoteId(pub Arc<str>);

impl NoteId {
    pub fn from_relative_path(path: &str) -> Self {
        let without_ext = path.strip_suffix(".md").unwrap_or(path);
        Self(Arc::from(without_ext))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NoteId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A parsed `[[wiki-link]]`, including optional alias and heading anchor.
///
/// Examples:
/// - `[[My Note]]`              → target="My Note", alias=None, heading=None
/// - `[[My Note|Display Name]]` → target="My Note", alias=Some("Display Name"), heading=None
/// - `[[My Note#Section]]`      → target="My Note", alias=None, heading=Some("Section")
/// - `[[My Note#Section|Name]]` → target="My Note", alias=Some("Name"), heading=Some("Section")
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WikiLink {
    pub target: Arc<str>,
    pub alias: Option<Arc<str>>,
    pub heading: Option<Arc<str>>,
}

impl WikiLink {
    /// Parses the inner content of a wiki-link (without the surrounding `[[` and `]]`).
    pub fn parse(inner: &str) -> Self {
        let (target_and_heading, alias) = if let Some(pipe_pos) = inner.find('|') {
            let alias = inner[pipe_pos + 1..].trim();
            (&inner[..pipe_pos], Some(Arc::from(alias)))
        } else {
            (inner, None)
        };

        let (target, heading) = if let Some(hash_pos) = target_and_heading.find('#') {
            let heading = target_and_heading[hash_pos + 1..].trim();
            (&target_and_heading[..hash_pos], Some(Arc::from(heading)))
        } else {
            (target_and_heading, None)
        };

        Self {
            target: Arc::from(target.trim()),
            alias,
            heading,
        }
    }

    /// Returns the display text for this link: the alias if present, otherwise the target.
    pub fn display_text(&self) -> &str {
        if let Some(alias) = &self.alias {
            alias
        } else {
            &self.target
        }
    }
}

/// Vault-level configuration, stored in `.oxidian/config.json` at the vault root.
#[derive(Clone, Debug)]
pub struct VaultConfig {
    /// Absolute path to the vault root directory.
    pub root: PathBuf,
    /// Path to the templates directory, relative to vault root.
    pub templates_dir: PathBuf,
    /// Path where daily notes are stored, relative to vault root.
    pub daily_notes_dir: PathBuf,
    /// Date format string for daily notes filenames (e.g. `"YYYY-MM-DD"`).
    pub daily_notes_format: Arc<str>,
    /// Separator character used between target and alias in wiki-links (default: `|`).
    pub wiki_link_alias_separator: char,
    /// Optional path to the Marksman binary for LSP integration.
    pub marksman_binary: Option<PathBuf>,
    /// Whether telemetry and Zed cloud features are disabled.
    pub disable_cloud_features: bool,
}

impl VaultConfig {
    pub fn default_for_root(root: PathBuf) -> Self {
        Self {
            templates_dir: root.join("_templates"),
            daily_notes_dir: root.join("Daily"),
            daily_notes_format: Arc::from("YYYY-MM-DD"),
            wiki_link_alias_separator: '|',
            marksman_binary: None,
            disable_cloud_features: true,
            root,
        }
    }
}

/// A reference from one note to another, either via wiki-link or standard Markdown link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoteRef {
    WikiLink(WikiLink),
    MarkdownLink { url: Arc<str>, title: Option<Arc<str>> },
}

/// Helper to check if a character column is inside a `[[wiki-link]]` on a given line.
/// If yes, returns the inner content of the wiki-link.
pub fn find_wiki_link_at_column(line: &str, col: usize) -> Option<String> {
    let chars: Vec<char> = line.chars().collect();
    
    // Look backwards from col for "[["
    let mut open_idx = None;
    if col >= 2 {
        for i in (0..=col.min(chars.len() - 2)).rev() {
            if chars[i] == '[' && chars[i+1] == '[' {
                open_idx = Some(i + 2);
                break;
            }
            if chars[i] == ']' && chars[i+1] == ']' && i + 2 <= col {
                return None;
            }
        }
    }

    let Some(start) = open_idx else { return None; };

    // Look forwards from start for "]]"
    let mut close_idx = None;
    for i in start..chars.len() {
        if i + 2 <= chars.len() && chars[i] == ']' && chars[i+1] == ']' {
            close_idx = Some(i);
            break;
        }
        if i + 2 <= chars.len() && chars[i] == '[' && chars[i+1] == '[' {
            return None;
        }
    }

    let Some(end) = close_idx else { return None; };

    if col >= start - 2 && col <= end + 2 {
        let content: String = chars[start..end].iter().collect();
        Some(content)
    } else {
        None
    }
}

/// A thread-safe global resolver for wiki-links.
/// Registered by `oxidian_vault` and called by `editor` to navigate to notes.
pub struct WikiLinkResolver(
    pub Arc<dyn Fn(&str, &mut gpui::Window, &mut gpui::App) -> Option<PathBuf> + Send + Sync>,
);

impl gpui::Global for WikiLinkResolver {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wiki_link_simple() {
        let link = WikiLink::parse("My Note");
        assert_eq!(link.target.as_ref(), "My Note");
        assert!(link.alias.is_none());
        assert!(link.heading.is_none());
    }

    #[test]
    fn test_wiki_link_with_alias() {
        let link = WikiLink::parse("My Note|Display Name");
        assert_eq!(link.target.as_ref(), "My Note");
        assert_eq!(link.alias.as_deref(), Some("Display Name"));
        assert_eq!(link.display_text(), "Display Name");
    }

    #[test]
    fn test_wiki_link_with_heading() {
        let link = WikiLink::parse("My Note#Section Title");
        assert_eq!(link.target.as_ref(), "My Note");
        assert_eq!(link.heading.as_deref(), Some("Section Title"));
        assert!(link.alias.is_none());
    }

    #[test]
    fn test_wiki_link_full() {
        let link = WikiLink::parse("My Note#Section|Label");
        assert_eq!(link.target.as_ref(), "My Note");
        assert_eq!(link.heading.as_deref(), Some("Section"));
        assert_eq!(link.alias.as_deref(), Some("Label"));
    }

    #[test]
    fn test_note_id_strips_md_extension() {
        let id = NoteId::from_relative_path("projects/alpha.md");
        assert_eq!(id.as_str(), "projects/alpha");
    }

    #[test]
    fn test_note_id_no_extension() {
        let id = NoteId::from_relative_path("projects/alpha");
        assert_eq!(id.as_str(), "projects/alpha");
    }

    #[test]
    fn test_find_wiki_link_at_column() {
        let line = "See [[My Note|Display]] here";
        assert_eq!(find_wiki_link_at_column(line, 8), Some("My Note|Display".to_owned())); // inside My Note
        assert_eq!(find_wiki_link_at_column(line, 4), Some("My Note|Display".to_owned())); // at [[
        assert_eq!(find_wiki_link_at_column(line, 22), Some("My Note|Display".to_owned())); // at ]]
        assert_eq!(find_wiki_link_at_column(line, 3), None); // before [[
        assert_eq!(find_wiki_link_at_column(line, 24), None); // after ]]
    }
}
