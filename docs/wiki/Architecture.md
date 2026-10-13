🌐 **English** · [Polski](Architektura)

# Architecture

The repository is a Cargo workspace with two crates.

| Crate | Purpose |
|---|---|
| `crates/lc-core` | UI-independent, unit-tested library |
| `crates/lightcreator` | [egui](https://github.com/emilk/egui) desktop application |

## baseCreator (shared with MillCreator)
The code that needs no laser lives in [baseCreator](https://github.com/michalsarna/baseCreator) and is pinned to a tag in `crates/lc-core/Cargo.toml` and `crates/lightcreator/Cargo.toml`.
* **base-core**: `geom`, `bezier`, `arcs`, `fillet`, `snap` (geometry, contours, round corners) · `ops` (boolean ops, offsets) · `text` (outlines) · `image`, `trace`, `camimg` · `scene` (shapes, groups, ids) · `svg`, `pdf` (readers) · `clipart` (library) · `units` · `grbl_settings`, `ser2net`.
* **base-ui**: `theme`, `fonts`, `icons`, `i18n` / `texts` · `menu`, `native_menu` (entries, in-window and macOS bars, the standard editing commands) · `window` (frame, pop-ups, buttons), `widgets` · `dialogs` (About, quick guide, update check) · `clipart_ui` · `editor` (canvas, tools, selection, node editing, editing commands, picture dialogs, view options, tool palette, arrange bar, the Properties sections, the pieces of the control and status bar behind the `EditorHost` trait) · `start` (start screen header, language and colour scheme row, window icon) · `markdown_ui`, `units_ui`, `camera`, `camera_stream`.

`lc-core` re-exports the modules of base-core, so `lc_core::geom` and the like keep working.

## lc-core
`doc` (the laser side of the document: layers, device, the laser section of the `.cre` project; the scene and the project container come from base-core) · `gcode`, `export` (G-code, PLT / DXF) · `raster`, `cutopts` · `svg`, `pdf`, `clipart` (placing imports on layers) · `materials`, `profiles`, `zones`, `check`, `hooks` · `controller`.

## lightcreator
`app` (main state, implements `EditorHost` and `MenuHost`) · `canvas` (bed markings, zones, live position, camera picture and preview as hooks of the editor), `shape_ops`, `overlay` · `menu` (the list of actions), `icons` (the icon of each action), `native_menu`, `panels`, `i18n` (the texts), `guide`, `help_ui`, `update_ui`, `prefs`, `window_ui` · `laser` (serial / TCP worker for GRBL and Marlin) · `device_ui`, `ser2net_ui`, `stream_ui` (devices, camera) · `clipart_ui` (the pictures that ship with the program), `materials_ui`, `preview_ui`, `print_ui`, `check_ui` · `screenshots` (headless rendering).

`build.rs` embeds the clipart. The interface icons are embedded by baseCreator (`assets/ui-icons`); a test checks that every icon an action asks for exists.

## Conventions
* Coordinates are **millimetres**, origin at the **top-left** of the work area. On output Y is flipped for machines whose zero is front-left (*Laser → Device settings*).
* Stored geometry is always mm; display units are per device.

## Testing and CI
* `cargo test --workspace` covers geometry, boolean, offset, raster, G-code, export and translation logic.
* Screenshots are rendered headlessly from the real UI: `cargo test -p lightcreator --release render_screenshots -- --ignored`.
* GRBL streaming is tested end to end in the [GRBL Simulator](GRBL-Simulator).
* GitHub workflows: `tests.yml`, `release.yml` (tag `v` + `Cargo.toml` version builds Windows / macOS / Linux packages), `codeql.yml`.

## Contributing
Issues and pull requests are welcome. Helpful contributions: bug reports, tests on real machines, translations, clipart and materials. See [Support](Support).
