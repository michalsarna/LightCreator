<p align="center"><img src="assets/icons/icon-256.png" alt="LightCreator icon" width="128"></p>

# LightCreator

**Current version: v0.02** — see [VERSIONS.md](VERSIONS.md) for the change log, [FEATURES.md](FEATURES.md) for what the app can do and [INSTALL.md](INSTALL.md) for how to run it from source.

An open-source alternative to [LightBurn](https://lightburnsoftware.com/) for CNC laser engravers and cutters,
written in Rust. The interface follows the look of [VectorCraft](https://github.com/storytold/vectorcraft)
(light flat chrome, vertical tool bar, collapsible side panels, colour swatch strip, red-orange accent).

## Screenshots

| | |
|---|---|
| ![Start window](docs/screenshots/01-start.png) | ![Properties tab](docs/screenshots/02-editor-properties.png) |
| Start window: pick the device to work with | Editor, Properties tab, toolpath preview on |
| ![Cuts / Layers tab](docs/screenshots/03-layers.png) | ![Console tab](docs/screenshots/04-console.png) |
| Cuts / Layers tab | Console tab: live view of the serial traffic |
| ![Device configuration](docs/screenshots/05-device-config.png) | ![Light scheme, Polish](docs/screenshots/06-light-polish.png) |
| Device configuration (units, work area, GRBL settings) | Light colour scheme with the Polish interface |

The pictures are rendered headlessly from the real UI; regenerate them with
`cargo test -p lightcreator --release render_screenshots -- --ignored`.

## Settings and configuration files

Language, colour scheme, device profiles (including their units), the last used profile and the window
size are stored by the application in a single file, `app.ron`:

| System | Location |
|--------|----------|
| macOS | `~/Library/Application Support/lightcreator/app.ron` |
| Windows | `%APPDATA%\lightcreator\data\app.ron` (usually `C:\Users\<you>\AppData\Roaming\lightcreator\data\app.ron`) |
| Linux | `$XDG_DATA_HOME/lightcreator/app.ron`, by default `~/.local/share/lightcreator/app.ron` |

The file is written when the application closes. Delete it to reset everything to the defaults (the start
window then asks for a device profile again). Projects (`.lcr`), SVG and G-code files are saved wherever you
choose in the file dialog.

## Features

A short summary is below; the full list lives in [FEATURES.md](FEATURES.md).

* Design, import / export (SVG, G-code, native `.lcr`), LightBurn-style cut layers, toolpath preview.
* GRBL laser control over a serial port.
* Native menu bar on macOS, in-window menu on Windows and Linux.
* Start window with device profiles, millimetre or inch units per device.
* Side-panel tabs: Properties, Cuts / Layers, Device and a live serial Console.
* Seven languages (English, Polski, Deutsch, Italiano, Suomi, 中文, हिन्दी) and four colour schemes.

Not yet implemented (contributions welcome): image/raster engraving, offset fill, boolean operations,
Bézier node editing, text tool, Ruida / Trocen / Marlin controllers, material library, camera overlay.

## Building

```sh
cargo run --release -p lightcreator                      # start empty
cargo run --release -p lightcreator -- examples/badge.svg  # open a file
cargo test --workspace
```

Requires a recent stable Rust toolchain. On Linux the GUI needs the usual X11/Wayland + OpenGL libraries
(`libxkbcommon-x11`, `libgl1`).

## Icon

The application icon lives in `assets/` (`lightcreator.icns` for macOS, `lightcreator.ico` for Windows and
PNG sizes 16–1024 px in `assets/icons/`). It is used for the window, the Dock, the Windows executable, the
About window and the Linux desktop entry.

## Layout

| Crate | Purpose |
|-------|---------|
| `crates/lc-core` | geometry, document model, SVG I/O, G-code generation (UI independent, unit-tested) |
| `crates/lightcreator` | egui desktop application, GRBL serial worker |

Coordinates are millimetres with the origin at the top-left of the work area; on output Y is flipped for
machines whose zero is at the front-left (configurable under *Laser → Device settings*).

> **Safety:** lasers are dangerous. Always test G-code at low power, wear eye protection, never leave a
> running machine unattended and verify your machine's `$30` (S-max) and `$32` (laser mode) settings.

## License

MIT

## Versioning

Work is committed directly to `main`. Each release gets the next version number and a branch named
after it (`v0.01`, `v0.02`, …). Current version: **v0.01**.
