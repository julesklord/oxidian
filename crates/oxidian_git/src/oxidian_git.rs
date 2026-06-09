use gpui::{
    App, Context, Empty, EventEmitter, IntoElement, ParentElement, Render, Task, WeakEntity, Window,
};
use oxidian_vault::ActiveVault;
use smol::process::Command;
use std::path::{Path, PathBuf};
use ui::prelude::*;
use workspace::{HideStatusItem, StatusItemView, Workspace, item::ItemHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatus {
    None,
    Clean,
    Conflicts,
}

pub struct GitStatusItem {
    _workspace: WeakEntity<Workspace>,
    status: GitStatus,
    _monitor_task: Option<Task<()>>,
}

impl GitStatusItem {
    pub fn new(workspace: WeakEntity<Workspace>, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            _workspace: workspace,
            status: GitStatus::None,
            _monitor_task: None,
        };
        this.start_monitoring(cx);
        this
    }

    fn start_monitoring(&mut self, cx: &mut Context<Self>) {
        self._monitor_task = Some(cx.spawn(async move |this, cx| {
            loop {
                let vault_root = cx.update(|cx| -> Option<PathBuf> {
                    let active_vault =
                        cx.try_global::<ActiveVault>().and_then(|av| av.0.clone())?;
                    let vault = active_vault.read(cx);
                    Some(vault.config.root.clone())
                });

                if let Some(vault_root) = vault_root {
                    let has_conflicts = check_git_conflicts(&vault_root).await.unwrap_or(false);
                    let status = if has_conflicts {
                        GitStatus::Conflicts
                    } else {
                        GitStatus::Clean
                    };

                    this.update(cx, |this, cx| {
                        if this.status != status {
                            this.status = status;
                            cx.notify();
                        }
                    })
                    .ok();
                } else {
                    this.update(cx, |this, cx| {
                        if this.status != GitStatus::None {
                            this.status = GitStatus::None;
                            cx.notify();
                        }
                    })
                    .ok();
                }

                cx.background_executor()
                    .timer(std::time::Duration::from_secs(5))
                    .await;
            }
        }));
    }
}

impl Render for GitStatusItem {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.status {
            GitStatus::None => Empty.into_any_element(),
            GitStatus::Clean => div()
                .flex()
                .items_center()
                .gap_1()
                .px_2()
                .py_0p5()
                .rounded_md()
                .bg(cx.theme().colors().element_hover)
                .child(
                    Icon::new(IconName::Check)
                        .size(IconSize::XSmall)
                        .color(Color::Success),
                )
                .child(Label::new("Git: Clean").size(LabelSize::XSmall))
                .into_any_element(),
            GitStatus::Conflicts => div()
                .flex()
                .items_center()
                .gap_1()
                .px_2()
                .py_0p5()
                .rounded_md()
                .bg(cx.theme().status().error_background)
                .border_1()
                .border_color(cx.theme().status().error_border)
                .child(
                    Icon::new(IconName::Warning)
                        .size(IconSize::XSmall)
                        .color(Color::Error),
                )
                .child(
                    Label::new("Git: Conflicts")
                        .size(LabelSize::XSmall)
                        .color(Color::Error),
                )
                .into_any_element(),
        }
    }
}

impl EventEmitter<workspace::ToolbarItemEvent> for GitStatusItem {}

impl StatusItemView for GitStatusItem {
    fn set_active_pane_item(
        &mut self,
        _active_pane_item: Option<&dyn ItemHandle>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    fn hide_setting(&self, _: &App) -> Option<HideStatusItem> {
        None
    }
}

/// Check if repository contains unmerged files (conflicts)
pub async fn check_git_conflicts(vault_path: &Path) -> anyhow::Result<bool> {
    let mut cmd = Command::new("git");
    cmd.args(["ls-files", "--unmerged"]);
    cmd.current_dir(vault_path);

    let output = cmd.output().await?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(!stdout.trim().is_empty())
}

/// Automatically add and commit all changes to the vault repository
pub async fn git_auto_commit(vault_path: &Path, message: &str) -> anyhow::Result<()> {
    // Stage all markdown files
    let mut add_cmd = Command::new("git");
    add_cmd.args(["add", "."]);
    add_cmd.current_dir(vault_path);
    add_cmd.output().await?;

    // Commit changes
    let mut commit_cmd = Command::new("git");
    commit_cmd.args(["commit", "-m", message]);
    commit_cmd.current_dir(vault_path);
    commit_cmd.output().await?;

    Ok(())
}

/// Fetch updates from remote
pub async fn git_fetch(vault_path: &Path) -> anyhow::Result<()> {
    let mut cmd = Command::new("git");
    cmd.arg("fetch");
    cmd.current_dir(vault_path);
    cmd.output().await?;
    Ok(())
}

/// Pull updates from remote
pub async fn git_pull(vault_path: &Path) -> anyhow::Result<()> {
    let mut cmd = Command::new("git");
    cmd.arg("pull");
    cmd.current_dir(vault_path);
    cmd.output().await?;
    Ok(())
}

/// Push updates to remote
pub async fn git_push(vault_path: &Path) -> anyhow::Result<()> {
    let mut cmd = Command::new("git");
    cmd.arg("push");
    cmd.current_dir(vault_path);
    cmd.output().await?;
    Ok(())
}

/// Registers oxidian_git status bar integration with GPUI Workspace
pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, window, cx| {
        let active_vault = cx.try_global::<ActiveVault>().and_then(|av| av.0.clone());
        if active_vault.is_some() {
            let git_status_view = cx.new(|cx| GitStatusItem::new(workspace.weak_handle(), cx));
            workspace.status_bar().update(cx, |status_bar, cx| {
                if let Some(window) = window {
                    status_bar.add_left_item(git_status_view, window, cx);
                }
            });
        }
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[gpui::test]
    async fn test_check_git_conflicts_no_repo(_cx: &mut gpui::TestAppContext) {
        let dir = tempdir().unwrap();
        // Since it's not a git repository, it should fail or return false.
        let res = check_git_conflicts(dir.path()).await;
        assert!(res.is_err());
    }
}
