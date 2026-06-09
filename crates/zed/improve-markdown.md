# Oxidian — Plan Maestro: Mejoras de Renderizado Markdown
> Prompt para agentes de código (Gemini CLI / Codex / OpenCode)
> Repo: `/home/julesklord/Proyectos/repos/oxidian`
> Nomenclatura del proyecto: Vault → **Silo**, Graph view → **Matrix**

---

## Contexto

El stack de markdown de Oxidian usa:
- `crates/markdown/` — parser (pulldown-cmark) + tipos de eventos + renderer GPUI
- `crates/markdown_preview/` — vista de preview: `MarkdownPreviewView` + `handle_url_click`
- `crates/oxidian_core/` — tipos base: `WikiLink`, `SiloConfig`, `WikiLinkResolver` (global GPUI)
- `crates/oxidian_vault/` — índice del Silo, `ActiveSilo` global, `load_silo_config`

El parser ya inyecta wiki-links como eventos `MarkdownTag::Link` con `dest_url = target_note_name` (sin esquema). El renderer actual pasa ese URL a `handle_url_click` → `open_preview_url` → `cx.open_url()` — lo cual falla silenciosamente o abre el browser.

**Objetivo:** 5 mejoras ordenadas por impacto real para PKM.

---

## Restricciones globales para el agente

1. No modificar crates de Zed upstream excepto dentro de bloques `// OXIDIAN BEGIN` / `// OXIDIAN END`.
2. Todos los cambios son aditivos — no romper comportamiento existente de Zed.
3. Inspeccionar API antes de usarla. No inventar métodos.
4. `cargo check` después de cada mejora antes de continuar.
5. Nomenclatura: `Silo` (no Vault), `Matrix` (no graph view), `SiloConfig`, `ActiveSilo`, `load_silo_config`.

---

## Mejora 1 — Navegación de wiki-links (CRÍTICO)

**Problema:** `[[Mi Nota]]` se renderiza como link pero al hacer click no hace nada útil.
**Causa:** `handle_url_click` en `markdown_preview_view.rs` recibe `dest_url = "Mi Nota"` (sin esquema), lo pasa a `open_preview_url` que intenta `resolve_preview_path("Mi Nota", base_dir)` → no encuentra el archivo → llama `cx.open_url("Mi Nota")` → falla.
**Fix:** Interceptar en `handle_url_click` antes del path resolver.

### 1.1 Modificar `handle_url_click` en `crates/markdown_preview/src/markdown_preview_view.rs`

Localizar la función `handle_url_click`. Actualmente comienza con:

```rust
fn handle_url_click(
    url: SharedString,
    view: &WeakEntity<MarkdownPreviewView>,
    base_directory: Option<PathBuf>,
    workspace: &WeakEntity<Workspace>,
    window: &mut Window,
    cx: &mut App,
) {
    let (path_part, fragment) = split_local_url_fragment(url.as_ref());
    ...
```

**Agregar al inicio del body**, antes de `let (path_part, fragment) = ...`:

```rust
// OXIDIAN BEGIN — wiki-link navigation
// Un wiki-link tiene dest_url sin esquema y sin extensión de archivo.
// Detectarlo: no empieza con http/https/file/zed, no contiene '/', no tiene extensión .md/.txt etc.
// El WikiLinkResolver global (registrado por oxidian_vault) lo convierte a PathBuf.
if is_wiki_link_url(url.as_ref()) {
    if let Some(resolver) = cx.try_global::<oxidian_core::WikiLinkResolver>() {
        // WikiLinkResolver.0 es Arc<dyn Fn(&str, &mut Window, &mut App) -> Option<PathBuf>>
        let target = url.as_ref().to_string();
        let resolver_fn = resolver.0.clone();
        let workspace = workspace.clone();
        window.defer(cx, move |window, cx| {
            if let Some(path) = (resolver_fn)(&target, window, cx) {
                if let Some(workspace) = workspace.upgrade() {
                    workspace.update(cx, |workspace, cx| {
                        workspace.open_abs_path(
                            path,
                            workspace::OpenOptions {
                                visible: Some(workspace::OpenVisible::None),
                                ..Default::default()
                            },
                            window,
                            cx,
                        ).detach();
                    }).ok();
                }
            } else {
                log::warn!("Oxidian: wiki-link target '{}' not found in Silo", target);
            }
        });
        return;
    }
}
// OXIDIAN END — wiki-link navigation
```

