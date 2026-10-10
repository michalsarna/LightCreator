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

| | |
|---|---|
| ![Start window](docs/screenshots/01-start.png) | ![Properties tab](docs/screenshots/02-editor-properties.png) |
| Start window: pick the device to work with | Editor, Properties tab, toolpath preview on |
| ![Cuts / Layers tab](docs/screenshots/03-layers.png) | ![Console tab](docs/screenshots/04-console.png) |
| Cuts / Layers tab | Console tab: live view of the serial traffic |
| ![Device configuration](docs/screenshots/05-device-config.png) | ![Light scheme, Polish](docs/screenshots/06-light-polish.png) |
| Device configuration (controller, units, work area, port, jog) | Light colour scheme with the Polish interface |
| ![Node editing](docs/screenshots/07-nodes.png) | ![Text tool](docs/screenshots/08-text.png) |
| Bézier node editing with its floating toolbar | Text tool and text properties |
| ![Image engraving](docs/screenshots/09-image.png) | ![Camera overlay](docs/screenshots/10-camera-overlay.png) |
| Raster image engraving, dithered toolpath preview | Camera overlay aligned with four corners |
| ![Material library](docs/screenshots/11-materials.png) | ![Preview window](docs/screenshots/12-preview.png) |
| Material library | Preview window: every burn operation in its own colour, with travel moves |
| ![Import image](docs/screenshots/13-image-import.png) | ![Trace image](docs/screenshots/14-trace.png) |
| Image import with rotate, flip, brightness, contrast and gamma | Trace an image into curves |
| ![Grid options](docs/screenshots/15-grid-layers.png) | ![Automatic shapes](docs/screenshots/16-shapes.png) |
| Main and secondary grid options, layers in burn order | Triangle, star and polygon tools with the sides popup |
| ![Camera view as a tab](docs/screenshots/17-camera-tab.png) | ![Share over network](docs/screenshots/18-ser2net.png) |
| Camera view docked as a side tab, rotated 90° | Sharing the laser's serial port with ser2net (Linux) |
| ![Layer options and locking](docs/screenshots/19-layers-lock.png) | ![Chinese interface](docs/screenshots/20-chinese.png) |
| Layer names in the strip, layer options window, locked object, light work area in a dark scheme | Chinese interface, fonts and icons from the assets folder |
| ![Hindi interface](docs/screenshots/21-hindi.png) | ![Quick guide and round corners](docs/screenshots/22-round-help.png) |
| Hindi interface | Built-in quick guide (F1) and the round corners window |
| ![Rounded shapes](docs/screenshots/23-rounded-shapes.png) | ![Clipart gallery](docs/screenshots/24-clipart.png) |
| Rounded rectangle and triangle | Clipart gallery: blue dots ship with the program, green dots are yours |
| ![Online clipart](docs/screenshots/25-clipart-online.png) | ![Node editing of a gallery picture](docs/screenshots/26-clipart-nodes.png) |
| Searching open icon collections and saving into a category | A gallery picture opened in the node tool |

The pictures are rendered headlessly from the real UI; regenerate them with
`cargo test -p lightcreator --release render_screenshots -- --ignored`.

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
* **Interface:** native menu bar on macOS, in-window menu on Windows and Linux, seven languages
  (English, Polski, Deutsch, Italiano, Suomi, 中文, हिन्दी) and four colour schemes.

### What has and has not been verified

All geometry, boolean, offset, raster, G-code, export and translation logic is covered by unit tests, and the
screenshots below are rendered from the real UI. Nothing has been run against real hardware yet: GRBL and
Marlin streaming, the Ruida / Trocen import path and the live camera still need testing on actual machines.
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
