# LightCreator

An open-source alternative to [LightBurn](https://lightburnsoftware.com/) for CNC laser engravers and cutters,
written in Rust. The interface follows the look of [VectorCraft](https://github.com/storytold/vectorcraft)
(light flat chrome, vertical tool bar, collapsible side panels, colour swatch strip, red-orange accent).

## Features

* **Design** – rectangle, ellipse, line and polyline tools; select / move / resize with handles; numeric
  X/Y/W/H, rotate, flip, align, centre on bed, grid array, copy/paste/duplicate, undo/redo, snap to grid, rulers.
* **Import / export** – SVG import (paths, shapes, text-to-path, colours mapped to layers), SVG export,
  native `.lcr` project files (JSON), G-code export.
* **Cuts / Layers** – the LightBurn model: 30 colour layers (C00–C29), each with mode (*Line*, *Fill*,
  *Fill + Line*), speed, power, passes, fill interval, scan angle, overscan, bidirectional scanning,
  output and visibility toggles.
* **Toolpaths** – inner shapes are cut before outer ones, nearest-neighbour path ordering, even-odd
  scan-line fill, overscan, live toolpath preview with time estimate.
* **Laser control (GRBL)** – serial connect, jog, home, unlock, pause/resume, stop, framing, console,
  character-counting streaming of jobs with progress.

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
