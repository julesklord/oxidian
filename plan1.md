Plan de implementación / orquestación para gemini-cli

Resumen
- Objetivo: Implementar persistencia y UI para las opciones de paneles de Oxidian (backlinks, daily, frontmatter, default flexible) y exponerlas desde la UI de Settings. Persistir los cambios en `.oxidian/config.json` y aplicar la nueva configuración al estado en memoria (ActiveVault) sin requerir reinicio.
- Entregables:
  - Helper atómico para escribir `features` en `.oxidian/config.json`.
  - Estructura OxidianPanelsSettings añadida a `crates/settings_content`.
  - Controles en la UI de Settings (`crates/settings_ui`) para toggles de paneles y `panels_default_flexible`.
  - Wiring que guarda la config en disco y actualiza el `ActiveVault` en memoria.
  - Pasos de verificación y tests mínimos.

Suposiciones
- El código existente ya lee `features.panels_default_flexible` desde `.oxidian/config.json` (lectura implementada).
- Los crates Oxidian modificados (backlinks/daily/frontmatter) ya consumen la configuración por defecto desde ActiveSilo/OxidianFeatureFlags.
- Se debe usar operaciones atómicas de archivo (write temp + rename) para persistencia.

Riesgos y mitigaciones
- Riesgo: corrupción de `.oxidian/config.json` si la escritura falla.
  - Mitigación: escribir a archivo temporal en el mismo directorio y renombrar; en caso de error dejar el archivo original intacto y reportar error al usuario.
- Riesgo: condiciones de carrera actualizando ActiveVault en memoria desde múltiples ventanas.
  - Mitigación: actualizar el VaultConfig vía la API de GPUI en un contexto serializado (workspace.update_in / cx.update_in) para que las vistas reciban notificaciones apropiadas.

Plan de trabajo (pasos concretos)

1) Añadir helper de persistencia atómica
   - Archivo objetivo: `crates/oxidian_vault/src/oxidian_vault.rs` (añadir función pública)
   - Nombre sugerido: `save_oxidian_features_for_vault(root: &Path, features: &OxidianFeatureFlags) -> anyhow::Result<()>`
   - Comportamiento:
     - Leer `.oxidian/config.json` si existe, parsear JSON.
     - Reemplazar/actualizar la clave `features` con el objeto `features` serializado (solo campos existentes o todos los defaults — preferir serializar solo el struct entero para claridad).
     - Escribir JSON resultante a archivo temporal en el mismo directorio y renombrar sobre `config.json`.
     - Usar permisos/owner consistentes y manejo de errores razonable.
   - Verificación local: compilar `cargo build -p oxidian_vault`.

2) Añadir estructura OxidianPanelsSettings en settings_content
   - Archivo objetivo: `crates/settings_content/src/workspace.rs` o `settings_content.rs` (mantener coherencia con estilos existentes).
   - Contenido:
     - Estructura con campos Option<bool>: `backlinks_panel`, `daily_notes_panel`, `frontmatter_panel`, `panels_default_flexible`.
     - Derivados: Serialize/Deserialize, JsonSchema, MergeFrom, Default, Copy/Clone según convenciones del repo (usar `with_fallible_options` si requerido por el repo en esa área).
   - Objetivo: hacer que la herramienta de settings pueda exponer estas opciones en el schema y persistirlas como parte de la configuración de workspace / vault.
   - Verificación: compilar `cargo build --workspace` (o al menos compilar crates afectados: `settings_content` and `settings_ui`).

3) UI: añadir toggles en Settings
   - Archivo objetivo: `crates/settings_ui/src/pages/feature_flags.rs` o crear nueva página `crates/settings_ui/src/pages/oxidian_panels.rs` y registrarla.
   - Diseño mínimo:
     - Cuatro toggles (Checkbox o similar) con etiquetas: "Backlinks Panel", "Daily Notes Panel", "Frontmatter Panel", "Panels Default Flexible".
     - Mostrar estado actual leyendo `ActiveVault` -> `VaultConfig.features` si hay `ActiveVault`, o usar `OxidianFeatureFlags::default()` si no hay silo activo.
     - Al cambiar un toggle:
       - Llamar al helper de persistencia para escribir la nueva `features` a `.oxidian/config.json`.
       - Actualizar la `ActiveVault` en memoria (ver paso 4) para propagar cambios inmediatamente.
     - Considerar deshabilitar toggles si no hay `ActiveVault` (no hay silo abierto); mostrar hint "Open a vault to change these settings".
   - Verificación: compilar `cargo build -p settings_ui` y arrancar Zed/ventana de Settings (manual QA).

