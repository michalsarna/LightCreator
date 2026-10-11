# Features

**Design**
* Rectangle, ellipse, line, polyline and text tools; select, move and resize with handles.
* **Automatic shapes:** triangle (Y), five-pointed star (S), regular polygon (G), heart (K) and spiral (I). The polygon
  tool opens a small window to choose the number of sides, 3 to 360, and the spiral tool one for the number of turns
  (3 by default). The window sits in the corner of the work area, has no OK button and stays while the tool is
  active; the polygon offers a slider from 5 to 36 sides and the spiral one from 1 to 24 turns next to the number box (3 to 360 and 1 to 30), and a change also
  applies to the shape drawn last. Shapes are drawn by dragging a box (Shift keeps it square)
  and are ordinary editable Bézier shapes.
* **Node editing (N):** drag nodes and Bézier handles, corner / smooth nodes, insert (double-click a segment) and
  delete nodes, turn segments into lines or curves, open and close paths. Any shape converts to curves.
  Open it with the second tool on the tool bar (N), Arrange > Edit nodes, the right-click menu, or by
  double-clicking a shape. Pictures placed from the clipart gallery are ordinary editable shapes and open the same
  way, all parts of a grouped picture at once. Text is not converted behind your back: use Convert to curves first.
* **Text tool (T):** live text from system fonts (family search, bold, italic, size, letter and line spacing,
  alignment), convertible to curves. Complex scripts such as Devanagari and Arabic are not shaped.
* **Round corners:** Arrange > "Round corners…" replaces sharp straight corners of the selected shapes with
  arcs of a radius you give (too large a radius is limited to what fits); with two straight lines selected it joins
  them with a rounded corner instead. In the node tool the bar rounds just the selected corners.
* **Boolean operations:** union, intersection, subtract and exclusive or of selected shapes.
* **Offset shape:** grow or shrink outlines by a distance, keeping or replacing the original.
* Numeric position and size, rotate, flip, align, centre on bed, **centre on each other** (Arrange menu and
  right-click menu: the centres of the selected objects, groups counting as one, are put on the same point), grid array, copy / paste / duplicate.
* **Grouping:** group objects so they select and transform together; layers are not changed. Copies get their
  own group.
* **Locking:** lock objects or groups (Ctrl+L, Ctrl+Shift+L to unlock). Locked objects cannot be moved,
  edited or deleted but can be selected and copied; the copy is not locked. A padlock marks them.
* **Resize, turn and slant:** the square handles around a selection resize it. Click a handle (without dragging) and
  they turn into round corner handles that rotate the selection about its middle (Shift snaps to 15°) and diamond
  edge handles that slant it along that edge; click a handle again to go back to resizing. The selection frame and
  handles are red while they resize and blue while they turn and slant.
* **Selection rectangle:** dragged from left to right it selects only objects (and groups) wholly inside;
  dragged from right to left it also selects everything it touches. Pressing Escape twice returns to the
  select tool.
* **Right-click context menu** (with small icons for flips, rotations, order, alignment) with the common commands (copy, paste, group, arrange, shape operations,
  adjust and trace for images).
* Undo / redo, snap to grid, rulers, zoom and pan, drag and drop of files.
* **Window buttons:** Preview and Camera view buttons turn light blue while their window is open and close it when
  pressed again. The Overlay button follows Camera view and is always visible; it is greyed out until there is a
  picture to show (a saved photo, or the live camera while its window is open) and then switches it on the work area.
  Start / STOP are written in capitals.