### 1.2 Agregar función helper `is_wiki_link_url` en el mismo archivo

Agregar justo antes de `handle_url_click`:

```rust
// OXIDIAN BEGIN — wiki-link URL detection
/// Detecta si una URL proviene de un wiki-link inyectado por oxidian_vault.
/// Un wiki-link no tiene esquema URI, no tiene separadores de path, y no tiene
/// extensión de archivo reconocida. Ejemplos:
///   "Mi Nota"          → wiki-link ✓
///   "Proyectos/Alpha"  → NO (tiene '/')
///   "notes.md"         → NO (tiene extensión)
///   "https://..."      → NO (tiene esquema)
fn is_wiki_link_url(url: &str) -> bool {
    if url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("file://")
        || url.starts_with("zed://")
        || url.starts_with("data:")
    {
        return false;
    }
    // Si contiene '/' o '\' es un path relativo, no un wiki-link
    if url.contains('/') || url.contains('\\') {
        return false;
    }
    // Si tiene extensión de archivo conocida, es un link a archivo regular
    let known_extensions = [".md", ".txt", ".pdf", ".png", ".jpg", ".jpeg", ".svg", ".gif"];
    if known_extensions.iter().any(|ext| url.ends_with(ext)) {
        return false;
    }
    true
}
// OXIDIAN END — wiki-link URL detection
```

### 1.3 Agregar `oxidian_core` como dependencia de `markdown_preview`

En `crates/markdown_preview/Cargo.toml`, dentro de `[dependencies]`:

```toml
# OXIDIAN BEGIN
oxidian_core.workspace = true
# OXIDIAN END
```

### 1.4 Agregar import en `markdown_preview_view.rs`

En el bloque de `use` statements, agregar:

```rust
// OXIDIAN BEGIN
use oxidian_core::WikiLinkResolver;
// OXIDIAN END
```

### 1.5 Verificación

```bash
cargo check -p markdown_preview 2>&1
```

Test manual: abrir una nota `.md` en el Silo que contenga `[[Otra Nota]]`, abrir preview, hacer click en el link. Debe abrir `Otra Nota.md` en el editor.

---

## Mejora 2 — Imágenes relativas al Silo

**Problema:** `![](imagen.png)` en una nota funciona si la imagen está en el mismo directorio que la nota, pero falla para paths relativos al Silo root como `_assets/imagen.png`.
**Estado actual:** `resolve_preview_image` ya resuelve paths relativos al directorio de la nota (`base_directory`) y paths absolutos al workspace root. Falta soporte para el root del Silo.

### 2.1 Crear función `resolve_silo_asset_path` en `crates/oxidian_vault/src/oxidian_vault.rs`

```rust
// OXIDIAN BEGIN — asset path resolver
use oxidian_core::SiloConfig;

/// Resuelve un path de asset relativo al Silo root.
/// Busca en este orden:
/// 1. Relativo al directorio de la nota actual (ya manejado por markdown_preview)
/// 2. Relativo al Silo root
/// 3. En el directorio _assets/ del Silo root
/// Retorna None si no encuentra el archivo en ningún lugar.
pub fn resolve_silo_asset(relative_path: &str, cx: &App) -> Option<std::path::PathBuf> {
    let silo_root = cx.try_global::<ActiveSilo>()?.0.root.clone();

    // Intento 1: relativo al Silo root
    let candidate = silo_root.join(relative_path);
    if candidate.exists() {
        return Some(candidate);
    }

    // Intento 2: dentro de _assets/
    let assets_candidate = silo_root.join("_assets").join(relative_path);
    if assets_candidate.exists() {
        return Some(assets_candidate);
    }

    None
}
// OXIDIAN END — asset path resolver
```

### 2.2 Modificar `resolve_preview_image` en `markdown_preview_view.rs`

Localizar el final de la función `resolve_preview_image`. Actualmente termina con:

```rust
    path.exists()
        .then(|| ImageSource::Resource(Resource::Path(Arc::from(path.as_path()))))
}
```

**Reemplazar** esa última sección para agregar el fallback al Silo:

