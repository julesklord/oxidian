use gpui::{
    Action, App, AsyncWindowContext, Context, Entity, EventEmitter, FocusHandle, Focusable,
    FontWeight, InteractiveElement as _, IntoElement, ParentElement, Pixels, Render,
    StatefulInteractiveElement as _, Styled as _, Subscription, WeakEntity, Window, actions, div,
};
use oxidian_core::NoteId;
use oxidian_vault::{ActiveVault, VaultDatabase, VaultEvent, VaultIndex};
use std::path::PathBuf;
use std::sync::Arc;
use ui::prelude::*;
use workspace::Workspace;
use workspace::dock::{DockPosition, Panel, PanelEvent, PanelSizeState};

actions!(oxidian_backlinks, [ToggleBacklinksPanel]);

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _action: &ToggleBacklinksPanel, window, cx| {
            workspace.toggle_panel_focus::<BacklinksPanel>(window, cx);
        });
    })
    .detach();
}

pub struct Backlink {
    pub note_id: NoteId,
    pub path: PathBuf,
    pub line: usize,
    pub snippet: String,
}

pub struct BacklinksPanel {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    active_note: Option<NoteId>,
    backlinks: Vec<Backlink>,
    position: DockPosition,
    active: bool,
    zoomed: bool,
    flexible: bool,
    default_size: Pixels,
    _subscriptions: Vec<Subscription>,
    _vault_subscription: Option<(Entity<VaultIndex>, Subscription)>,
}

impl EventEmitter<PanelEvent> for BacklinksPanel {}

impl BacklinksPanel {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let mut this = Self {
            workspace: workspace.downgrade(),
            focus_handle,
            active_note: None,
            backlinks: Vec::new(),
            position: DockPosition::Right,
            active: false,
            zoomed: false,
            flexible: true,
            default_size: Pixels::from(300.0),
            _subscriptions: Vec::new(),
            _vault_subscription: None,
        };

        this._subscriptions
            .push(cx.subscribe(&workspace, move |this, _, event, cx| {
                if let workspace::Event::ActiveItemChanged = event {
                    this.active_item_changed(cx);
                }
            }));

        // Defer initial active item changed to avoid borrowing workspace while it is updating
        let handle = cx.weak_entity();
        cx.defer(move |cx| {
            handle
                .update(cx, |this, cx| {
                    this.active_item_changed(cx);
                })
                .ok();
        });