* **Web server and API** (Program options, off by default, no password): a web page with the state of the machine, the project and a camera, and an API for the status, the project as SVG, the cameras and the commands stop, home, cancel jog, jog and camera choice. Listens only on this computer unless you allow the network. See the wiki page Web Server and API.
* **Delete buttons:** everything that deletes (devices, cameras, zones, macros, counters, clipart, nodes, the overlay...) has a bin icon and is red.
* **Description tab:** a plain multi-line text about the project in the side panel (it can be moved like the other tabs), saved in the project file.
* **Materials:** every device has a list of materials (a new device starts with an empty, translated *Default material* and starting points for its laser type: Plywood, Cardboard, Leather, Paper, and Acrylic for CO2; a new material is empty); a material has a name and the settings of the layers it uses. The Layers tab edits the layers of the selected material and its heading chooses, adds (a copy), renames and deletes materials; layers the design uses are added to the material with default settings. The list belongs to the device (profile, exports, backups) and to saved projects.
* **Layer cut options:** kerf offset, perforation (cut / skip lengths) and automatic tabs / bridges for line layers, saved with the layer and the project; a layer of images is always in the Image mode (mode fixed, the rest editable) and images have a scan angle. Not implemented: sub-layers and hand-placed tabs.
* **Control bar:** name of the current tool in a box of fixed width, New / Open / Save icons, switches for the grid,
  the secondary grid, snapping to the grid and snapping to other objects (while moving, edges and centres join with a guide line) (icons, highlighted while on) and the Toolpaths button, green while the toolpaths
  are shown and grey otherwise. Toolpaths are drawn in layer colours darkened or lightened until they stand out from
  the work area.
* **Tool bar groups:** selecting and node editing, then the tools that insert new objects (shapes, line,
  polyline, text), then the shape operations (union, intersection, subtract, exclusive or) and finally view
  navigation, each group set apart by a separator.
* **Arrange bar:** a narrow icon strip between the work area and the side panel with align, centre, turn, mirror,
  group / lock and order commands for the selection (hover for the names).
* The node-editing bar on the canvas hides when you click another window such as the preview, and comes back when
  you click the work area again.
* **Navigation:** pan, zoom in and zoom out sit in their own group under the drawing tools, together with two
  fit buttons, "Fit work area" (Ctrl+0) and "Fit all objects" (Ctrl+9). Holding Space switches to panning with any
  tool and returns to the previous tool on release (not while typing in a text field). Ctrl++ and Ctrl+- zoom
  around the middle of the view; the mouse wheel and the middle button still work everywhere.
* **Auto-fit:** the work area is fitted to the window and refitted whenever the window or the side panels are
  resized. Zooming or panning by hand switches it off; View > "Fit bed to window" turns it back on (and
  "Auto-fit work area to window" toggles it). The preview window behaves the same way.
* **View options** (Settings menu): main grid and a finer secondary grid, each with its own line distance and
  colour, the work-area background colour (automatically very light grey in the dark colour schemes and
  white in the light ones, or your own) and the thickness of the drawn lines.

**Files**
* **Saving:** a "File saved" message appears for 5 seconds after saving by hand. Auto-save (Settings → Program options…,
  every 120 s by default) writes work without a file name to a temporary folder and a named file in place. Closing
  with unsaved work asks: Save, Discard (also deletes the temporary file) or Cancel.
* **Open recent** (File menu, under Open): the last 10 projects and SVG files you opened or saved, newest first, kept
  between sessions; "Clear list" empties it and a file that no longer exists is dropped when picked.
* **Import** submenu: SVG (curves are kept as Béziers; colours map to layers), bitmaps (PNG, JPEG, BMP, GIF,
  WebP) and Adobe Illustrator / PDF (first page, vector paths; Illustrator files saved with PDF compatibility).
