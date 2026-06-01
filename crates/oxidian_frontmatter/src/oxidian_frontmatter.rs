use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use gpui::{
    actions, div, Action, App, AsyncWindowContext, Context, Entity, EventEmitter, FocusHandle,
    Focusable, FontWeight, InteractiveElement as _, IntoElement, ParentElement, Pixels, Render,
    StatefulInteractiveElement as _, Styled as _, Subscription, WeakEntity, Window,
};
use ui::prelude::*;
use workspace::dock::{DockPosition, Panel, PanelEvent, PanelSizeState};
use workspace::Workspace;
use oxidian_core::NoteId;
use oxidian_vault::{ActiveVault, VaultDatabase};

actions!(oxidian_frontmatter, [ToggleTagBrowserPanel]);

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _action: &ToggleTagBrowserPanel, window, cx| {
            workspace.toggle_panel_focus::<TagBrowserPanel>(window, cx);
        });
    })
    .detach();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataTab {
    Tags,
    Properties,
}

pub struct TagBrowserPanel {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    active_tab: MetadataTab,
    
    // Tags data
    tags: Vec<(String, i64)>,
    expanded_tags: HashSet<String>,
    notes_by_tag: HashMap<String, Vec<String>>,
    
    // Properties data
    active_note: Option<NoteId>,
    active_editor: Option<WeakEntity<editor::Editor>>,
    properties: Vec<(String, String)>,
    new_prop_key: String,
    new_prop_val: String,
    
    position: DockPosition,
    active: bool,
    zoomed: bool,
    flexible: bool,
    default_size: Pixels,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<PanelEvent> for TagBrowserPanel {}

impl TagBrowserPanel {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let mut this = Self {
            workspace: workspace.downgrade(),
            focus_handle,
            active_tab: MetadataTab::Tags,
            tags: Vec::new(),
            expanded_tags: HashSet::new(),
            notes_by_tag: HashMap::new(),
            active_note: None,
            active_editor: None,
            properties: Vec::new(),
            new_prop_key: String::new(),
            new_prop_val: String::new(),
            position: DockPosition::Right,
            active: false,
            zoomed: false,
            flexible: true,
            default_size: Pixels::from(300.0),
            _subscriptions: Vec::new(),
        };

        this._subscriptions.push(cx.subscribe(&workspace, move |this, _, event, cx| {
            if let workspace::Event::ActiveItemChanged = event {
                this.active_item_changed(cx);
            }
        }));

