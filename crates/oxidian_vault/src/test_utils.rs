use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// A helper struct for managing a temporary silo during tests.
pub struct TestSilo {
    temp_dir: TempDir,
}

impl TestSilo {
    /// Creates a new temporary silo with the `.oxidian` directory marker.
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let oxidian_dir = temp_dir.path().join(".oxidian");
        fs::create_dir(&oxidian_dir).expect("failed to create .oxidian directory");

        // Write a minimal config file to avoid warnings
        let config_json = r#"{"features": {"backlinks_panel": true, "daily_notes_panel": true, "frontmatter_panel": true}}"#;
        fs::write(oxidian_dir.join("config.json"), config_json)
            .expect("failed to write config.json");

        Self { temp_dir }
    }

    /// Returns the absolute path to the silo root.
    pub fn path(&self) -> PathBuf {
        self.temp_dir.path().to_path_buf()
    }

    /// Writes a note with the given relative path and content.
    /// Returns the absolute path of the written file.
    pub fn write_note(&self, relative_path: impl AsRef<Path>, content: &str) -> PathBuf {
        let path = self.path().join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create parent directories");
        }
        fs::write(&path, content).expect("failed to write note file");
        path
    }

    /// Deletes the note at the given relative path.
    pub fn delete_note(&self, relative_path: impl AsRef<Path>) {
        let path = self.path().join(relative_path);
        if path.exists() {
            fs::remove_file(path).expect("failed to delete note file");
        }
    }
}

// TODO(#oxidian-tests): Add `init_test_env` and `wait_for_indexing` once
// GPUI integration tests are introduced (requires theme + theme_settings dev-deps).