```rust
    if path.exists() {
        return Some(ImageSource::Resource(Resource::Path(Arc::from(path.as_path()))));
    }

    // OXIDIAN BEGIN — Silo asset fallback
    // Si no se resolvió como path relativo a la nota, no intentar el Silo aquí
    // porque resolve_preview_image no tiene acceso a cx.
    // El Silo root se pasa como parámetro adicional (ver 2.3).
    if let Some(silo_root) = silo_root {
        let candidate = silo_root.join(&decoded);
        if candidate.exists() {
            return Some(ImageSource::Resource(Resource::Path(Arc::from(candidate.as_path()))));
        }
        let assets_candidate = silo_root.join("_assets").join(&decoded);
        if assets_candidate.exists() {
            return Some(ImageSource::Resource(Resource::Path(Arc::from(assets_candidate.as_path()))));
        }
    }
    // OXIDIAN END — Silo asset fallback

    None
}
```

### 2.3 Actualizar la firma de `resolve_preview_image` para recibir `silo_root`

Cambiar la firma de:
```rust
fn resolve_preview_image(
    dest_url: &str,
    base_directory: Option<&Path>,
    workspace_directory: Option<&Path>,
) -> Option<ImageSource>
```
a:
```rust
fn resolve_preview_image(
    dest_url: &str,
    base_directory: Option<&Path>,
    workspace_directory: Option<&Path>,
    silo_root: Option<&Path>,  // OXIDIAN: raíz del Silo activo
) -> Option<ImageSource>
```

### 2.4 Actualizar el call site en `render_markdown_element`

Localizar el bloque `.image_resolver(...)` en `render_markdown_element`. Antes del closure, obtener el silo_root:

```rust
// OXIDIAN BEGIN — obtener Silo root para resolver assets
let silo_root: Option<PathBuf> = cx.try_global::<oxidian_core::ActiveSilo>()
    .map(|silo| silo.0.root.clone());
// OXIDIAN END

let mut markdown_element = MarkdownElement::new(...)
    .image_resolver({
        let base_directory = self.base_directory.clone();
        // OXIDIAN BEGIN
        let silo_root = silo_root.clone();
        // OXIDIAN END
        move |dest_url| {
            resolve_preview_image(
                dest_url,
                base_directory.as_deref(),
                workspace_directory.as_deref(),
                silo_root.as_deref(),  // OXIDIAN
            )
        }
    })
```

**Nota:** `ActiveSilo` debe estar importado. Agregar al bloque de imports:
```rust
// OXIDIAN BEGIN
use oxidian_core::ActiveSilo;
// OXIDIAN END
```

### 2.5 Verificación

```bash
cargo check -p markdown_preview 2>&1
```

Test manual: nota con `![](imagen.png)` donde `imagen.png` está en `_assets/imagen.png` del Silo root. Verificar que el preview muestra la imagen.

---

## Mejora 3 — Frontmatter como Properties panel

**Problema:** El bloque YAML frontmatter (`---\ntitle: Mi Nota\ntags: [PKM]\n---`) se skipea en preview o se muestra como texto crudo.
**Estado actual:** `parse_markdown_with_options` ya parsea `metadata_blocks` con `MetadataRow { key, value }`. El renderer de Zed tiene soporte para `MarkdownTag::MetadataBlock` pero lo renderiza de forma minimal. Necesitamos un renderer estilizado tipo "Properties panel".

### 3.1 Agregar renderer de frontmatter en `crates/markdown/src/markdown.rs`

Localizar el render de `MarkdownTag::MetadataBlock` en el renderer GPUI del crate `markdown`. Si no existe un handler específico, crear uno dentro de un bloque `// OXIDIAN BEGIN`.

El objetivo visual es una tabla compacta:

```zsh
┌─────────────────────────────┐
│  title    Mi Nota           │
│  tags     PKM, dev          │
│  date     2025-06-01        │
└─────────────────────────────┘
```

**Implementación:**