        this
    }

    pub async fn load(
        workspace: WeakEntity<Workspace>,
        mut cx: AsyncWindowContext,
    ) -> anyhow::Result<Entity<Self>> {
        workspace.update_in(&mut cx, |_, _window, cx| {
            let workspace_view = cx.entity();
            cx.new(|cx| Self::new(workspace_view, cx))
        })
    }

    fn subscribe_to_vault_if_needed(&mut self, cx: &mut Context<Self>) {
        let active_vault = cx.try_global::<ActiveVault>().and_then(|av| av.0.clone());
        if let Some(active_vault) = active_vault {
            if let Some((ref subbed_vault, _)) = self._vault_subscription {
                if subbed_vault == &active_vault {
                    return;
                }
            }
            let subscription = cx.subscribe(&active_vault, move |this, _, event, cx| match event {
                VaultEvent::NoteIndexed(_)
                | VaultEvent::NoteRemoved(_)
                | VaultEvent::InitialScanComplete => {
                    this.active_item_changed(cx);
                }
            });
            self._vault_subscription = Some((active_vault, subscription));
        } else {
            self._vault_subscription = None;
        }
    }

    fn active_item_changed(&mut self, cx: &mut Context<Self>) {
        self.subscribe_to_vault_if_needed(cx);
        let Some(workspace) = self.workspace.upgrade() else {
            return;
        };
        let workspace = workspace.read(cx);
        let Some(active_item) = workspace.active_item(cx) else {
            self.active_note = None;
            self.backlinks.clear();
            cx.notify();
            return;
        };

        if let Some(editor) = active_item.downcast::<editor::Editor>() {
            let multi_buffer = editor.read(cx).buffer().read(cx);
            if let Some(buffer_handle) = multi_buffer.as_singleton() {
                let buffer = buffer_handle.read(cx);
                let file = buffer.file();
                if let Some(file) = file {
                    let path = file.path();
                    if path.extension().is_some_and(|ext| ext == "md") {
                        if let Some(active_vault) =
                            cx.try_global::<ActiveVault>().and_then(|av| av.0.clone())
                        {
                            let vault = active_vault.read(cx);
                            let note_id = NoteId::from_relative_path(path.as_unix_str());
                            self.active_note = Some(note_id.clone());

                            let db = VaultDatabase::global(cx);
                            let mut backlinks = Vec::new();
                            if let Ok(db_links) = db.get_backlinks(note_id.as_str()) {
                                for (from_note, _alias, line_idx) in db_links {
                                    let from_note_id = NoteId(Arc::from(from_note.as_str()));
                                    if let Some(from_path) = vault.resolve_note(&from_note_id) {
                                        let snippet = if let Ok(content) =
                                            std::fs::read_to_string(from_path)
                                        {
                                            content
                                                .lines()
                                                .nth(line_idx as usize)
                                                .unwrap_or("")
                                                .trim()
                                                .to_string()
                                        } else {
                                            "".to_string()
                                        };
                                        backlinks.push(Backlink {
                                            note_id: from_note_id,
                                            path: from_path.clone(),
                                            line: line_idx as usize,
                                            snippet,
                                        });
                                    }
                                }
                            }
                            self.backlinks = backlinks;
                            cx.notify();
                            return;
                        }
                    }
                }
            }
        }

        self.active_note = None;
        self.backlinks.clear();
        cx.notify();
    }

    fn open_backlink(&self, backlink: &Backlink, window: &mut Window, cx: &mut Context<Self>) {
        let Some(workspace) = self.workspace.upgrade() else {
            return;
        };
        let path = backlink.path.clone();
        let line_num = backlink.line;

        let open_task = workspace.update(cx, |workspace, cx| {
            workspace.open_paths(
                vec![path],
                workspace::OpenOptions {
                    visible: Some(workspace::OpenVisible::All),
                    ..Default::default()
                },
                None,
                window,
                cx,
            )
        });

        cx.spawn_in(window, async move |_, cx| {
            let mut results = open_task.await;
            if let Some(Some(Ok(item_handle))) = results.pop() {
                if let Some(editor) = item_handle.downcast::<editor::Editor>() {
                    if let Err(err) = editor.update_in(cx, |editor, window, cx| {
                        let point = text::Point::new(line_num as u32, 0);
                        editor.go_to_singleton_buffer_point(point, window, cx);
                    }) {
                        log::error!("Oxidian: failed to jump to backlink target: {err}");
                    }
                }
            }
        })
        .detach();
    }
}

impl Render for BacklinksPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let panel_header = div()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(Icon::new(IconName::Link).color(Color::Muted))
                    .child(
                        Label::new("Backlinks")
                            .weight(FontWeight::BOLD)
                            .color(Color::Default),
                    ),
            );

        let content = if let Some(ref active_note) = self.active_note {
            if self.backlinks.is_empty() {
                div()
                    .id("empty-backlinks")
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .h_full()
                    .gap_2()
                    .p_4()
                    .child(Label::new("No backlinks found").color(Color::Muted))
                    .child(
                        Label::new(format!("No notes link to [[{}]]", active_note))
                            .color(Color::Muted)
                            .size(LabelSize::Small),
                    )
            } else {
                let backlinks_list = self.backlinks.iter().enumerate().map(|(index, backlink)| {
                    let note_id = backlink.note_id.clone();
                    let line_str = format!("Line {}", backlink.line + 1);
                    let snippet = backlink.snippet.clone();
                    let backlink_clone = Backlink {
                        note_id: backlink.note_id.clone(),
                        path: backlink.path.clone(),
                        line: backlink.line,
                        snippet: backlink.snippet.clone(),
                    };

                    div()
                        .id(("backlink", index))
                        .flex()
                        .flex_col()
                        .gap_1()
                        .p_3()
                        .mx_2()
                        .my_1()
                        .bg(cx.theme().colors().element_background)
                        .hover(|style| style.bg(cx.theme().colors().element_hover))
                        .rounded_md()
                        .border_1()
                        .border_color(cx.theme().colors().border)
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _event, window, cx| {
                            this.open_backlink(&backlink_clone, window, cx);
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    Label::new(note_id.to_string())
                                        .weight(FontWeight::SEMIBOLD)
                                        .color(Color::Default),
                                )
                                .child(
                                    Label::new(line_str)
                                        .size(LabelSize::Small)
                                        .color(Color::Muted),
                                ),
                        )
                        .when(!snippet.is_empty(), |this| {
                            this.child(
                                Label::new(format!("\"{}\"", snippet))
                                    .size(LabelSize::Small)
                                    .color(Color::Muted)
                                    .italic(),
                            )
                        })
                });

                div()
                    .id("backlinks-scroll-container")
                    .flex_1()
                    .overflow_y_scroll()
                    .py_2()
                    .children(backlinks_list)
            }
        } else {
            div()
                .id("no-active-note")
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .h_full()
                .gap_2()
                .p_4()
                .child(Icon::new(IconName::Link).color(Color::Muted))
                .child(Label::new("No active note").color(Color::Muted))
                .child(
                    Label::new("Open a Markdown note to view its backlinks")
                        .color(Color::Muted)
                        .size(LabelSize::Small),
                )
        };

        div()
            .flex()
            .flex_col()
            .h_full()
            .bg(cx.theme().colors().panel_background)
            .track_focus(&self.focus_handle(cx))
            .child(panel_header)
            .child(content)
    }
}

