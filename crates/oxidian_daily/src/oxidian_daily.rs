use std::collections::HashSet;
use chrono::{Datelike, Local, NaiveDate};
use gpui::{
    actions, div, Action, App, AsyncWindowContext, Context, Entity, EventEmitter, FocusHandle,
    Focusable, FontWeight, InteractiveElement as _, IntoElement, ParentElement, Pixels, Render,
    StatefulInteractiveElement as _, Styled as _, Subscription, WeakEntity, Window,
};
use ui::prelude::*;
use workspace::dock::{DockPosition, Panel, PanelEvent, PanelSizeState};
use workspace::Workspace;
use oxidian_vault::ActiveVault;

actions!(oxidian_daily, [ToggleDailyNotesPanel, OpenTodayNote]);

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _action: &ToggleDailyNotesPanel, window, cx| {
            workspace.toggle_panel_focus::<DailyNotesPanel>(window, cx);
        });

        workspace.register_action(|workspace, _action: &OpenTodayNote, window, cx| {
            let today = Local::now().naive_local().date();
            open_daily_note_for_date(today, workspace.weak_handle(), window, cx);
        });
    })
    .detach();
}

fn convert_obsidian_format_to_chrono(obsidian_format: &str) -> String {
    obsidian_format
        .replace("YYYY", "%Y")
        .replace("YY", "%y")
        .replace("MM", "%m")
        .replace("DD", "%d")
}

pub fn open_daily_note_for_date(
    date: NaiveDate,
    workspace: WeakEntity<Workspace>,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(active_vault) = cx.try_global::<ActiveVault>().and_then(|av| av.0.clone()) else {
        return;
    };

    let vault = active_vault.read(cx);
    let daily_dir = vault.config.daily_notes_dir.clone();
    let format_str = vault.config.daily_notes_format.clone();
    let templates_dir = vault.config.templates_dir.clone();

    let chrono_format = convert_obsidian_format_to_chrono(&format_str);
    let date_str = date.format(&chrono_format).to_string();
    let entry_path = daily_dir.join(format!("{}.md", date_str));

    let template_path = templates_dir.join("daily.md");

    let entry_path_clone = entry_path.clone();
    let date_clone = date;
    let format_str_clone = format_str.clone();

    let create_task = cx.background_spawn(async move {
        if !entry_path_clone.exists() {
            if let Some(parent) = entry_path_clone.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut content = String::new();
            if template_path.exists() {
                if let Ok(tpl) = std::fs::read_to_string(&template_path) {
                    content = tpl;
                }
            }

            if content.is_empty() {
                content = format!("# {}\n\n", date_str);
            } else {
                let now_time = Local::now().format("%H:%M").to_string();
                let yesterday = date_clone.pred_opt().unwrap_or(date_clone);
                let chrono_format = convert_obsidian_format_to_chrono(&format_str_clone);
                let yesterday_str = yesterday.format(&chrono_format).to_string();

                content = content
                    .replace("{{date}}", &date_str)
                    .replace("{{time}}", &now_time)
                    .replace("{{yesterday}}", &format!("[[{yesterday_str}]]"));
            }

            std::fs::write(&entry_path_clone, content)?;
        }
        anyhow::Ok(entry_path_clone)
    });

    window
        .spawn(cx, async move |cx| {
            if let Ok(path) = create_task.await {
                if let Some(workspace) = workspace.upgrade() {
                    let _ = workspace.update_in(cx, |workspace, window, cx| {
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
                        .detach();
                    });
                }
            }
            anyhow::Ok(())
        })
        .detach();
}