```rust
// OXIDIAN BEGIN — frontmatter properties panel
/// Renderiza un bloque de metadata YAML como una tabla de propiedades estilizada.
/// Reemplaza el renderizado default de MetadataBlock cuando hay rows parseadas.
fn render_metadata_as_properties(
    rows: &[crate::parser::MetadataRow],
    source: &str,
    cx: &App,
) -> impl gpui::IntoElement {
    use gpui::prelude::*;
    use ui::{Label, LabelSize};

    let colors = cx.theme().colors();

    div()
        .rounded_md()
        .border_1()
        .border_color(colors.border_variant)
        .bg(colors.surface_background)
        .mb_3()
        .p_3()
        .children(rows.iter().map(|row| {
            let key = source[row.key.clone()].to_string();
            let value = source[row.value.clone()].to_string();
            div()
                .flex()
                .gap_3()
                .py_0p5()
                .child(
                    div()
                        .w_24()
                        .flex_shrink_0()
                        .child(Label::new(key).size(LabelSize::Small).color(ui::Color::Muted))
                )
                .child(
                    div()
                        .flex_1()
                        .child(Label::new(value).size(LabelSize::Small))
                )
        }))
}
// OXIDIAN END — frontmatter properties panel
```

**Nota para el agente:** Verificar la API exacta de `ui::Label`, colores de tema, y el sistema de layout de GPUI en el contexto del renderer de markdown antes de implementar. El renderer de markdown usa elementos GPUI directamente — adaptar al patrón existente en el archivo, no inventar una API nueva.

### 3.2 Hookear el renderer en el pipeline de renderizado

En el renderer de markdown (localizar el match sobre `MarkdownEvent::Start(MarkdownTag::MetadataBlock(_))`), dentro de un bloque `// OXIDIAN BEGIN`:

```rust
// OXIDIAN BEGIN — render frontmatter as properties
MarkdownTag::MetadataBlock(_) => {
    // Si tenemos rows parseadas, usar el properties panel
    if let Some(block) = parsed_data.metadata_blocks.get(&event_range.start) {
        if let Some(rows) = &block.rows {
            // Renderizar como properties panel en lugar del default
            return render_metadata_as_properties(rows, source, cx).into_any_element();
        }
    }
    // Fallback: comportamiento original de Zed
    render_metadata_block_default(...)
}
// OXIDIAN END — render frontmatter as properties
```

**Nota:** Inspeccionar el archivo `crates/markdown/src/markdown.rs` para entender el patrón exacto del match de eventos antes de implementar. La estructura puede ser un `fn render_block(event, ...)` o un loop sobre eventos — adaptar sin romper el patrón existente.

### 3.3 Verificación

```bash
cargo check -p markdown 2>&1
cargo check -p markdown_preview 2>&1
```

Test manual: nota con frontmatter YAML válido (pares `key: value` simples). El preview debe mostrar una tabla de propiedades en lugar de texto crudo.

---

## Mejora 4 — Callouts de Obsidian (tipos arbitrarios)

**Problema:** Obsidian usa `> [!tipo]` con tipos custom (`> [!warning]`, `> [!info]`, `> [!question]`, etc.) además de los 5 GFM standard. El parser actual solo reconoce los 5 de `pulldown_cmark::BlockQuoteKind`.

**Estado actual:** `MarkdownTag::BlockQuote(Option<pulldown_cmark::BlockQuoteKind>)` — el `Option` es `None` para tipos no reconocidos por GFM.

### 4.1 Agregar un pre-scanner de callouts en `crates/markdown/src/parser.rs`

Agregar dentro del bloque `// OXIDIAN BEGIN` existente o crear uno nuevo, antes de `parse_markdown_with_options`:

