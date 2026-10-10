🌐 **English** · [Polski](Architektura)

# Architecture

The repository is a Cargo workspace with two crates.

| Crate | Purpose |
|---|---|
| `crates/lc-core` | UI-independent, unit-tested library |
| `crates/lightcreator` | [egui](https://github.com/emilk/egui) desktop application |

## lc-core
`geom`, `bezier`, `fillet` (geometry, contours, round corners) · `ops` (boolean ops, offsets) · `text` (outlines) · `raster`, `image`, `trace` (engraving, dithering, tracing) · `gcode`, `export` (G-code, PLT / DXF) · `svg`, `pdf` (import) · `doc` (document model, `.lcr`) · `materials`, `clipart` · `controller`, `ser2net`.

## lightcreator
`app` (main state) · `canvas`, `nodes`, `shape_ops`, `overlay` (editing) · `panels`, `menu`, `native_menu`, `theme`, `icons`, `fonts`, `i18n`, `guide` / `help_ui` (UI) · `laser` (serial / TCP worker for GRBL and Marlin) · `device_ui`, `ser2net_ui`, `camera`, `camera_stream`, `stream_ui` (devices, camera) · `clipart_ui`, `clip_net`, `materials_ui`, `image_ui`, `preview_ui`, `units_ui` · `prefs` (`app.ron`) · `screenshots` (headless rendering).

`build.rs` collects interface icons from `assets/ui-icons` and embeds the clipart; a test checks that every requested icon exists.

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