pub struct DailyNotesPanel {
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    current_month: u32,
    current_year: i32,
    selected_date: NaiveDate,
    notes_with_daily_dates: HashSet<NaiveDate>,
    position: DockPosition,
    active: bool,
    zoomed: bool,
    flexible: bool,
    default_size: Pixels,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<PanelEvent> for DailyNotesPanel {}

impl DailyNotesPanel {
    pub fn new(workspace: Entity<Workspace>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let today = Local::now().naive_local().date();
        let mut this = Self {
            workspace: workspace.downgrade(),
            focus_handle,
            current_month: today.month(),
            current_year: today.year(),
            selected_date: today,
            notes_with_daily_dates: HashSet::new(),
            position: DockPosition::Right,
            active: false,
            zoomed: false,
            flexible: true,
            default_size: Pixels::from(280.0),
            _subscriptions: Vec::new(),
        };

        this._subscriptions.push(cx.subscribe(&workspace, move |this, _, event, cx| {
            if let workspace::Event::ActiveItemChanged = event {
                this.update_existing_notes(cx);
            }
        }));

        // Defer initial update to avoid borrowing workspace while it is updating
        let handle = cx.weak_entity();
        cx.defer(move |cx| {
            handle.update(cx, |this, cx| {
                this.update_existing_notes(cx);
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

    fn update_existing_notes(&mut self, cx: &mut Context<Self>) {
        let Some(active_vault) = cx.try_global::<ActiveVault>().and_then(|av| av.0.clone()) else {
            return;
        };

        let vault = active_vault.read(cx);
        let daily_dir = vault.config.daily_notes_dir.clone();
        let format_str = vault.config.daily_notes_format.clone();

        cx.spawn(async move |this, cx| {
            let existing_dates = cx.background_spawn(async move {
                let mut dates: HashSet<NaiveDate> = HashSet::new();
                let chrono_format = convert_obsidian_format_to_chrono(&format_str);
                if let Ok(entries) = std::fs::read_dir(daily_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().map_or(false, |ext| ext == "md") {
                            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                if let Ok(parsed_date) = NaiveDate::parse_from_str(stem, &chrono_format) {
                                    dates.insert(parsed_date);
                                }
                            }
                        }
                    }
                }
                dates
            })
            .await;

            this.update(cx, |this, cx| {
                this.notes_with_daily_dates = existing_dates;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn prev_month(&mut self, cx: &mut Context<Self>) {
        if self.current_month == 1 {
            self.current_month = 12;
            self.current_year -= 1;
        } else {
            self.current_month -= 1;
        }
        self.update_existing_notes(cx);
    }

    fn next_month(&mut self, cx: &mut Context<Self>) {
        if self.current_month == 12 {
            self.current_month = 1;
            self.current_year += 1;
        } else {
            self.current_month += 1;
        }
        self.update_existing_notes(cx);
    }

    fn select_day(&mut self, date: NaiveDate, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_date = date;
        cx.notify();
        open_daily_note_for_date(date, self.workspace.clone(), window, cx);
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

impl Render for DailyNotesPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let month_name = match self.current_month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        };

        let month_title = format!("{} {}", month_name, self.current_year);

        let calendar_header = div()
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
                        Label::new("Daily Notes")
                            .weight(FontWeight::BOLD)
                            .color(Color::Default)
                    )
            );

        let navigation_header = div()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .py_1p5()
            .child(
                Label::new(month_title)
                    .weight(FontWeight::SEMIBOLD)
                    .color(Color::Default)
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        IconButton::new("prev-month", IconName::ChevronLeft)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.prev_month(cx);
                            }))
                    )
                    .child(
                        IconButton::new("next-month", IconName::ChevronRight)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.next_month(cx);
                            }))
                    )
            );

        // Generate grid cells
        let first_day = NaiveDate::from_ymd_opt(self.current_year, self.current_month, 1).unwrap();
        let weekday_num = first_day.weekday().number_from_monday(); // 1..=7
        let prev_month_days = weekday_num - 1;

        let (prev_year, prev_month) = if self.current_month == 1 {
            (self.current_year - 1, 12)
        } else {
            (self.current_year, self.current_month - 1)
        };
        let prev_month_len = days_in_month(prev_year, prev_month);

        let mut cells = Vec::new();

        for i in (0..prev_month_days).rev() {
            let day = prev_month_len - i;
            let date = NaiveDate::from_ymd_opt(prev_year, prev_month, day).unwrap();
            cells.push((date, false));
        }