```rust
// OXIDIAN BEGIN — Obsidian callout pre-scanner

/// Tipos de callout soportados por Obsidian que no son GFM standard.
/// GFM standard (ya manejados): note, tip, important, warning, caution
const OBSIDIAN_CALLOUT_TYPES: &[&str] = &[
    "abstract", "summary", "tldr",
    "info", "todo",
    "success", "check", "done",
    "question", "help", "faq",
    "failure", "fail", "missing",
    "danger", "error",
    "bug",
    "example",
    "quote", "cite",
];

/// Representa un callout de Obsidian con tipo arbitrario.
#[derive(Clone, Debug, PartialEq)]
pub struct OxidianCallout {
    /// El tipo del callout, ej: "warning", "info", "question"
    pub kind: Arc<str>,
    /// Si el callout es colapsable (empieza con `> [!tipo]-` o `> [!tipo]+`)
    pub collapsible: bool,
    /// Si el callout empieza colapsado
    pub collapsed: bool,
    /// El título del callout (la parte después de `[!tipo]` en la misma línea)
    pub title: Option<Arc<str>>,
    /// Offset de bytes donde empieza el bloque
    pub byte_offset: usize,
}

/// Pre-escanea el texto buscando callouts de Obsidian no reconocidos por GFM.
/// Retorna un map de byte_offset → OxidianCallout.
pub(crate) fn extract_obsidian_callouts(text: &str) -> std::collections::BTreeMap<usize, OxidianCallout> {
    let mut results = std::collections::BTreeMap::new();
    let mut byte_offset: usize = 0;

    for line in text.lines() {
        let trimmed = line.trim_start_matches(|c: char| c == '>' || c == ' ');

        if let Some(inner) = trimmed.strip_prefix("[!") {
            if let Some(end_bracket) = inner.find(']') {
                let raw_kind = &inner[..end_bracket];
                let after_bracket = &inner[end_bracket + 1..];

                // Detectar si es collapsible
                let (collapsible, collapsed, title_part) = match after_bracket.chars().next() {
                    Some('-') => (true, true, after_bracket[1..].trim()),
                    Some('+') => (true, false, after_bracket[1..].trim()),
                    _ => (false, false, after_bracket.trim()),
                };

                let kind_lower = raw_kind.to_lowercase();
                // Solo procesar si es un tipo Obsidian (no GFM ya manejado)
                let is_gfm = matches!(kind_lower.as_str(), "note" | "tip" | "important" | "warning" | "caution");

                if !is_gfm {
                    results.insert(byte_offset, OxidianCallout {
                        kind: Arc::from(kind_lower.as_str()),
                        collapsible,
                        collapsed,
                        title: if title_part.is_empty() {
                            None
                        } else {
                            Some(Arc::from(title_part))
                        },
                        byte_offset,
                    });
                }
            }
        }

        byte_offset += line.len() + 1; // +1 for newline
    }

    results
}
// OXIDIAN END — Obsidian callout pre-scanner
```

### 4.2 Exponer `OxidianCallout` en `ParsedMarkdownData`

En `ParsedMarkdownData`, agregar el campo dentro del bloque `// OXIDIAN BEGIN`:

```rust
// OXIDIAN BEGIN
pub wiki_links: Vec<(Range<usize>, WikiLink)>,
pub obsidian_callouts: std::collections::BTreeMap<usize, OxidianCallout>,  // <-- nuevo
// OXIDIAN END
```

En `parse_markdown_with_options`, llamar el scanner y guardar en el struct:

```rust
// OXIDIAN BEGIN — wiki-link y callout pre-scan
let wiki_links = extract_wiki_links(text);
let obsidian_callouts = extract_obsidian_callouts(text);  // <-- nuevo
let events = inject_wiki_link_events(state.events, &wiki_links);
// OXIDIAN END
```

Y en el `ParsedMarkdownData { ... }` retornado:
```rust
// OXIDIAN BEGIN
wiki_links,
obsidian_callouts,  // <-- nuevo
// OXIDIAN END
```

### 4.3 Agregar íconos y colores por tipo en el renderer

En `crates/markdown/src/markdown.rs`, agregar una función helper dentro de `// OXIDIAN BEGIN`:

```rust
// OXIDIAN BEGIN — Obsidian callout renderer helpers

/// Retorna el ícono y color para un tipo de callout de Obsidian.
/// Retorna (icon_name, ui::Color) para usar en el renderer.
fn callout_style_for_kind(kind: &str) -> (ui::IconName, ui::Color) {
    match kind {
        "abstract" | "summary" | "tldr" => (ui::IconName::FileText, ui::Color::Accent),
        "info" | "todo" => (ui::IconName::Info, ui::Color::Info),
        "success" | "check" | "done" => (ui::IconName::Check, ui::Color::Success),
        "question" | "help" | "faq" => (ui::IconName::QuestionMark, ui::Color::Warning),
        "failure" | "fail" | "missing" => (ui::IconName::XCircle, ui::Color::Error),
        "danger" | "error" | "bug" => (ui::IconName::Warning, ui::Color::Error),
        "example" => (ui::IconName::ListBullet, ui::Color::Accent),
        "quote" | "cite" => (ui::IconName::Quote, ui::Color::Muted),
        _ => (ui::IconName::Info, ui::Color::Muted),  // fallback
    }
}
// OXIDIAN END — Obsidian callout renderer helpers
```

**Nota:** Verificar que `ui::IconName` tiene los iconos mencionados. Si alguno no existe, usar el más cercano disponible. No inventar nombres de íconos.

### 4.4 Hookear el renderer para callouts

