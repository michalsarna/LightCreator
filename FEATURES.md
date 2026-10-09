# Features (v0.02)

**Design**
* Rectangle, ellipse, line and polyline tools; select, move and resize with handles.
* Numeric position and size, rotate, flip, align, centre on bed, grid array, copy / paste / duplicate.
* Undo / redo, snap to grid, rulers, zoom and pan.

**Files**
* Import SVG (paths, shapes, text, colours mapped to layers), export SVG, native `.lcr` projects, G-code export.

**Cuts and layers**
* 30 colour layers with mode (Line, Fill, Fill + Line), speed, power, passes, fill interval, scan angle,
  overscan and bidirectional scanning; output and visibility switches.
* Inner shapes are cut first, nearest-neighbour ordering, live toolpath preview with time estimate.

**Laser control (GRBL)**
* Serial connect, jog, home, unlock, pause / resume, stop, framing, console, streamed jobs with progress.

**Interface**
* Native macOS menu bar; in-window menu on Windows and Linux.
* Languages: English, Polski, Deutsch, Italiano, Suomi, 中文, हिन्दी.
* Colour schemes: Dark, Light Dark, Medium Light, Light.
* Application icon in every place the platform supports.

**Not yet implemented:** raster engraving, offset fill, boolean operations, Bézier node editing, text tool,
other controllers (Ruida, Trocen, Marlin), material library, camera overlay.