4) Runtime wiring: actualizar ActiveVault en memoria
   - Acción: después de guardar con `save_oxidian_features_for_vault`, cargar el VaultConfig actualizado (o modificar solo `features`) y establecerlo en la entidad `ActiveVault` / `VaultIndex` para que vistas lo lean.
   - Implementación preferida:
     - Usar la API que ya existe donde `ActiveVault` se expone como `Global` en `oxidian_vault`.
     - Ejecutar una actualización segura en el contexto de GPUI (por ejemplo `workspace.update_in` o `cx.update_in`) para reemplazar `vault.config.features` y notificar.
     - Si no hay acceso directo, exponer una función pública `set_vault_features(root: &Path, features: OxidianFeatureFlags, cx: &mut App)` que haga el merge y notifique.
   - Verificación: después de cambiar un toggle, observar que paneles abiertos cambian `flexible` property o que al abrir nuevos paneles usan el nuevo default.

5) Tests y verificación automatizada
   - Tests unitarios:
     - En `crates/oxidian_vault`: pruebas para `save_oxidian_features_for_vault` usando tempdir para verificar comportamiento atómico y que el archivo final contiene `features` esperado.
   - Tests de integración / manual:
     - Abrir Zed -> Open vault -> Settings -> toggles -> guardar. Verificar `.oxidian/config.json` actualizado y que la UI refleja cambios sin reinicio.
   - Comandos:
     - `cargo build --workspace`
     - `cargo nextest run -p oxidian_vault --no-fail-fast` para tests del crate.

6) Documentación y notas de PR
   - Incluir en la descripción del PR:
     - Qué archivos cambian y por qué
     - Cómo se gestiona la escritura atómica y por qué
     - Pasos de QA manuales
     - Release Notes breve

Lista detallada de archivos a tocar
- crates/oxidian_vault/src/oxidian_vault.rs
  - add: save_oxidian_features_for_vault
  - reuse: import_oxidian_features (lectura existente)
- crates/settings_content/src/workspace.rs (o settings_content.rs)
  - add: OxidianPanelsSettings struct + serde/schema derives
- crates/settings_ui/src/pages/
  - add/modify: página `oxidian_panels.rs` o `feature_flags.rs` para incluir los toggles
- crates/oxidian_backlinks/src/oxidian_backlinks.rs
- crates/oxidian_daily/src/oxidian_daily.rs
- crates/oxidian_frontmatter/src/oxidian_frontmatter.rs
  - (ya leen defaults de ActiveSilo/OxidianFeatureFlags) — verificar que respetan cambios en memoria

Comandos de desarrollo y QA
- Compilar workspace: `cargo build --workspace`
- Compilar un crate: `cargo build -p oxidian_vault`
- Correr tests para un crate: `cargo nextest run -p oxidian_vault --no-fail-fast`
- Ejecutar la app Zed localmente (si es la forma de QA) — usar el binario de desarrollo habitual del repo.

Estimación y milestones
- Paso 1 (helper persistencia): 1-2 horas
- Paso 2 (settings_content schema): 1 hora
- Paso 3 (UI toggles): 2-4 horas (incluye QA manual)
- Paso 4 (wiring runtime): 1-2 horas
- Paso 5 (tests y PR): 1-2 horas

Checklist antes de PR
- [ ] Helper de persistencia implementado y cubierto por tests unitarios
- [ ] Schema de settings añadido
- [ ] UI con toggles funcionando y compilando
- [ ] Guardado en disco atómico verificado
- [ ] Actualización en memoria propagada a vistas existentes
- [ ] PR con descripción, instrucciones de QA, Release Notes

Siguientes pasos si confirmas
- Procedo a implementar los cambios en el orden arriba listado. Haré commits pequeños y auto-verificaré con builds parciales antes de abrir el PR.

Si prefieres otra aproximación
- Podemos integrar estas opciones usando el `FeatureFlagStore` ya existente (reutilizar la UI `feature_flags.rs`) en lugar de añadir una sección nueva en `settings_content` — dime si prefieres que lo haga así.

Contacto durante la implementación
- Responderé aquí con actualizaciones por cada cambio importante (archivo modificado, comando ejecutado, errores de build, y resultado de tests).