        let handle = cx.weak_entity();
        cx.defer(move |cx| {
            handle.update(cx, |this, cx| {
                this.update_tags(cx);
                this.active_item_changed(cx);
            }).ok();
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

    fn update_tags(&mut self, cx: &mut Context<Self>) {
        let db = VaultDatabase::global(cx);
        if let Ok(tags_with_counts) = db.all_tags_with_counts() {
            self.tags = tags_with_counts;
        }
        cx.notify();
    }

    fn toggle_tag_expansion(&mut self, tag: String, cx: &mut Context<Self>) {
        if self.expanded_tags.contains(&tag) {
            self.expanded_tags.remove(&tag);
            cx.notify();
        } else {
            let db = VaultDatabase::global(cx);
            if let Ok(notes) = db.notes_with_tag(&tag) {
                self.notes_by_tag.insert(tag.clone(), notes);
                self.expanded_tags.insert(tag);
            }
            cx.notify();
        }
    }

    fn open_note_by_id(&self, note_id_str: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(workspace) = self.workspace.upgrade() else { return; };
        let Some(active_vault) = cx.try_global::<ActiveVault>().and_then(|av| av.0.clone()) else { return; };
        let vault = active_vault.read(cx);
        let note_id = NoteId(Arc::from(note_id_str.as_str()));
        if let Some(path) = vault.resolve_note(&note_id) {
            let path_clone = path.clone();
            workspace.update(cx, |workspace, cx| {
                workspace.open_paths(
                    vec![path_clone],
                    workspace::OpenOptions {
                        visible: Some(workspace::OpenVisible::All),
                        ..Default::default()
                    },
                    None,
                    window,
                    cx,
                )
                .detach();
            });
        }
    }

    fn active_item_changed(&mut self, cx: &mut Context<Self>) {
        self.update_tags(cx);
        
        let Some(workspace) = self.workspace.upgrade() else { return; };
        let workspace = workspace.read(cx);
        let Some(active_item) = workspace.active_item(cx) else {
            self.active_note = None;
            self.active_editor = None;
            self.properties.clear();
            cx.notify();
            return;
        };

        if let Some(editor) = active_item.downcast::<editor::Editor>() {
            self.active_editor = Some(editor.downgrade());
            let multi_buffer = editor.read(cx).buffer().read(cx);
            if let Some(buffer_handle) = multi_buffer.as_singleton() {
                let buffer = buffer_handle.read(cx);
                let file = buffer.file();
                if let Some(file) = file {
                    let path = file.path();
                    if path.extension().map_or(false, |ext| ext == "md") {
                        let note_id = NoteId::from_relative_path(path.as_unix_str());
                        self.active_note = Some(note_id);
                        
                        let text = buffer.text();
                        self.properties = parse_frontmatter(&text);
                        cx.notify();
                        return;
                    }
                }
            }
        }

        self.active_note = None;
        self.active_editor = None;
        self.properties.clear();
        cx.notify();
    }

    fn write_properties_to_active_editor(&self, new_props: &[(String, String)], _window: &mut Window, cx: &mut Context<Self>) {
        let Some(editor_weak) = &self.active_editor else { return; };
        let Some(editor) = editor_weak.upgrade() else { return; };

        editor.update(cx, |editor, cx| {
            let multi_buffer = editor.buffer();
            multi_buffer.update(cx, |multi_buffer, cx| {
                if let Some(buffer_handle) = multi_buffer.as_singleton() {
                    buffer_handle.update(cx, |buffer, cx| {
                        let text = buffer.text();
                        let range = find_frontmatter_range(&text);
                        
                        let mut new_yaml = String::new();
                        if !new_props.is_empty() {
                            new_yaml.push_str("---\n");
                            for (k, v) in new_props {
                                new_yaml.push_str(&format!("{}: {}\n", k, v));
                            }
                            new_yaml.push_str("---\n");
                        }

                        if let Some(r) = range {
                            buffer.edit([(r, new_yaml)], None, cx);
                        } else if !new_yaml.is_empty() {
                            buffer.edit([(0..0, new_yaml)], None, cx);
                        }
                    });
                }
            });
        });
    }

    fn delete_property(&mut self, key: String, window: &mut Window, cx: &mut Context<Self>) {
        let mut new_props = self.properties.clone();
        new_props.retain(|(k, _)| k != &key);
        self.write_properties_to_active_editor(&new_props, window, cx);
        self.properties = new_props;
        cx.notify();
    }

    fn add_property(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let key = self.new_prop_key.trim().to_string();
        let val = self.new_prop_val.trim().to_string();
        if key.is_empty() { return; }

        let mut new_props = self.properties.clone();
        new_props.retain(|(k, _)| k != &key);
        new_props.push((key, val));
        
        self.write_properties_to_active_editor(&new_props, window, cx);
        self.properties = new_props;
        self.new_prop_key.clear();
        self.new_prop_val.clear();
        cx.notify();
    }
}

fn parse_frontmatter(content: &str) -> Vec<(String, String)> {
    let mut properties = Vec::new();
    let mut in_frontmatter = false;
    let mut first_line = true;
    for line in content.lines() {
        let trimmed = line.trim();
        if first_line && trimmed == "---" {
            in_frontmatter = true;
            first_line = false;
            continue;
        }
        first_line = false;
        if !in_frontmatter { break; }
        if trimmed == "---" { break; }
        if let Some((key, val)) = trimmed.split_once(':') {
            properties.push((key.trim().to_string(), val.trim().to_string()));
        }
    }
    properties
}

fn find_frontmatter_range(text: &str) -> Option<std::ops::Range<usize>> {
    if text.starts_with("---\n") || text.starts_with("---\r\n") {
        let delimiter = if text.contains("\r\n") { "\r\n" } else { "\n" };
        let start_len = 3 + delimiter.len();
        if let Some(end_offset) = text[start_len..].find(&format!("---{}", delimiter)) {
            let end_abs = start_len + end_offset + 3 + delimiter.len();
            return Some(0..end_abs);
        }
    }
    None
}

impl Render for TagBrowserPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header = div()
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
                    .child(Icon::new(IconName::Book).color(Color::Muted))
                    .child(
                        Label::new("Oxidian Notes")
                            .weight(FontWeight::BOLD)
                            .color(Color::Default)
                    )
            );