En el handler de `MarkdownTag::BlockQuote(None)` (blockquotes sin tipo GFM reconocido), dentro de un bloque `// OXIDIAN BEGIN`:

```rust
// OXIDIAN BEGIN — Obsidian callout render
MarkdownTag::BlockQuote(None) => {
    // Verificar si es un callout de Obsidian basado en el byte offset
    if let Some(callout) = parsed_data.obsidian_callouts.get(&event_range.start) {
        let (icon, color) = callout_style_for_kind(&callout.kind);
        // Renderizar como callout estilizado con ícono y borde de color
        // Estructura: borde izquierdo coloreado + header con ícono + título + contenido
        return render_obsidian_callout(callout, icon, color, cx).into_any_element();
    }
    // Fallback: blockquote normal
    render_blockquote_default(...)
}
// OXIDIAN END — Obsidian callout render
```

### 4.5 Verificación

```bash
cargo check -p markdown 2>&1
```

Test manual: nota con `> [!info]\n> Contenido del callout`. Debe mostrar un callout con ícono y borde de color.

---

## Mejora 5 — Math (LaTeX) opcional

**Problema:** `Options::ENABLE_MATH` está en `UNWANTED_OPTIONS` — correcto para Zed como editor de código, pero muchos usuarios de PKM tienen fórmulas LaTeX.
**Fix:** Hacerlo activable via `OxidianFeatureFlags.enable_math`.

### 5.1 Agregar `enable_math` a `OxidianFeatureFlags` en `crates/oxidian_core/src/oxidian_core.rs`

En el struct `OxidianFeatureFlags` (agregado en el plan de orquestación Fase 2):

```rust
pub struct OxidianFeatureFlags {
    pub backlinks_panel: bool,
    pub daily_notes_panel: bool,
    pub frontmatter_panel: bool,
    pub vim_mode: bool,
    pub git_panel: bool,
    pub enable_math: bool,    // <-- nuevo, default: false
}

impl Default for OxidianFeatureFlags {
    fn default() -> Self {
        Self {
            backlinks_panel: true,
            daily_notes_panel: true,
            frontmatter_panel: true,
            vim_mode: false,
            git_panel: false,
            enable_math: false,   // <-- nuevo
        }
    }
}
```

### 5.2 Exponer un global `OxidianRenderOptions` en `oxidian_core`

```rust
// OXIDIAN BEGIN — render options global
/// Opciones de renderizado de Oxidian, registradas como global GPUI por oxidian_init.
#[derive(Clone, Debug, Default)]
pub struct OxidianRenderOptions {
    pub enable_math: bool,
}

impl gpui::Global for OxidianRenderOptions {}
// OXIDIAN END — render options global
```

### 5.3 Registrar `OxidianRenderOptions` en `oxidian_init::init`

En `crates/oxidian_init/src/oxidian_init.rs`, dentro de `init()`:

```rust
// OXIDIAN BEGIN — register render options
cx.set_global(oxidian_core::OxidianRenderOptions {
    enable_math: features.frontmatter_panel, // usar el flag correspondiente
});
// OXIDIAN END
```

**Corrección:** usar `features.enable_math` (necesita ser agregado a `OxidianFeatures` en `oxidian_init`):

```rust
pub struct OxidianFeatures {
    pub backlinks_panel: bool,
    pub daily_notes_panel: bool,
    pub frontmatter_panel: bool,
    pub enable_math: bool,   // <-- nuevo
}

impl Default for OxidianFeatures {
    fn default() -> Self {
        Self {
            backlinks_panel: true,
            daily_notes_panel: true,
            frontmatter_panel: true,
            enable_math: false,
        }
    }
}
```

### 5.4 Usar el flag en `parse_markdown_with_options`

En `crates/markdown/src/parser.rs`, modificar `PARSE_OPTIONS` para que math sea condicional. Actualmente `PARSE_OPTIONS` es una constante. Convertirlo en una función:

