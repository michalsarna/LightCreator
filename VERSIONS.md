# Versions

Each release gets the next version number and a branch with the same name. Newest first.

## v0.04

**Devices and connections**
* Network (TCP) link, for example to `ser2net` on a Raspberry Pi; on Linux, Laser > Share over network (ser2net)…
  prepares ser2net for a laser plugged into this computer.
* Camera URL per device (MJPEG stream or JPEG snapshot) with a Camera view that floats, opens as a separate
  window or docks as a side tab; picture rotation is a device setting. The camera overlay on the work area can
  be switched on and off at any time.
* "Read from device" and the device configuration window grouped into Device, Connection and Camera.
* New layers start at 100 % power and 1000 mm/s.

**Editing**
* Round corners of shapes, of selected nodes, and between two straight lines.
* Object locking (copies are not locked), crossing selection (right to left touches, left to right wholly inside),
  Escape twice returns to the select tool, flips and rotations in their own section of the context menu.
* Auto-fit of the work area to the window; View options with the work-area background colour (light grey in dark
  schemes, white in light ones) and line thickness; thinner lines.
* Layer names in the colour strip and a layer options window (double-click a layer name).

**Preview**
* The machine zero is shown at its real corner and jobs start and end there.
* Smooth playback in job time with selectable speed; totals (time, length, passes, moves) under the picture.

**Interface**
* All icons and text fonts come from the `assets` folder (SVG icon set, Noto fonts) and are embedded.
* Icons on every menu entry, including the native macOS menu bar.
* Help > Quick guide (F1, English and Polish) and a link to the project page on GitHub.

## v0.03

**Design and editing**
* Bézier node editing (N), text tool (T) with system fonts, boolean operations (union, intersection, subtract,
  exclusive or), shape offset, grouping (layers unchanged), right-click context menu.
* Automatic shapes: triangle, five-pointed star and polygon with 3 to 360 sides.
* Image import dialog (rotate, flip, brightness, contrast, gamma, auto levels, negative, size), adjusting
  images already on the page, and tracing bitmaps into curves.
* Import and export grouped in submenus; Adobe Illustrator / PDF import (PDF-compatible files).
* Grid options: main and finer secondary grid with their own distances and colours.

**Engraving and output**
* Raster image engraving with threshold, Floyd-Steinberg, Jarvis, Stucki, Atkinson, ordered and grayscale modes.
* Offset fill layer mode.
* Layers can be reordered to change the burn order; the layer colour is shown in Properties.
* Preview window: each burn operation in its own colour, travel moves, per-operation switches, play / scrub.
* Material library with built-in starting values and your own presets.
* PLT / DXF export for Ruida and Trocen software.

**Devices**
* A profile now holds controller (GRBL, Marlin, Ruida, Trocen), laser type, units, port, jog settings and the
  camera alignment. The Device tab selects a device and shows its settings.
* Marlin streaming, live serial console tab, "Read from device" in the device configuration window.
* Start window to choose the device; the editor opens only after a profile exists.
* Millimetre or inch units per device.
* Camera overlay with four-corner perspective alignment (live capture with `--features camera`).

**Project**
* The "Cuts / Layers" tab is now "Layers". New SECURITY.md, per-OS INSTALL.md, screenshots in the README.
* Development happens on the `dev` branch; `main` and the version branches only receive releases.

## v0.02

* Native macOS menu bar (via `muda`, as in VectorCraft); Windows and Linux keep an in-window menu bar. Both
  are generated from one menu definition.
* Interface languages: English (default), Polski, Deutsch, Italiano, Suomi, 中文, हिन्दी, switchable at run time.
* Colour schemes: Dark, Light Dark, Medium Light, Light. Language and scheme are remembered.
* Application icon in several sizes, used for the window, Dock, Windows executable, About window and the
  Linux desktop entry.
* Added INSTALL.md, FEATURES.md and this file.

## v0.01

* First release: Rust / egui rewrite of the LightBurn workflow with design tools, SVG import / export,
  G-code generation, cut layers and GRBL laser control.
* About window with version and origin information.