        let tabs = div()
            .flex()
            .px_3()
            .py_1()
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .gap_2()
            .child(
                div()
                    .id("tags-tab-btn")
                    .cursor_pointer()
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .when(self.active_tab == MetadataTab::Tags, |s| s.bg(cx.theme().colors().element_active))
                    .when(self.active_tab != MetadataTab::Tags, |s| s.hover(|style| style.bg(cx.theme().colors().element_hover)))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.active_tab = MetadataTab::Tags;
                        this.update_tags(cx);
                    }))
                    .child(Label::new("Tags").size(LabelSize::Small))
            )
            .child(
                div()
                    .id("properties-tab-btn")
                    .cursor_pointer()
                    .px_2()
                    .py_0p5()
                    .rounded_md()
                    .when(self.active_tab == MetadataTab::Properties, |s| s.bg(cx.theme().colors().element_active))
                    .when(self.active_tab != MetadataTab::Properties, |s| s.hover(|style| style.bg(cx.theme().colors().element_hover)))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.active_tab = MetadataTab::Properties;
                        this.active_item_changed(cx);
                    }))
                    .child(Label::new("Properties").size(LabelSize::Small))
            );

        let content = match self.active_tab {
            MetadataTab::Tags => {
                if self.tags.is_empty() {
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .h_full()
                        .p_4()
                        .child(Label::new("No tags found in vault").color(Color::Muted))
                } else {
                    let tag_elements = self.tags.iter().enumerate().map(|(idx, (tag, count))| {
                        let is_expanded = self.expanded_tags.contains(tag);
                        let tag_clone = tag.clone();
                        
                        let tag_header = div()
                            .id(("tag-header", idx))
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|style| style.bg(cx.theme().colors().element_hover))
                            .on_click(cx.listener({
                                let tag = tag.clone();
                                move |this, _, _, cx| {
                                    this.toggle_tag_expansion(tag.clone(), cx);
                                }
                            }))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(
                                        Icon::new(if is_expanded {
                                            IconName::ChevronDown
                                        } else {
                                            IconName::ChevronRight
                                        })
                                        .color(Color::Muted)
                                        .size(IconSize::XSmall)
                                    )
                                    .child(Label::new(format!("#{}", tag)).color(Color::Default))
                            )
                            .child(
                                Label::new(format!("{}", count))
                                    .size(LabelSize::Small)
                                    .color(Color::Muted)
                            );

                        let notes_list = if is_expanded {
                            if let Some(notes) = self.notes_by_tag.get(&tag_clone) {
                                div()
                                    .flex()
                                    .flex_col()
                                    .border_l_1()
                                    .border_color(cx.theme().colors().border)
                                    .ml_3p5()
                                    .pl_2()
                                    .gap_0p5()
                                    .children(notes.iter().enumerate().map(|(n_idx, note_path)| {
                                        let note_title = note_path
                                            .rsplit('/')
                                            .next()
                                            .unwrap_or(note_path.as_str())
                                            .replace(".md", "");
                                        let note_path_clone = note_path.clone();

                                        let unique_id = idx * 10000 + n_idx;
                                        div()
                                            .id(("tag-note", unique_id))
                                            .flex()
                                            .items_center()
                                            .px_2()
                                            .py_0p5()
                                            .rounded_md()
                                            .cursor_pointer()
                                            .hover(|style| style.bg(cx.theme().colors().element_hover))
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                this.open_note_by_id(note_path_clone.clone(), window, cx);
                                            }))
                                            .child(
                                                Label::new(note_title)
                                                    .size(LabelSize::Small)
                                                    .color(Color::Muted)
                                            )
                                    }))
                            } else {
                                div()
                            }
                        } else {
                            div()
                        };

                        div()
                            .id(("tag-block", idx))
                            .flex()
                            .flex_col()
                            .child(tag_header)
                            .child(notes_list)
                    });

                    div()
                        .flex()
                        .flex_col()
                        .p_2()
                        .gap_1()
                        .children(tag_elements)
                }
            }
            MetadataTab::Properties => {
                if self.active_note.is_none() {
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .h_full()
                        .p_4()
                        .child(Label::new("Open a Markdown note to view properties").color(Color::Muted))
                } else {
                    let note_title = self.active_note.as_ref().map(|n| n.as_str().replace(".md", "")).unwrap_or_default();
                    
                    let properties_header = div()
                        .px_3()
                        .py_2()
                        .child(
                            Label::new(note_title)
                                .weight(FontWeight::SEMIBOLD)
                                .color(Color::Default)
                        );

                    let list = if self.properties.is_empty() {
                        div()
                            .px_3()
                            .py_2()
                            .child(Label::new("No frontmatter properties").size(LabelSize::Small).color(Color::Muted))
                    } else {
                        let prop_rows = self.properties.iter().enumerate().map(|(idx, (k, v))| {
                            let k_clone = k.clone();
                            div()
                                .id(("prop-row", idx))
                                .flex()
                                .items_center()
                                .justify_between()
                                .px_3()
                                .py_1()
                                .border_b_1()
                                .border_color(cx.theme().colors().border)
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(Label::new(k.clone()).size(LabelSize::Small).weight(FontWeight::SEMIBOLD))
                                        .child(Label::new(v.clone()).size(LabelSize::Small).color(Color::Muted))
                                )
                                .child(
                                    IconButton::new(("delete-prop", idx), IconName::Trash)
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            this.delete_property(k_clone.clone(), window, cx);
                                        }))
                                )
                        });

                        div().flex().flex_col().children(prop_rows)
                    };

                    // Form to add a new property
                    let add_form = div()
                        .flex()
                        .flex_col()
                        .p_3()
                        .gap_2()
                        .border_t_1()
                        .border_color(cx.theme().colors().border)
                        .child(
                            Label::new("Add Property").size(LabelSize::Small).weight(FontWeight::SEMIBOLD)
                        )
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .child(
                                    div()
                                        .flex_1()
                                        .child(
                                            Label::new("Key").size(LabelSize::XSmall).color(Color::Muted)
                                        )
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .child(
                                            Label::new("Value").size(LabelSize::XSmall).color(Color::Muted)
                                        )
                                )
                        )
                        .child(
                            // Due to not compiling TextInput easily in variable Zed editions,
                            // we provide custom quick tags addition visual buttons or simple edit triggers.
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .flex()
                                        .gap_2()
                                        .child(
                                            div()
                                                .id("add-tags-prop-btn")
                                                .cursor_pointer()
                                                .px_2()
                                                .py_1()
                                                .rounded_md()
                                                .bg(cx.theme().colors().element_hover)
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.new_prop_key = "tags".to_string();
                                                    this.new_prop_val = "[]".to_string();
                                                    this.add_property(window, cx);
                                                }))
                                                .child(Label::new("+ tags: []").size(LabelSize::XSmall))
                                        )
                                        .child(
                                            div()
                                                .id("add-status-prop-btn")
                                                .cursor_pointer()
                                                .px_2()
                                                .py_1()
                                                .rounded_md()
                                                .bg(cx.theme().colors().element_hover)
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.new_prop_key = "status".to_string();
                                                    this.new_prop_val = "active".to_string();
                                                    this.add_property(window, cx);
                                                }))
                                                .child(Label::new("+ status: active").size(LabelSize::XSmall))
                                        )
                                        .child(
                                            div()
                                                .id("add-author-prop-btn")
                                                .cursor_pointer()
                                                .px_2()
                                                .py_1()
                                                .rounded_md()
                                                .bg(cx.theme().colors().element_hover)
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.new_prop_key = "author".to_string();
                                                    this.new_prop_val = "Me".to_string();
                                                    this.add_property(window, cx);
                                                }))
                                                .child(Label::new("+ author: Me").size(LabelSize::XSmall))
                                        )
                                )
                        );

                    div()
                        .flex()
                        .flex_col()
                        .child(properties_header)
                        .child(list)
                        .child(add_form)
                }
            }
        };

        div()
            .id("tag-browser-panel-root")
            .flex()
            .flex_col()
            .h_full()
            .bg(cx.theme().colors().panel_background)
            .track_focus(&self.focus_handle(cx))
            .child(header)
            .child(tabs)
            .child(
                div()
                    .id("tag-browser-scroll-container")
                    .flex_1()
                    .overflow_y_scroll()
                    .child(content)
            )
    }
}