* **Print** (File > Print…, Ctrl+P): the design on a paper printer, as a one-page PDF made by the program: the whole work
  area to scale or only the objects as large as the page allows, only visible layers or all, A4 or Letter, a margin, and a
  warning when objects reach outside the work area. *Print* sends the page to the system printer (`lp` on macOS and Linux
  with a choice of printer, the PDF viewer's Print verb on Windows); *Save as PDF…* and *Open the PDF* are the alternatives.
  Layers are drawn in their colours, filled shapes lightly tinted, images in grey.
* **Export** submenu: SVG, G-code, and HPGL (`.plt`) / DXF. Native `.lcr` projects embed images.
* **Layer list:** the three switches at the end of each row (output, visible, air pump) have icon headings above
  the list; the layer colour is shown next to the title of the cut settings.
* **Tab order:** the tabs of the side panel can be dragged to another place; the order is kept in the settings and in backups.
* **Side panel layout:** Properties is laid out as a table with sections separated by lines (position and size,
  rotation, mirroring, alignment, layer) and buttons of one size. Image settings of a layer show only when the layer
  holds bitmap images. In the Device tab the jog arrows, Home, Unlock, Pause and Resume are equal icon buttons (STOP
  stays red and plain), and jog step, jog feed and frame power sit together above them. The console output has its
  own background colour.
* **Image import dialog:** preview with rotate (90° steps and free angle), flip, brightness, contrast, gamma,
  auto levels, negative and size. The same dialog adjusts an image already on the page.
* **Trace image:** turn the dark (or light) areas of a bitmap into Bézier outlines with threshold, speckle
  filter, simplification and curve smoothing.

**Cuts and layers**
* 30 colour layers with mode Line, Fill, Fill + Line or Offset fill; speed, power, passes, interval, scan
  angle, overscan and bidirectional scanning; output and visibility switches.
* **Air assist:** each layer has an "Air pump on" switch; the G-code turns the pump on (M8, or the fan on Marlin)
  when a layer that wants it starts, off when a layer without it starts, and off at the end of the job.
* **Offset fill:** concentric inward rings at the layer interval.
* **Raster engraving:** images engrave line by line at the layer interval with threshold, Floyd-Steinberg,
  Jarvis, Stucki, Atkinson, ordered or grayscale power (minimum and maximum power); negative option per image.
* Inner shapes are cut first, nearest-neighbour ordering, live toolpath preview with time estimate.
* The layer colour strip at the bottom shows the layer names. Double-clicking a layer name there or in the
  Layers tab opens the layer options window (name, mode, speed, power, passes, fill and image settings). New layers start at 100 % power and 1000 mm/min; the settings of the layers belong to the project file, together with the device profile, not in the device profile itself.
* The **Layers** tab lists layers in burn order; move them up or down to change the order. The Properties tab
  shows the colour of the selected object's layer.
* **Preview window:** playback starts at ×10; the bottom line shows the real job time at the slider position out of
  the total; the play / pause button has a fixed width.
* **Preview window (details):** the whole job with each burn operation (layer, fill, line, offset, image, pass) in its own
  colour or in the layer colour, per-operation switches, optional travel moves with the laser off, the machine
  zero marked at its real corner, and smooth time-based playback (the totals are shown under the picture) (15 s fit, real time, ×10 to ×200) with a scrub slider. The view opens zoomed to the objects to burn; the "Fit all objects" and "Fit work area" buttons switch between that and the whole work area.

**Devices and controllers**
* Start window to choose the device (machine profile) you work with; profiles are remembered.
* A profile holds the controller (GRBL, Marlin, Ruida, Trocen), laser type (diode or CO2), units, work area,
  machine zero, S-value max, travel speed, baud rate, serial port, jog step and feed, frame power and the
  camera alignment. The device configuration window is a resizable window with pages: General, Hardware, Laser, G-code, Connection (serial port or TCP) and Camera.
  The Device tab selects a device and shows all its settings.
* **Axes and limits:** machine zero in any of four corners, inverted axes and panel orientation (one design-to-machine
  transform for jog, frame, G-code and file export); margins and optional software limits (hatched / outlined on the
  canvas, jog stops at them); a **check before the job** blocks Start, Frame and export when something lies outside the
  allowed area and asks for confirmation on warnings (0 % power, nothing to burn); after connecting GRBL is read once
  and warned about laser mode off (`$32`) and a `$30` that differs from the profile.
* **G-code per device:** maximum cutting speed and acceleration (ramped time estimate, speed warning), dialects (GRBL M3 /
  M4, Marlin, Smoothieware), an Advanced page with own start / end of the job and air assist commands, power mode per
  layer (M3 / M4), and optional arcs (G2 / G3) with their own tolerance and a setting for the curve flattening.
* **Connection and firmware:** simple (one line at a time) or counting streaming, a report when a job waits for an answer
  while the machine stands still, serial port bound to its USB ids with automatic reconnect, homing / clearing an alarm
  after connecting, single-axis homing when the firmware has it, and a **Firmware ($$)** page (all GRBL settings with
  meaning and typical values, checked and confirmed writes, backup / restore with read-back, typed-word factory reset;
  `M503` text for Marlin).
* **Laser heads:** several heads per device (tool number, laser type including fibre, wavelength, power, frame and focus
  settings, beam spot, pointer offset, stored PWM values); layers choose a head, the G-code scales S by the head's S max
  and writes `T<n>`; a focus beam with a safety off switch; pointer alignment and a laser-off dry run.
* **Hooks, macros and no-go zones:** own G-code at the start and end of layers and objects with `{variables}`, named macros
  with `@include` run from the Device tab, and rectangular / round no-go zones that block Start, Frame and export when
  any move (also travel) touches them; all validated before they are kept.
* **Z axis and rotary:** an optional Z axis (range, layer heights and a step per pass, Z jog, checked heights) and a rotary
  attachment (chuck or rollers, Y or true A axis, steps per mm for `$101`, rotary layers, circumference check and marks).
* **While a job runs** the Device tab locks jog, cancel jog, home, unlock and the jog options; pause, resume and STOP
  stay. Connect is green and Disconnect purple.
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
* **Adding a device and profiles:** no predefined profiles and no wizard: one window with the basic settings (opens by
  itself when there is no device; a first material can be named at the bottom; optional read from the controller), duplicate, export and import of a profile (format
  version, strict validation, no port from the file).
* **Maintenance and notes:** working time and own counters with limits (counted from the time jobs really run) and
  per-device notes in Markdown with a preview.
* **Cameras:** several cameras per device (http / https stream or a camera of this computer), picture settings
  (brightness, contrast, white balance, noise reduction, YUYV), manual lens correction, and alignment of the overlay
  by 3 or more reference points (least-squares perspective fit).
* **Camera view:** give the device a camera URL (an MJPEG stream or a JPEG snapshot address, http or https) and a
  "Camera view" button appears next to the preview button. The picture can also be shown as the overlay on the
  work area. RTSP streams are not supported; use an MJPEG address (most camera software offers one). The view
  can float inside the application, open as a separate operating-system window that can leave the application
  (and sit on another screen) or be docked as a tab of the side panel, and the picture can be rotated by 90°,
  180° or 270°. The rotation is a device setting (Camera rotation in the device configuration) and is saved
  with the other device options. The picture can be zoomed in and out (+ / − buttons or the mouse wheel, up to 8×),
  dragged around when zoomed, and returned to the whole picture with "Default zoom" (or a double-click).
* The camera overlay on the work area can be switched on and off at any time: the "Overlay" button in the
  control bar and View > Show camera overlay work even when no camera window is open.
* **GRBL** and **Marlin** (laser feature, inline power) stream over a serial port or TCP: connect, jog, home, unlock,
  pause / resume, stop, framing, progress.
* **Ruida** and **Trocen** jobs are exported as HPGL / DXF with one pen or layer per colour, to be opened in the
  controller's own software (for example RDWorks). Fill is exported as hatch lines, raster images are skipped.
* Per-device display units: millimetres or inches (stored geometry stays in millimetres) and, separately, speeds
  per second or per minute (mm/s, mm/min, in/s, in/min). This applies to layer speed, travel speed, jog feed and the
  other speeds; values are converted automatically and the G-code always carries the mm/min feed that GRBL and Marlin
  expect after `G21`.
* The editor cannot be opened until at least one device profile exists.

**Clipart gallery**
* A side-panel tab with 310 ready-made single-colour pictures in 9 categories (animals, nature, celebrations,
  symbols, shapes, tools, food, objects, music and fun) shipped with the program, searchable by name. Each category has an arrow on its title line that folds it up or opens it again. Click a picture
  to place it on the active layer at a size you set; multi-part pictures are grouped.
* **Dots:** blue for pictures that come with the program, green for pictures you saved or downloaded.
* **Online:** search the open icon collections of Iconify from inside the program, see the collection, licence
  and author of each result (orange dot when the licence asks for credit), and save a picture on your disk under a
  category of your choice, existing or new. Pictures can also be downloaded from a web address, or added from files.
  Saved pictures keep their source, licence and author and can be renamed, moved to another category, inspected
  or deleted from the right-click menu. They live in the application's data folder next to `app.ron`, in the `clipart` folder (the location per system is in the README).

**Helpers**
* **Camera overlay:** a photo or the live camera (build with `--features camera`) under the design, aligned
  with four draggable corners (perspective corrected), opacity, flip and rotate; saved per device.

**Help**
* **Download backup…** (Help menu): asks where to save a zip file with everything the program keeps on this computer,
  the settings (`app.ron`: device profiles, language, colour scheme, recent files) and the whole
  `clipart` folder, plus a README. **Restore from backup…** (Help menu) reads such a file back: settings and clipart
  are replaced after a confirmation. When no profile files are found at start, the start window offers the same
  restore with a green button.
* **Updates:** the program looks for a newer release on GitHub when it starts (switch it off in Settings → Program
  options…) and Help → Check for updates… does it on demand. A newer version opens a window with a button to the page
  with the release packages.
* Help > Quick guide (F1): built-in documentation of the program's options with search (English and Polish),
  and Help > Project page on GitHub opens the repository.

**Interface**
* Side panel with tabs for Properties, Layers, Device, Console (live view of everything sent to and received from
  the serial port, with status polling hidden by default), Clipart and, when a camera is set, Camera view.
* **Own window frame:** no system title bar and no rounded corners. A flat bar at the top shows the icon, the
  title (with the open file) and modern minimise / maximise / close buttons; drag it to move the window, double-click
  to maximise, and resize from the window edges.
* Native macOS menu bar; in-window menu on Windows and Linux.
* Languages (30): English, Polski, Deutsch, Italiano, Suomi, 中文, हिन्दी and 23 more European languages: Български, Català, Hrvatski, Čeština, Dansk, Nederlands, Eesti, Français, Ελληνικά, Magyar, Íslenska, Gaeilge, Latviešu, Lietuvių, Malti, Norsk bokmål, Português, Română, Slovenčina, Slovenščina, Español, Svenska, Українська. The texts of the European languages are in `assets/lang/<code>.txt`, one file per language,
  and were prepared with machine help: corrections from native speakers are welcome. Russian is not included. The quick
  guide exists in English and Polish; the other languages show the English text there.
* Colour schemes: Dark, Light Dark, Medium Light, Light.
* Application icon in every place the platform supports.
* All interface icons (tool bar, menus, context menu, buttons) are SVG files and all text fonts (Noto Sans,
  Noto Sans Mono, Noto Sans SC, Noto Sans Devanagari) are font files from the `assets` folder, embedded in the
  program, so the look is the same on every system. Every menu entry has an icon, in the in-window menus (Windows,
  Linux) and in the native macOS menu bar (menu titles on macOS stay text, as the system requires).

**Not yet implemented:** direct Ruida / Trocen binary protocols, scan angles for images, kerning and complex
script shaping in text, automatic lens calibration for the camera (the lens correction is entered by hand).