```rust
// OXIDIAN BEGIN — math options
/// Retorna las opciones de parse, opcionalmente con math habilitado.
pub(crate) fn parse_options(enable_math: bool) -> pulldown_cmark::Options {
    let base = Options::ENABLE_TABLES
        .union(Options::ENABLE_FOOTNOTES)
        .union(Options::ENABLE_STRIKETHROUGH)
        .union(Options::ENABLE_TASKLISTS)
        .union(Options::ENABLE_SMART_PUNCTUATION)
        .union(Options::ENABLE_HEADING_ATTRIBUTES)
        .union(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS)
        .union(Options::ENABLE_OLD_FOOTNOTES)
        .union(Options::ENABLE_GFM)
        .union(Options::ENABLE_SUPERSCRIPT)
        .union(Options::ENABLE_SUBSCRIPT);

    if enable_math {
        base.union(Options::ENABLE_MATH)
    } else {
        base
    }
}
// OXIDIAN END — math options
```

### 5.5 Pasar `enable_math` a `parse_markdown_with_options`

Agregar el parámetro `enable_math: bool` a `parse_markdown_with_options` y reemplazar `PARSE_OPTIONS` con `parse_options(enable_math)` en el body.

En `MarkdownPreviewView::new`, cuando se crea el `Markdown::new_with_options`, leer el global:

```rust
// OXIDIAN BEGIN — leer render options
let enable_math = cx.try_global::<oxidian_core::OxidianRenderOptions>()
    .map(|opts| opts.enable_math)
    .unwrap_or(false);
// OXIDIAN END

let markdown = cx.new(|cx| {
    Markdown::new_with_options(
        SharedString::default(),
        Some(language_registry),
        None,
        MarkdownOptions {
            parse_html: true,
            render_mermaid_diagrams: true,
            parse_heading_slugs: true,
            render_metadata_blocks: true,
            enable_math,   // OXIDIAN: pasar el flag
            ..Default::default()
        },
        cx,
    )
});
```

**Nota:** Verificar si `MarkdownOptions` ya tiene un campo `enable_math` o si necesita ser agregado. Inspeccionar `crates/markdown/src/markdown.rs` antes de implementar.

### 5.6 Agregar `enable_math` en `config.example.json`

En `assets/oxidian/config.example.json`:

```json
{
  "features": {
    "backlinks_panel": true,
    "daily_notes_panel": true,
    "frontmatter_panel": true,
    "vim_mode": false,
    "git_panel": false,
    "enable_math": false
  }
}
```

### 5.7 Agregar import de `oxidian_core` en `markdown_preview_view.rs` si no existe

Ya fue agregado en Mejora 1. Verificar que no hay duplicados.

### 5.8 Verificación

```bash
cargo check -p oxidian_core 2>&1
cargo check -p oxidian_init 2>&1
cargo check -p markdown 2>&1
cargo check -p markdown_preview 2>&1
cargo check -p zed 2>&1
```

Test manual: activar `enable_math: true` en `.oxidian/config.json` del Silo, reiniciar, abrir nota con `$E = mc^2$`. Debe renderizar la fórmula.

---

## Árbol de archivos a crear/modificar

```
MODIFICAR:
  crates/markdown_preview/Cargo.toml                    (agregar oxidian_core dep)
  crates/markdown_preview/src/markdown_preview_view.rs  (Mejoras 1, 2, 5)
  crates/markdown/src/parser.rs                         (Mejoras 3, 4, 5)
  crates/markdown/src/markdown.rs                       (Mejoras 3, 4)
  crates/oxidian_core/src/oxidian_core.rs               (Mejoras 5: OxidianRenderOptions, enable_math)
  crates/oxidian_init/src/oxidian_init.rs               (Mejora 5: registrar OxidianRenderOptions)
  assets/oxidian/config.example.json                    (Mejora 5: enable_math)
```

---

## Orden de ejecución

```
Mejora 1 → cargo check → Mejora 2 → cargo check → Mejora 3 → cargo check → Mejora 4 → cargo check → Mejora 5 → cargo check → cargo build -p zed
```

Las mejoras 1 y 2 son independientes entre sí y pueden hacerse en paralelo.
La mejora 3 es independiente de 1 y 2.
La mejora 4 depende de que el parser esté estable (después de 3).
La mejora 5 depende del plan de orquestación Fase 2 (`OxidianFeatureFlags` ya existente).

---

## Notas de nomenclatura

En todo el código generado usar consistentemente:
- `ActiveSilo` (no `ActiveVault`)
- `SiloConfig` (no `VaultConfig`)
- `load_silo_config` (no `load_vault_config`)
- `silo_root` (no `vault_root`)
- `WikiLinkResolver` — se mantiene (es nomenclatura técnica del formato, no de UX)