        let current_month_len = days_in_month(self.current_year, self.current_month);
        for day in 1..=current_month_len {
            let date = NaiveDate::from_ymd_opt(self.current_year, self.current_month, day).unwrap();
            cells.push((date, true));
        }

        let remaining = 42 - cells.len();
        let (next_year, next_month) = if self.current_month == 12 {
            (self.current_year + 1, 1)
        } else {
            (self.current_year, self.current_month + 1)
        };
        for day in 1..=remaining {
            let date = NaiveDate::from_ymd_opt(next_year, next_month, day as u32).unwrap();
            cells.push((date, false));
        }

        let weekdays_header = div()
            .grid_cols(7)
            .gap_1()
            .px_3()
            .py_1()
            .children(["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"].iter().map(|day| {
                div()
                    .flex()
                    .justify_center()
                    .child(
                        Label::new(*day)
                            .size(LabelSize::XSmall)
                            .color(Color::Muted)
                    )
            }));

        let today = Local::now().naive_local().date();

        let cells_views = cells.into_iter().enumerate().map(|(idx, (date, is_current))| {
            let has_note = self.notes_with_daily_dates.contains(&date);
            let is_today = date == today;
            let is_selected = date == self.selected_date;

            div()
                .id(("day", idx))
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .aspect_ratio(1.0)
                .rounded_md()
                .cursor_pointer()
                .when(!is_current, |s| s.opacity(0.4))
                .when(is_today, |s| s.border_1().border_color(cx.theme().colors().element_active))
                .when(is_selected, |s| s.bg(cx.theme().colors().element_active))
                .when(!is_selected && is_current, |s| s.hover(|style| style.bg(cx.theme().colors().element_hover)))
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.select_day(date, window, cx);
                }))
                .child(
                    Label::new(format!("{}", date.day()))
                        .size(LabelSize::Small)
                        .weight(if has_note || is_selected { FontWeight::BOLD } else { FontWeight::NORMAL })
                        .color(if is_selected { Color::Default } else if has_note { Color::Default } else { Color::Muted })
                )
                .when(has_note, |this| {
                    this.child(
                        div()
                            .w(px(3.0))
                            .h(px(3.0))
                            .rounded_full()
                            .bg(cx.theme().colors().element_active)
                    )
                })
        });

        let grid = div()
            .id("calendar-grid")
            .grid_cols(7)
            .gap_1()
            .px_3()
            .py_1()
            .children(cells_views);

        div()
            .id("daily-notes-panel-root")
            .flex()
            .flex_col()
            .h_full()
            .bg(cx.theme().colors().panel_background)
            .track_focus(&self.focus_handle(cx))
            .child(calendar_header)
            .child(navigation_header)
            .child(weekdays_header)
            .child(grid)
    }
}

impl Panel for DailyNotesPanel {
    fn persistent_name() -> &'static str {
        "DailyNotesPanel"
    }

    fn panel_key() -> &'static str {
        "DailyNotesPanel"
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
        Some("Daily Notes Panel")
    }

    fn toggle_action(&self) -> Box<dyn Action> {
        ToggleDailyNotesPanel.boxed_clone()
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
        110
    }
}

impl Focusable for DailyNotesPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_static_properties() {
        assert_eq!(DailyNotesPanel::persistent_name(), "DailyNotesPanel");
        assert_eq!(DailyNotesPanel::panel_key(), "DailyNotesPanel");
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2026, 1), 31);
        assert_eq!(days_in_month(2026, 2), 28);
        assert_eq!(days_in_month(2024, 2), 29); // Bisiesto
        assert_eq!(days_in_month(2026, 4), 30);
    }

    #[test]
    fn test_obsidian_format_conversion() {
        assert_eq!(convert_obsidian_format_to_chrono("YYYY-MM-DD"), "%Y-%m-%d");
        assert_eq!(convert_obsidian_format_to_chrono("YYYY/MM/DD"), "%Y/%m/%d");
    }
}
