<p align="center"><img src="assets/icons/icon-256.png" alt="LightCreator icon" width="128"></p>

<p align="center">
  <a href="SUPPORT.md">
    <img src="https://img.buymeacoffee.com/button-api/?text=Buy%20me%20a%20coffee&emoji=%E2%98%95&slug=michalsarna&button_colour=FFDD00&font_colour=000000&font_family=Cookie&outline_colour=000000&coffee_colour=ffffff" alt="Buy me a coffee" height="48">
  </a>
</p>

# LightCreator

See [FEATURES.md](FEATURES.md) for what the app can do, [SUPPORT.md](SUPPORT.md) for how to support the project, [INSTALL.md](INSTALL.md) for how to run it from source and [VERSIONS.md](VERSIONS.md) for the change log.

An open-source alternative to [LightBurn](https://lightburnsoftware.com/) for CNC laser engravers and cutters,
written in Rust. The interface follows the look of [VectorCraft](https://github.com/storytold/vectorcraft)
(light flat chrome, vertical tool bar, collapsible side panels, colour swatch strip, red-orange accent).

## Screenshots

![Editor, Properties tab, toolpath preview on](docs/screenshots/02-editor-properties.png)

More pictures (start window, layers, device configuration, node editing, preview, clipart gallery, other languages
and more) are on the [screenshots page](docs/SCREENSHOTS.md).

## Settings and configuration files

Language, colour scheme, device profiles (controller, units, port, camera alignment and so on), your own material
presets, the last used profile and the window size are stored by the application in a single file, `app.ron`:

| System | Location |
|--------|----------|
| macOS | `~/Library/Application Support/lightcreator/app.ron` |
| Windows | `%APPDATA%\lightcreator\data\app.ron` (usually `C:\Users\<you>\AppData\Roaming\lightcreator\data\app.ron`) |
| Linux | `$XDG_DATA_HOME/lightcreator/app.ron`, by default `~/.local/share/lightcreator/app.ron` |

Pictures you saved or downloaded for the clipart gallery are kept in the `clipart` folder next to it, together with an
`index.json` that records each picture's name, category, source, licence and author.

The file is written when the application closes. Delete it to reset everything to the defaults (the start
window then asks for a device profile again). Projects (`.lcr`), SVG and G-code files are saved wherever you
choose in the file dialog.

## Features

A short summary is below; the full list lives in [FEATURES.md](FEATURES.md).

* **Design:** shapes (including triangle, star and 3 to 360 sided polygons), Bézier node editing, text tool, boolean operations (union, intersection, subtract,
  exclusive or), shape offset, grouping, right-click context menu, SVG / Adobe Illustrator / PDF import,
  bitmap import with adjustments, image tracing.
* **Engraving:** line, fill, offset fill and raster image engraving with six dithering methods and
  grayscale power; LightBurn-style cut layers; toolpath preview with time estimate.
* **Devices:** profile per machine (controller, laser type, work area, units, serial port or TCP / ser2net,
  jog settings, camera URL), start window, live console, camera view.
* **Controllers:** GRBL and Marlin over a serial port. Ruida and Trocen jobs are exported as PLT / DXF for the
  controller's own software.
* **Helpers:** material library with starting settings and your own presets, camera / photo overlay with
  four-corner alignment.
* **Preview:** separate preview window with every burn operation in its own colour, optional travel moves and a
  simulation slider; layers can be reordered to change the burn order.
* **Interface:** flat, square window frame with its own title bar, icon control bar and arrange bar, recent files,
  native menu bar on macOS, in-window menu on Windows and Linux, seven languages
  (English, Polski, Deutsch, Italiano, Suomi, 中文, हिन्दी) and four colour schemes.

### What has and has not been verified

All geometry, boolean, offset, raster, G-code, export and translation logic is covered by unit tests, and the
screenshots are rendered from the real UI. Nothing has been run against real hardware yet: GRBL and
Marlin streaming, the Ruida / Trocen import path and the live camera still need testing on actual machines.
GRBL streaming is exercised end to end against the real GRBL 1.1h firmware running in a simulator, see
[docs/grbl-sim.md](docs/grbl-sim.md).
Reading settings from a device, material values and the Illustrator import (PDF-compatible files only) are
untested on real machines and files from Adobe. Material library values are generic starting points, always test on scrap. Complex scripts (Devanagari,
Arabic) are not shaped by the text tool.

## Building

```sh
cargo run --release -p lightcreator                      # start empty
cargo run --release -p lightcreator -- examples/badge.svg  # open a file
cargo test --workspace
```

Add `--features camera` to enable live camera capture. Requires a recent stable Rust toolchain. On Linux the GUI needs the usual X11/Wayland + OpenGL libraries
(`libxkbcommon-x11`, `libgl1`).

## Icon

The application icon lives in `assets/` (`lightcreator.icns` for macOS, `lightcreator.ico` for Windows and
PNG sizes 16–1024 px in `assets/icons/`). It is used for the window, the Dock, the Windows executable, the
About window and the Linux desktop entry.

## Layout

| Crate | Purpose |
|-------|---------|
| `crates/lc-core` | geometry, Bézier contours, boolean ops and offsets, text outlines, raster engraving, G-code and PLT / DXF export, material presets (UI independent, unit-tested) |
| `crates/lightcreator` | egui desktop application, serial worker (GRBL / Marlin), camera capture |

Coordinates are millimetres with the origin at the top-left of the work area; on output Y is flipped for
machines whose zero is at the front-left (configurable under *Laser → Device settings*).

> **Safety:** lasers are dangerous. Always test G-code at low power, wear eye protection, never leave a
> running machine unattended and verify your machine's `$30` (S-max) and `$32` (laser mode) settings.
