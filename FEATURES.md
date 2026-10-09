# Features

**Design**
* Rectangle, ellipse, line, polyline and text tools; select, move and resize with handles.
* **Automatic shapes:** triangle (Y), five-pointed star (S) and regular polygon (G). The polygon tool opens a
  popup to choose the number of sides, 3 to 360. Shapes are drawn by dragging a box (Shift keeps it square)
  and are ordinary editable Bézier shapes.
* **Node editing (N):** drag nodes and Bézier handles, corner / smooth nodes, insert (double-click a segment) and
  delete nodes, turn segments into lines or curves, open and close paths. Any shape converts to curves.
* **Text tool (T):** live text from system fonts (family search, bold, italic, size, letter and line spacing,
  alignment), convertible to curves. Complex scripts such as Devanagari and Arabic are not shaped.
* **Boolean operations:** union, intersection, subtract and exclusive or of selected shapes.
* **Offset shape:** grow or shrink outlines by a distance, keeping or replacing the original.
* Numeric position and size, rotate, flip, align, centre on bed, grid array, copy / paste / duplicate.
* **Grouping:** group objects so they select and transform together; layers are not changed. Copies get their
  own group.
* **Right-click context menu** (with small icons for flips, rotations, order, alignment) with the common commands (copy, paste, group, arrange, shape operations,
  adjust and trace for images).
* Undo / redo, snap to grid, rulers, zoom and pan, drag and drop of files.
* **Grid options** (Settings menu): main grid and a finer secondary grid, each with its own line distance and
  colour.

**Files**
* **Import** submenu: SVG (curves are kept as Béziers; colours map to layers), bitmaps (PNG, JPEG, BMP, GIF,
  WebP) and Adobe Illustrator / PDF (first page, vector paths; Illustrator files saved with PDF compatibility).
* **Export** submenu: SVG, G-code, and HPGL (`.plt`) / DXF. Native `.lcr` projects embed images.
* **Image import dialog:** preview with rotate (90° steps and free angle), flip, brightness, contrast, gamma,
  auto levels, negative and size. The same dialog adjusts an image already on the page.
* **Trace image:** turn the dark (or light) areas of a bitmap into Bézier outlines with threshold, speckle
  filter, simplification and curve smoothing.

**Cuts and layers**
* 30 colour layers with mode Line, Fill, Fill + Line or Offset fill; speed, power, passes, interval, scan
  angle, overscan and bidirectional scanning; output and visibility switches.
* **Offset fill:** concentric inward rings at the layer interval.
* **Raster engraving:** images engrave line by line at the layer interval with threshold, Floyd-Steinberg,
  Jarvis, Stucki, Atkinson, ordered or grayscale power (minimum and maximum power); negative option per image.
* Inner shapes are cut first, nearest-neighbour ordering, live toolpath preview with time estimate.
* The **Layers** tab lists layers in burn order; move them up or down to change the order. The Properties tab
  shows the colour of the selected object's layer.
* **Preview window:** the whole job with each burn operation (layer, fill, line, offset, image, pass) in its own
  colour or in the layer colour, per-operation switches, optional travel moves with the laser off, the machine
  zero marked at its real corner, and smooth time-based playback (15 s fit, real time, ×10 to ×200) with a scrub slider.

**Devices and controllers**
* Start window to choose the device (machine profile) you work with; profiles are remembered.
* A profile holds the controller (GRBL, Marlin, Ruida, Trocen), laser type (diode or CO2), units, work area,
  machine zero, S-value max, travel speed, baud rate, serial port, jog step and feed, frame power and the
  camera alignment. The Device tab selects a device and shows all its settings.
* **Read from device:** the device configuration window can connect and read work area, S-value max, travel
  speed and laser mode from GRBL (and the feed rate from Marlin).
* **TCP link:** a device can connect through the network instead of a serial port, for example to `ser2net` on
  a Raspberry Pi (set the host and TCP port in the device configuration; the baud rate is configured in
  `ser2net`). Everything else works the same, including the console and reading settings.
* **Share over network (Linux):** Laser > "Share over network (ser2net)…" prepares ser2net for the laser on this
  computer: it picks the serial port, baud rate and TCP port, writes the configuration for ser2net 4.x (YAML) or
  3.x, shows the address to connect to, and can copy or save the file or install it and restart the service
  (asks for administrator rights through `pkexec`, keeps a backup of the old file, needs a confirmation). A
  local-only option is offered for use with an SSH tunnel, with a warning that the port is otherwise open to the
  network.
* **Camera view:** give the device a camera URL (an MJPEG stream or a JPEG snapshot address, http or https) and a
  "Camera view" button appears next to the preview button. The picture can also be shown as the overlay on the
  work area. RTSP streams are not supported; use an MJPEG address (most camera software offers one). The view
  can float inside the application, open as a separate operating-system window that can leave the application
  (and sit on another screen) or be docked as a tab of the side panel, and the picture can be rotated by 90°,
  180° or 270°. The rotation is a device setting (Camera rotation in the device configuration) and is saved
  with the other device options.
* The camera overlay on the work area can be switched on and off at any time: the "Overlay" button in the
  control bar and View > Show camera overlay work even when no camera window is open.
* **GRBL** and **Marlin** (laser feature, inline power) stream over a serial port or TCP: connect, jog, home, unlock,
  pause / resume, stop, framing, progress.
* **Ruida** and **Trocen** jobs are exported as HPGL / DXF with one pen or layer per colour, to be opened in the
  controller's own software (for example RDWorks). Fill is exported as hatch lines, raster images are skipped.
* Per-device display units: millimetres or inches (stored geometry stays in millimetres).
* The editor cannot be opened until at least one device profile exists.

**Helpers**
* **Material library:** starting settings for common materials and operations for diode and CO2 lasers,
  apply to the active layer, save your own presets.
* **Camera overlay:** a photo or the live camera (build with `--features camera`) under the design, aligned
  with four draggable corners (perspective corrected), opacity, flip and rotate; saved per device.

**Interface**
* Side panel with four tabs: Properties, Cuts / Layers, Device and Console (live view of everything sent to and
  received from the serial port, with status polling hidden by default).
* Native macOS menu bar; in-window menu on Windows and Linux.
* Languages: English, Polski, Deutsch, Italiano, Suomi, 中文, हिन्दी.
* Colour schemes: Dark, Light Dark, Medium Light, Light.
* Application icon in every place the platform supports.

**Not yet implemented:** direct Ruida / Trocen binary protocols, scan angles for images, kerning and complex
script shaping in text, lens-distortion correction for the camera.
