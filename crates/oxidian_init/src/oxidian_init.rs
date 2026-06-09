use feature_flags::{
    FeatureFlagAppExt, OxidianBacklinksPanel, OxidianDailyNotesPanel, OxidianFrontmatterPanel,
};
use fs::Fs;
use gpui::{App, Context, Window};
use oxidian_vault::load_vault_config;
use settings::KeymapFile;
use std::sync::Arc;
use workspace::Workspace;

/// Feature flags de Oxidian controlables en runtime.
/// En Fase 2 estos valores vendrán de `.oxidian/config.json`.
/// Por ahora son defaults que no duplican init calls de Zed upstream.
///
/// IMPORTANTE: vim, git_ui y markdown_preview ya son inicializados
/// por Zed upstream en main.rs. Esta struct controla SOLO los crates
/// de Oxidian propios. La integración con flags de Zed se completa en Fase 3.
pub struct OxidianFeatures {
    /// Activa el panel de backlinks (oxidian_backlinks).
    pub backlinks_panel: bool,
    /// Activa el panel de notas diarias (oxidian_daily).
    pub daily_notes_panel: bool,
    /// Activa el panel de tags y propiedades (oxidian_frontmatter).
    pub frontmatter_panel: bool,
    /// Habilita soporte para fórmulas LaTeX (math) en Markdown.
    pub enable_math: bool,
    /// Whether new panels should default to flexible sizing
    pub panels_default_flexible: bool,
}

impl Default for OxidianFeatures {
    fn default() -> Self {
        Self {
            backlinks_panel: true,
            daily_notes_panel: true,
            frontmatter_panel: true,
            enable_math: false,
            panels_default_flexible: true,
        }
    }
}

/// Construye OxidianFeatures leyendo .oxidian/config.json si existe un silo activo.
/// Si no hay silo detectado, usa defaults conservadores.
pub fn features_from_config(vault_root: Option<&std::path::Path>) -> OxidianFeatures {
    let Some(root) = vault_root else {
        return OxidianFeatures::default();
    };
    let config = load_vault_config(root.to_path_buf());
    OxidianFeatures {
        backlinks_panel: config.features.backlinks_panel,
        daily_notes_panel: config.features.daily_notes_panel,
        frontmatter_panel: config.features.frontmatter_panel,
        enable_math: config.features.enable_math,
        // Respect new default for panel flexibility if present in silo config
        // (stored under features.panels_default_flexible)
        panels_default_flexible: config.features.panels_default_flexible,
    }
}

/// Punto de entrada único para toda la inicialización de Oxidian.
///
/// Llama esta función desde `crates/zed/src/main.rs` dentro del bloque
/// `// OXIDIAN BEGIN` / `// OXIDIAN END`, en sustitución de las llamadas
/// individuales a los crates oxidian_*.
///
/// Orden garantizado:
/// 1. oxidian_vault — registra el índice y el WikiLinkResolver global (requerido por los demás)
/// 2. oxidian_backlinks — requiere ActiveVault registrado por vault
/// 3. oxidian_daily — requiere ActiveVault registrado por vault
/// 4. oxidian_frontmatter — requiere ActiveVault y VaultDatabase registrados por vault
pub fn init(fs: Arc<dyn Fs>, features: OxidianFeatures, cx: &mut App) {
    // Siempre activo — es el núcleo, los paneles dependen de él
    oxidian_vault::init(fs.clone(), cx);
    log::info!("Oxidian: silo index initialized");

    // OXIDIAN BEGIN — register render options global
    cx.set_global(oxidian_core::OxidianRenderOptions {
        enable_math: features.enable_math,
    });
    // OXIDIAN END

    // El flag de GPUI tiene prioridad sobre el config si está definido.
    // Como has_flag() devuelve bool, lo usamos con OR para que el config actúe como default.
    let backlinks = cx.has_flag::<OxidianBacklinksPanel>() || features.backlinks_panel;
    let daily = cx.has_flag::<OxidianDailyNotesPanel>() || features.daily_notes_panel;
    let frontmatter = cx.has_flag::<OxidianFrontmatterPanel>() || features.frontmatter_panel;

    if backlinks {
        oxidian_backlinks::init(cx);
        log::info!("Oxidian: backlinks panel initialized");
    }

    if daily {
        oxidian_daily::init(cx);
        log::info!("Oxidian: daily notes panel initialized");
    }

    if frontmatter {
        oxidian_frontmatter::init(cx);
        log::info!("Oxidian: frontmatter panel initialized");
    }

    oxidian_git::init(cx);
    log::info!("Oxidian: git integration initialized");

    // Cargar keymap de Oxidian por default
    if let Ok(bindings) =
        KeymapFile::load_asset_allow_partial_failure("keymaps/oxidian-default.json", cx)
    {
        cx.bind_keys(bindings);
        log::info!("Oxidian: keymap loaded");
    } else {
        log::warn!("Oxidian: failed to load default keymap");
    }

    // Registrar observer para aplicar el layout por default en nuevos silos
    cx.observe_new(move |workspace: &mut Workspace, window, cx| {
        let Some(worktree) = workspace.visible_worktrees(cx).next() else {
            return;
        };
        let root_path = worktree.read(cx).abs_path().to_path_buf();

        if oxidian_vault::is_vault_root(&root_path) {
            let layout_marker = root_path.join(".oxidian").join(".layout_applied");
            if !layout_marker.exists() {
                let workspace_handle = cx.weak_entity();
                if let Some(window) = window {
                    window.defer(cx, move |window, cx| {
                        if let Some(workspace) = workspace_handle.upgrade() {
                            workspace.update(cx, |workspace, cx| {
                                apply_default_vault_layout(workspace, window, cx);
                            });

                            // Marcar como aplicado
                            if let Some(parent) = layout_marker.parent() {
                                std::fs::create_dir_all(parent).ok();
                            }
                            std::fs::write(&layout_marker, "").ok();
                        }
                    });
                }
            }
        }
    })
    .detach();
}

/// Aplica el layout inicial de Oxidian a un workspace recién abierto sobre un vault.
/// Llamar desde el observer de workspace en init() cuando se detecta un vault.
///
/// Layout: [DailyNotes | Editor | Backlinks]
/// Panel izquierdo: DailyNotesPanel @ 280px
/// Panel derecho: BacklinksPanel @ 300px
pub fn apply_default_vault_layout(
    workspace: &mut Workspace,
    window: &mut Window,
    cx: &mut Context<Workspace>,
) {
    use oxidian_backlinks::BacklinksPanel;
    use oxidian_daily::DailyNotesPanel;

    // Abrir DailyNotesPanel en el dock izquierdo si no está ya visible
    if workspace.panel::<DailyNotesPanel>(cx).is_none() {
        workspace.toggle_panel_focus::<DailyNotesPanel>(window, cx);
    }

    // Abrir BacklinksPanel en el dock derecho si no está ya visible
    if workspace.panel::<BacklinksPanel>(cx).is_none() {
        workspace.toggle_panel_focus::<BacklinksPanel>(window, cx);
    }

    // Devolver el foco al editor
    workspace.focus_center_pane(window, cx);
}
