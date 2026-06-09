use gpui::{ScrollHandle, prelude::*};
use oxidian_core::OxidianFeatureFlags;
use oxidian_vault::{ActiveVault, save_oxidian_features_for_vault};
use ui::{Checkbox, ToggleState, prelude::*};

use crate::SettingsWindow;

pub(crate) fn render_oxidian_panels_page(
    _settings_window: &SettingsWindow,
    scroll_handle: &ScrollHandle,
    _window: &mut Window,
    cx: &mut Context<SettingsWindow>,
) -> AnyElement {
    let mut features = OxidianFeatureFlags::default();
    let mut has_vault = false;
    let mut vault_root = None;

    if let Some(active_vault) = cx.global::<ActiveVault>().0.clone() {
        let vault = active_vault.read(cx);
        features = vault.config.features.clone();
        has_vault = true;
        vault_root = Some(vault.config.root.clone());
    }

    let toggle = |name: &'static str,
                  value: bool,
                  update_fn: fn(&mut OxidianFeatureFlags, bool)| {
        let vault_root = vault_root.clone();
        let features_clone = features.clone();

        h_flex()
            .gap_2()
            .items_center()
            .child(
                Checkbox::new(SharedString::from(name), ToggleState::from(value))
                    .disabled(!has_vault)
                    .on_click(cx.listener(move |_, state, _, cx| {
                        if let Some(root) = vault_root.clone() {
                            let mut new_features = features_clone.clone();
                            update_fn(&mut new_features, *state == ToggleState::Selected);
                            if let Err(e) = save_oxidian_features_for_vault(&root, &new_features) {
                                log::error!("Failed to save oxidian features: {}", e);
                            } else {
                                // Update ActiveVault in memory
                                if let Some(active_vault) = cx.global::<ActiveVault>().0.clone() {
                                    active_vault.update(cx, |vault, cx| {
                                        vault.config.features = new_features.clone();
                                        cx.notify();
                                    });
                                    if cx.has_global::<oxidian_core::ActiveSilo>() {
                                        cx.update_global::<oxidian_core::ActiveSilo, _>(
                                            |silo, _| {
                                                silo.0.features = new_features.clone();
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    })),
            )
            .child(Label::new(name).color(if has_vault {
                Color::Default
            } else {
                Color::Muted
            }))
    };

    v_flex()
        .id("oxidian-panels-page")
        .min_w_0()
        .size_full()
        .pt_2p5()
        .px_8()
        .pb_16()
        .gap_4()
        .overflow_y_scroll()
        .track_scroll(scroll_handle)
        .when(!has_vault, |this| {
            this.child(Label::new("Open a vault to change these settings.").color(Color::Warning))
        })
        .child(toggle(
            "Backlinks Panel",
            features.backlinks_panel,
            |f, v| f.backlinks_panel = v,
        ))
        .child(toggle(
            "Daily Notes Panel",
            features.daily_notes_panel,
            |f, v| f.daily_notes_panel = v,
        ))
        .child(toggle(
            "Frontmatter Panel",
            features.frontmatter_panel,
            |f, v| f.frontmatter_panel = v,
        ))
        .child(toggle(
            "Panels Default Flexible",
            features.panels_default_flexible,
            |f, v| f.panels_default_flexible = v,
        ))
        .into_any_element()
}