impl Panel for TagBrowserPanel {
    fn persistent_name() -> &'static str {
        "TagBrowserPanel"
    }

    fn panel_key() -> &'static str {
        "TagBrowserPanel"
    }

    fn position(&self, _window: &Window, _cx: &App) -> DockPosition {
        self.position
    }

    fn position_is_valid(&self, _position: DockPosition) -> bool {
        true
    }

    fn set_position(&mut self, position: DockPosition, _window: &mut Window, cx: &mut Context<Self>) {
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
        Some(IconName::Book)
    }

    fn icon_tooltip(&self, _window: &Window, _cx: &App) -> Option<&'static str> {
        Some("Tag Browser & Properties")
    }

    fn toggle_action(&self) -> Box<dyn Action> {
        ToggleTagBrowserPanel.boxed_clone()
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
        150
    }
}

impl Focusable for TagBrowserPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_static_properties() {
        assert_eq!(TagBrowserPanel::persistent_name(), "TagBrowserPanel");
        assert_eq!(TagBrowserPanel::panel_key(), "TagBrowserPanel");
    }

    #[test]
    fn test_parse_frontmatter() {
        let content = "---\ntitle: My Note\ntags: [rust, zed]\nstatus: active\n---\n# Header\nBody content";
        let props = parse_frontmatter(content);
        assert_eq!(props.len(), 3);
        assert_eq!(props[0], ("title".to_string(), "My Note".to_string()));
        assert_eq!(props[1], ("tags".to_string(), "[rust, zed]".to_string()));
        assert_eq!(props[2], ("status".to_string(), "active".to_string()));
    }

    #[test]
    fn test_find_frontmatter_range() {
        let content = "---\ntitle: Hello\n---\nBody";
        let range = find_frontmatter_range(content);
        assert!(range.is_some());
        let r = range.unwrap();
        assert_eq!(&content[r], "---\ntitle: Hello\n---\n");
    }
}