impl Panel for BacklinksPanel {
    fn persistent_name() -> &'static str {
        "BacklinksPanel"
    }

    fn panel_key() -> &'static str {
        "BacklinksPanel"
    }

    fn position(&self, _window: &Window, _cx: &App) -> DockPosition {
        self.position
    }

    fn position_is_valid(&self, _position: DockPosition) -> bool {
        true
    }

    fn set_position(
        &mut self,
        position: DockPosition,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.position = position;
        cx.notify();
    }

    fn default_size(&self, _window: &Window, _cx: &App) -> Pixels {
        self.default_size
    }

    fn initial_size_state(&self, _window: &Window, _cx: &App) -> PanelSizeState {
        PanelSizeState {
            size: None,
            flex: None,
        }
    }

    fn supports_flexible_size(&self) -> bool {
        self.flexible
    }

    fn has_flexible_size(&self, _window: &Window, _cx: &App) -> bool {
        self.flexible
    }

    fn set_flexible_size(&mut self, flexible: bool, _window: &mut Window, cx: &mut Context<Self>) {
        self.flexible = flexible;
        cx.notify();
    }

    fn icon(&self, _window: &Window, _cx: &App) -> Option<IconName> {
        Some(IconName::Link)
    }

    fn icon_tooltip(&self, _window: &Window, _cx: &App) -> Option<&'static str> {
        Some("Backlinks Panel")
    }

    fn toggle_action(&self) -> Box<dyn Action> {
        ToggleBacklinksPanel.boxed_clone()
    }

    fn is_zoomed(&self, _window: &Window, _cx: &App) -> bool {
        self.zoomed
    }

    fn set_zoomed(&mut self, zoomed: bool, _window: &mut Window, cx: &mut Context<Self>) {
        self.zoomed = zoomed;
        cx.notify();
    }

    fn set_active(&mut self, active: bool, _window: &mut Window, cx: &mut Context<Self>) {
        self.active = active;
        cx.notify();
    }

    fn activation_priority(&self) -> u32 {
        100
    }
}

impl Focusable for BacklinksPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backlink_struct() {
        let note_id = NoteId::from_relative_path("notes/my-note");
        let path = PathBuf::from("/vault/notes/my-note.md");
        let backlink = Backlink {
            note_id,
            path: path.clone(),
            line: 42,
            snippet: "See [[my-note]]".to_string(),
        };

        assert_eq!(backlink.note_id.as_str(), "notes/my-note");
        assert_eq!(backlink.path, path);
        assert_eq!(backlink.line, 42);
        assert_eq!(backlink.snippet, "See [[my-note]]");
    }

    #[test]
    fn test_panel_static_properties() {
        assert_eq!(BacklinksPanel::persistent_name(), "BacklinksPanel");
        assert_eq!(BacklinksPanel::panel_key(), "BacklinksPanel");
    }
}
