# Versions

Each release gets the next version number and a branch with the same name. Newest first.

## v0.3.6

* **Tabs of open drawings:** *New* opens a new tab (at most 10) instead of replacing the drawing; each tab has its own undo, view and file, a dot marks unsaved work, **Ctrl+1…9** (0 = the last) switch tabs and **Ctrl+W** closes one. Closing the program goes through the tabs one by one and asks about each unsaved one; every tab has its own recovery file; closing the last tab closes the program.
* **Control tab** like in MillCreator and PenCreator: Home, Unlock, Pause and Resume in a block of 2 × 2 with coloured icons (also in the Device tab), a wide START and a **STOP** that is live only while the machine works, then **Outline** (the laser follows the line that wraps all the shapes, like a rubber band) and **Outline frame** (the box round them) with icons, then the position, the jog buttons, Speed, and the options that were in the Device tab (Start from, Frame power, Live laser position).
* The bar over the work area keeps the editing controls, the views (Toolpaths, Preview…, Camera view, Overlay), the size of the selection and **Save G-code**.
* The ring round the active layer colour is no longer cut off; the Generate shape… and Import windows of plugins keep their height instead of growing to the height of the program window.
* `Fit work area` / `Fit all` moved to **Ctrl+Shift+0** / **Ctrl+Shift+9** (Ctrl+0…9 switch tabs).
* baseCreator v1.20.1.

## v0.3.5

* **G-code viewer from the start screen:** the **View G-code file…** button (or a dropped file) opens the viewer straight away, without starting a project; closing it returns to the start screen. The viewer itself is now shared with MillCreator and PenCreator.
* **Navigation cube** (like in FreeCAD) in every 3D view, i.e. in the G-code viewer and in the 3D view of a plugin's import window: click a face, an edge or a corner to look from there, the arrows turn the view by 15°, drag the cube to rotate.
* The edge of the side panel no longer lights up and takes the drag while the pointer is over a window; the list in the Plugins window is not taller than its window.
* `examples/`: a small badge as a project (`badge.cre`) and its G-code.
* baseCreator v1.17.0.

## v0.3.4

* **G-code viewer:** *File → View G-code file…* (or drop a `.gcode`, `.nc`, `.tap`… file on the window) opens a view over the whole work area that shows the toolpath: flat for a laser file, in 3D when Z changes. Zoom with the wheel, pan by dragging, rotate in 3D, **Reset view**; rapid moves can be hidden, 3D paths coloured by height; size, cutting length and an estimate of the time are shown.
* **Materials window:** a place to manage materials and their layers, with export and import of materials; a material has a colour (a swatch in the Layers tab and in the list) and a layer has a description (saved in the project, shown in the tips).
* **Layers list:** a mode column (an icon with a menu) before the speed column.
* **Illustrator / PDF import:** asks whether to put the paths on layers by colour or by the Illustrator layers.
* **Plugins:** the import window has a form, a preview and a 3D view; the generator window has a preview and pictures and closes after *Create*; a field of a form can be shown only for some choices. Plugins can import a KiCad board (`.kicad_pcb`), Gerber, EAGLE and EasyEDA boards, and make boxes with finger joints for a laser (from the official list).
* **Device tab:** uses the shared XY jog pad; the dithering methods come from baseCreator (no change in the result).
* The materials window no longer grows to the height of the screen.
* egui / eframe 0.37; baseCreator v1.15.0.

## v0.3.3

* **Properties tab:** the buttons **Adjust image…** and **Trace image…** of a selected picture come from the shared editor (the same as in MillCreator and PenCreator).
* baseCreator v1.5.0.

## v0.3.2

* **Shared tools:** the operations on shapes (union, intersection, subtract, exclusive or, offset shape, round corners, convert to curves) and the right-click menu now come from the editor of baseCreator, the same as in MillCreator and PenCreator. The menu looks and works as before.
* baseCreator v1.4.0.

## v0.3.1

* **Alarm window:** an alarm of the controller, or an error in the middle of a job, opens a big window that says what happened, with a red **STOP** and an orange **UNLOCK** button; it goes away by itself when the alarm is cleared.
* **Side panel:** the tabs can be dragged into another order (remembered, also in backups), and a button collapses the panel to a narrow strip with the tab names written vertically; a click on a name opens the panel on that tab.
* **Layer bar:** more room between the colours and at both ends, so the outline of the first one is not pressed against the arrow.
* baseCreator v1.2.0.

## v0.3.0

* **Layer bar:** the selected layer's outline is no longer clipped, and the bar ends exactly at the work panel (not under the right panel).
* **baseCreator v1.1.0:** windows keep their size, double Esc selects the Select tool, and the shared texts have new translations.

## v0.2.9

* **Layer bar** under the editor, the same in all programs: one row of **64 layer colours** (arrows or the wheel scroll it; a dot marks layers with shapes). A click puts the selected shapes on that layer (or, with nothing selected, makes it the layer to draw on); a double click opens its options. One palette in all programs. Projects with fewer layers open with the rest added.
* **Autosave and recovery:** the temporary file for a project without a name is now one fixed file, offered as a big **Recover** button on the start screen after a crash, and removed at the first save (the interval and the switch are still in the program options).
* **Ctrl / Cmd** click and drag add to the selection, like Shift.
* Two new buttons at the bottom of the tool strip: **Generate shape** (plugins) and **Online clipart**.
* **Plugins:** an official list (Settings → Plugins… → Official plugins) with Install / Update; the Plugins window no longer grows to the height of the screen.
* The colours of the layers are no longer LightBurn's: the shared palette (an SVG or PDF import puts a colour on the nearest layer).
* **Security:** plugin zips are installed from exactly the bytes that were checked, with real size limits; backups are restored with limits; temporary files (print, ser2net) are private; rustls 0.23.46 (four advisories); build actions pinned.
* macOS releases are only the `.dmg` (no more `.tar.gz`).
* baseCreator v1.0.0.

## v0.2.8

* **Official plugin list** (Settings → Plugins… → Official plugins): all plugins of the repository *baseCreator-plugins* (also those for other programs) with **Install** and **Update**, checksum check, and a switch to look for updates at the start (off by default). A fractals plugin (Sierpinski, Koch, Hilbert, dragon curve, Pythagoras tree) and more are in the list. Guide for authors and the way to send a plugin: the plugins repository.
* baseCreator v0.9.9.

## v0.2.7

* **Plugins** (Settings → Plugins…): packs without code (clipart, presets, translations, data files), importers of other file formats, generators of shapes with a form and G-code postprocessors; scripts (Rhai) run in a sandbox, the output of a postprocessor is checked and controller commands are refused. File → Import → *Import with a plugin…* and *Generate shape…*. Shared with the other programs on baseCreator; see the Plugins wiki page.
* baseCreator v0.9.8.

## v0.2.6

* **Import of UGS Designer files** (`.ugsd`, File → Import → Import UGS design…, also drag and drop): shapes, rounded rectangles, ellipses, text and pictures; every way of cutting of the file (cut type, depth, feed, spindle speed) becomes a layer (outline or fill); milling feeds and depths are not used by the laser. Hidden objects are drawn too.
* **Imports (LightBurn and UGS) are really imports now:** the shapes are added to the open drawing, selected and can be undone with one step; the layers of the file are used only for layers that have no shapes yet. They no longer replace the project or appear in the recent files.
* baseCreator v0.9.7.

## v0.2.5

* **Start from line…** (Laser menu): starts a job in the middle; the laser is first brought into the state the program has at that line.
* **Calibration…** (Laser menu): steps per millimetre and squareness of the axes.
* Alarms and errors of GRBL are explained in the console and the status bar, in 30 languages.
* **Copies of the settings of the controller** are kept with every profile: one at the first connection and again whenever the settings differ from the newest copy, with a message saying what changed; make, restore and delete them on the Firmware page. They are part of the backup of the program.
* baseCreator v0.9.6.

## v0.2.4

* No more moving of settings from the old per-program folders (v0.2.3 did it once); the shared folder is simply used. baseCreator v0.9.5.

## v0.2.3

* One data folder for all programs: `baseCreator` (macOS `~/Library/Application Support/baseCreator`). The clipart gallery is now **shared** by LightCreator, MillCreator and PenCreator; settings, device profiles, materials and tools stay separate per program in `apps/<program>/app.ron`.
* The first start after the update moves the old settings and pictures into the shared folder (the old folder is left untouched, identical pictures are not duplicated). `BASECREATOR_DIR` selects another folder.
* The gallery index is written safely and re-read, so two programs open at once do not lose each other's pictures.
* Restore from backup no longer overwrites the shared gallery: the settings are replaced, the pictures are **added** (nothing is lost, nothing is doubled). A backup made by another program is refused. The zip and its dialogs now come from baseCreator, where MillCreator and PenCreator use the same ones.
* baseCreator v0.9.4.

## v0.2.2

* The Console tab now comes from baseCreator (v0.9.2), where MillCreator and PenCreator use the same one; it looks and works as before.

## v0.2.1

* An unused import left a compiler warning; it is gone.
* baseCreator v0.9.1.

## v0.2.0

**Interface** (for all three programs)
* The pointer is the tool's only over the work area (crosshair, text cursor and so on); outside it, it is the ordinary pointer.
* The line that resizes the side panel lights up red and gets thicker under the pointer, so it is clear that it moves.
* In the Layers tab the line between the list of layers and their options is dragged up and down to change the size of the list; it lights up the same way.

**Internal**
* The serial / TCP link to the controller (streaming, status, jobs) and the controller types (`Controller`, `Connection`, `Rotary`) now live in baseCreator (v0.9.0), where MillCreator uses them too.

## v0.1.10

**Start screen**
* The start screen looks and works like the one of MillCreator and PenCreator: the list of devices on top, then **New**, **Open…** and the recent files. The buttons of the device section are centred: *Add device…*, *Edit…*, *Duplicate* and *Import…* in the first row, *I have no device* in the second; *Export…* and *Delete* are in the window of *Edit…*.
* *I have no device* makes the Dummy and selects it. **New**, **Open…**, a recent file and a dropped file start the project with the selected device, or with the Dummy when there is none; a file that cannot be opened keeps the start screen. The first-run setup window no longer opens by itself and cancelling it no longer quits the program.
* Two new texts in 30 languages; the quick guide and the wiki follow.

**Internal**
* baseCreator v0.8.1 (`centered_row`).

## v0.1.9

**Projects**
* LightBurn projects are imported with **File → Import → Import LightBurn project…** instead of **File → Open**; dropping the file on the window still works. An import is not added to the recent files.
* The **Description** tab now comes from the shared baseCreator (0.8.0), so MillCreator has the same tab and the same text in the shared `.cre` project file.

**Devices**
* *I have no device* is the same in the start screen and the *Add a device* window (v0.1.8).

## v0.1.8

**Projects**
* The project file is now the shared `.cre` of baseCreator (the same file MillCreator uses). Each program keeps its
  own section and leaves the sections of the other programs untouched, so a project passes between them without losing
  anything. The old `.lcr` files are no longer read.
* The project keeps a snapshot of the machine it was made for (name, units, controller, work area, speeds). When it is
  opened with a device that differs in controller, work area or machine zero, a warning is shown.
* LightBurn projects (`.lbrn2`, `.lbrn`) can be opened with *Open* or by dropping them on the window: shapes, rounded
  rectangles, ellipses, polygons and stars, paths with curves, text as outlines, bitmaps and groups, with the speed,
  power, passes and interval of the layers. The project opens as a new, unsaved document on the selected device. What
  cannot be used (for example variable text or QR codes) is listed in a message.

**Devices**
* *I have no device* (on the start screen and in the *Add a device* window) makes a plain device called Dummy with the
  default work area and speeds and opens the editor, so a project can be prepared without a machine.

## v0.1.7

**Under the hood** (no change in what you see, apart from the details below)
* The Properties sections, the pieces of the control bar and the status bar, the start screen header, the window icon
  and the clipart gallery pictures now come from the shared baseCreator (v0.5.1). MillCreator uses the same code and
  the same 310 pictures, so the two programs look and behave alike.
* With a selection that points at shapes that no longer exist, the control bar no longer shows `0.0 mm × 0.0 mm`.
* The simulator test of a long job no longer fails when grbl-sim ends a few millimetres off the end position.

## v0.1.6

**First run**
* The *Add a device* window has a new order: name, controller, connection (type, port and a blue *Read from device*
  button), units, speed units, work area (X and Y on one line), machine zero, S-value max and laser type.
* An optional camera can be set there, with a *Test* button and the picture rotation.
* *Read from device* now works on the first run (it used to stay on *Reading…*): the button turns grey with *Read OK*
  or red with *Error reading* (in all 30 languages).
* The material field is gone (the device gets the default material), the warnings are black on orange and larger, and
  *Cancel* on the first run quits the program.

**Under the hood**
* The scene, the editor of the work area, the menus, the icons and the generic dialogs (view options, About, quick
  guide, update check, picture dialogs, clipart gallery) moved to the shared baseCreator (v0.3.0), together with the
  new MillCreator app. The `.lcr` file format and the look are unchanged (byte-identical screenshots, a test reads a
  project written by v0.1.5).

## v0.1.5

**Device**
* **Live laser Position:** a switch in the Device tab (off by default, saved with the device, greyed out without a
  connection) that shows the head as a red dot on the work area. Switched on while the machine stands still it homes
  first; during a job, a frame or a jog it only shows the dot, refreshed once a second.
* **Start from:** a device setting, *Home (machine zero)* (the default, as before) or *Current position of the laser*:
  the top-left corner of what is burned is put where the head stands, for the job, the frame and the dry run; the check
  before the job tests the moved design, and the job ends where it started instead of at the machine zero. Exported
  files and the preview keep the design where it is.
* The default jog step is 10 mm (was 5 mm); saved profiles keep their step.

**Look**
* The open-project icon on the tool bar is orange (new project blue, save green).

**Tests and wiki**
* A stuck job in the GRBL simulator tests fails after 20 s and says how many `ok` came and which replies were not a plain
  `ok`, to find the sporadic failure of the long-job test (not reproduced yet).
* The wiki has a page with all keyboard and mouse shortcuts and a page with the GRBL codes (real-time characters,
  `$` commands, the G and M codes the program writes, settings, states, `error:N` and `ALARM:N`), in English and Polish.

## v0.1.4

**Look**
* Ten new colour schemes: a dark and a light one in blue, green, purple, brown and orange (the greys and the accent
  take the colour), named *Dark Blue*, *Light Blue* and so on, in all 30 languages.
* The colour of the workspace around the work area can be chosen in the view options (automatic follows the scheme).
* The new-project icon on the tool bar is blue.

**Web server**
* Every request needs an access phrase (HTTP Basic, default `LightCreator`); it is changed in the program options with
  a Save button and applies at once; it is kept in the settings as base64 (an encoding, not encryption). There is still no encryption.
* A new web page: two columns on a computer (camera on two thirds, control panel and the project on one third), one
  column on a phone (status, camera, project, control); a crossed-out camera when there is none; the project opens
  full screen on double-click with zoom, move and reset; the colour schemes of the program (the one in use by
  default); the camera picture can be rotated, with the device's camera rotation as the default.

**Tests**
* The GRBL simulator tests wait for the `ok` that closes the `$$` report and the `$G` reply instead of an `ok` of an
  earlier line (they failed now and then on the CI).

Claude is named in `CREDITS.md`.

## v0.1.3

**Devices and materials**
* Adding a device is one window with the basic settings (name, controller, laser type, units, **speed units**, work area,
  machine zero, S max, connection) and a first material at the bottom; it opens by itself when there is no device. No
  ready-made profiles and no step-by-step wizard; *More settings…* opens the whole window.
* A device keeps a **library of materials** (at least one): each holds the layer settings. The Layers tab has a header
  to choose, add, rename and delete a material; layers in use are added with default settings, projects bring their
  material along, and the library is saved with the device. New devices come with a few starter materials.
* Cameras: the tick before the name switches a camera on (one at a time, new cameras are ticked), the *Active* field is
  gone; a **Test** button shows the picture before saving; the view mode, zoom and window place are kept per device and
  a working-time clock sits under the picture.

**Layers and G-code**
* Per layer: kerf offset, perforation, tabs / bridges and the **Image** mode with a scan angle (all off by default; the
  G-code of old projects is byte-for-byte the same).
* Snap to objects with guide lines.

**Program**
* **Print…** (Ctrl+P) as a one-page PDF to scale; a **Description** tab saved in the project; the **web server and API**
  (off by default, no password) with a page, status, camera pictures and basic controls.
* The status bar shows the device and the web server; save buttons are green, import buttons blue, delete buttons are
  red with a bin; a Home icon and four diagonal jog buttons; the update window has a Close button and its links work
  (issues #20 to #24).

## v0.1.2

Version numbers now count each part from 0 to 9 (so the last 0.x release will be v0.9.9, then v1.0.0); this release follows v0.0.11 as v0.1.2.

**Devices and safety**
* The device profile is grouped (axes, work area, speeds, G-code, connection, laser heads, rotary, maintenance, zones,
  macros, cameras, notes) in a resizable configuration window with pages; older profiles are read as they are, and
  the G-code of old profiles stays byte-for-byte the same (golden tests).
* Zero in any of four corners, inverted axes and rotated panels, work-area margins, software limits, and a **job check**
  (objects, toolpaths, power, speed, no-go zones, Z range, rotary) that blocks Start, Frame and export on errors.
* After connecting, a GRBL controller is read once; warnings for laser mode off or a different $30. The **Firmware ($$)**
  page shows the settings with their meaning, writes with validation, backs up and restores them.
* Simple streaming mode, a stalled-job report, serial port bound to its USB id, automatic reconnect, and optional
  homing / unlocking after connecting.

**G-code**
* Dialects (GRBL M3 / M4, Marlin, Smoothieware), optional arcs (G2 / G3, off by default), own start / end G-code,
  power mode per layer, maximum speed and acceleration with a ramped time estimate.
* Several laser heads (`T<n>`, S scaled by the head's S max), a focus beam with a big off button, and aiming by a
  red pointer with a dry run.
* Hooks (layer / object start and end) with variables, macros you run by hand, and no-go zones (rectangles and circles,
  optionally with a height range).
* Z axis (range, step per pass) and a rotary attachment (chuck or rollers, Y or a real A axis in degrees).

**Profiles, maintenance and cameras**
* Generic device templates (GRBL diode, GRBL CO2, K40 conversion, Marlin diode), a first-run setup, duplicate, and
  export / import of one profile (format version, strict validation, no port from the file).
* Maintenance: working time and own counters with limits. Per-device notes in Markdown with a preview.
* Several cameras per device with picture settings (brightness, contrast, white balance, noise reduction, YUYV),
  manual lens correction, and alignment of the overlay by 3 or more reference points. RTSP and automatic lens
  calibration are not supported.

**Fixes**
* Pop-up windows, About included, can be made taller again (#19).
* The MIT OR Apache-2.0 licence; a renamed parameter that CodeQL took for a hard-coded cryptographic value.

Nothing in this release has been run against real hardware; see Safety and Limitations.

## v0.0.11

**Materials**
* A material holds the settings of all the layers it needs (mode, speed, power, passes, interval, angle, overscan,
  dithering, minimum power, air assist) together with each layer's colour and label, a category, tags and a note.
* One material serves both laser types, has an id and a translated name; the built-in values can be changed locally
  (double-click to edit) and restored. Libraries: built-in, local main library, and one per device in the new
  **Materials** tab. "Delete all materials" shows the main library again with per-device copies on edit.
* Saved projects carry the device profile, its library and the material used; opening one merges the library with a
  window of differences. Libraries are exported and imported as `.lcb` files with a choice of what goes in.
* Category folding as in the clipart list, category filter, tag suggestions, layer labels picked from a list that you
  can extend, striped name-and-tag lists with scroll arrows, and a draggable border between list and editor.

**Interface**
* Side-panel tabs can be dragged into any order; the side panel no longer widens by itself.
* Questions are asked in the program's own language; pop-up windows share one style and resize both ways.
* Camera view: no more large bottom margin in its own window; the side tab is selected again when it is turned back on.

## v0.0.10

**Languages**
* 23 more European languages, 30 in all: Български, Català, Hrvatski, Čeština, Dansk, Nederlands, Eesti, Français,
  Ελληνικά, Magyar, Íslenska, Gaeilge, Latviešu, Lietuvių, Malti, Norsk bokmål, Português, Română, Slovenčina,
  Slovenščina, Español, Svenska and Українська (Russian is left out). The texts live in `assets/lang/<code>.txt`, one file
  per language, keyed by the line of `assets/lang/keys.txt`.
* They were prepared with machine help; corrections from native speakers are welcome.
* Tests check that every language file is complete, keeps the `{}` placeholders and that the interface font has a glyph
  for every character used.

## v0.0.9

**Backup and updates**
* **Help > Download backup…** saves a zip with your settings (device profiles, language, colour scheme, material
  presets, recent files) and the whole clipart folder.
* **Help > Restore from backup…** reads such a file back (after a confirmation); the start window offers it with a green
  button when no profile files are found.
* The program looks for a newer release on GitHub when it starts (switch off in Settings > Program options…);
  **Help > Check for updates…** does it by hand. A newer version opens the page with the packages.
* The About window has the slim look of the preview and camera windows.

**Release packages**
* macOS: fixed the app bundle metadata. The macOS build of v0.0.8 failed, so v0.0.8 was never published as a release;
  v0.0.9 contains everything of v0.0.8 and builds for all systems.

## v0.0.8

**Editor**
* Click a handle of the selection to swap resizing for turning and slanting: round corner handles rotate (Shift snaps to
  15°), diamond edge handles slant; the frame is red while resizing and blue while turning / slanting.
* **Spiral** tool (I) with a number-of-turns window (3 by default). The polygon and spiral windows have the slim look
  of the preview window, no OK button, stay while the tool is active, sit in the corner of the work area and a change
  in them also applies to the shape drawn last. The polygon has a slider from 5 to 36 sides, the spiral from 1 to 24 turns.

**Saving**
* A "File saved" message for 5 seconds after saving by hand.
* **Auto-save** (Settings > Program options…, every 120 s by default): work without a file name goes to a temporary
  folder, a named file is saved in place.
* Closing with unsaved work asks Save, Discard (deletes the temporary file) or Cancel; the question also appears for the
  window close button and Quit (on macOS Cmd+Q).

**Preview and device**
* Preview: slimmer bottom margin, the real job time at the slider position, ×10 by default, play / pause with icons and a
  fixed width.
* Device tab: jog, cancel jog, home, unlock and the jog options are locked while a job runs; Connect is green,
  Disconnect purple.

**Other**
* `SUPPORT.md` in English (Polish in `SUPPORT.pl.md`).
* macOS: the app bundle is ad-hoc code signed and carries a camera usage description.
* The tool-name test no longer fails in debug builds (CI).

## v0.0.7

**Window and bars**
* Own flat, square window frame with a title bar and minimise / maximise / close buttons (no system frame).
* New control bar: tool name in a fixed-width box, New / Open / Save, grid, secondary grid and snap icons, green / grey
  Toolpaths button, Preview and Camera view buttons that turn light blue while their window is open and close it when
  pressed again, Overlay button after Camera view. START / STOP in capitals; Start and Frame need a connected laser.
* **File > Open recent** with the last 10 files.
* **Arrange bar** between the work area and the side panel: align, centre, turn, mirror, group, lock and order as icons.
  Union, intersection, subtract and exclusive or on the tool bar. Double chevrons show when more icons are out of sight.
* Arrange menu regrouped (curve editing together, "Round corners…" with it; Adjust and Trace image apart), wider menus,
  new icons for node editing and trace image, "Centre on each other".

**Panels**
* Properties laid out as a table with titled sections and icon buttons of one size; Layers tab with the note on top,
  switch headings (output, visible, air pump) and the layer colour in the settings title; image settings only for
  layers that hold bitmaps.
* **Air assist** per layer (M8 / M9 for GRBL, fan commands for Marlin).
* Layer speeds are kept per device: default 1000 mm/min for every layer, saved in the device profile.
* Speed units per device: per second or per minute, millimetres or inches, converted automatically.
* Device tab: jog arrows, Home, Unlock, Pause and Resume as large icon buttons, orange cancel-jog, full-width STOP,
  frame power with the other jog options, properties in full width. Console output has its own background.
* Clipart categories fold up with an arrow on their title line.

**Preview and camera**
* The preview opens zoomed to the objects to burn; fit icons in its title line; slimmer window titles.
* Camera view: zoom in / out, drag, default zoom, icons for floating / separate window / tab, no address shown.
* The Start button turns into STOP while a job runs, and Frame is disabled meanwhile.
* Toolpaths on the work area are drawn in higher contrast.

**Documentation**
* Screenshots moved to their own page (`docs/SCREENSHOTS.md`) and re-rendered; wiki and quick guide updated.
* A test keeps the version in `app.rs`, `Cargo.toml`, `VERSIONS.md` and `SECURITY.md` in sync.

## v0.0.6

**Release packages**
* Releases now also include Linux ARM64 (`.tar.gz`, `.deb`) and macOS Intel (`.dmg`) builds.

## v0.0.5

From this release the tag is `v` plus the version in `Cargo.toml` (`v0.0.5`), as the release workflow requires.

**Clipart gallery**
* Side-panel tab with 310 bundled single-colour pictures in 9 categories, searchable, placed on the active layer
  at a chosen size (multi-part pictures are grouped). Blue dot: shipped with the program; green dot: saved or
  downloaded by you.
* Online search of open icon collections (Iconify) with licence and author shown, saving to your disk under a
  category of your choice, downloading from a web address, adding SVG files; rename, move and delete your pictures.
* Gallery pictures are ordinary shapes and can be edited node by node.

**Navigation**
* Hold Space to pan with any tool. Pan, zoom in and zoom out are a separate group on the tool bar, together
  with "Fit work area" (Ctrl+0) and "Fit all objects" (Ctrl+9); Ctrl++ and Ctrl+- zoom.

**Editing**
* Heart added to the automatic shapes. The tool bar is split into selecting / node editing, insert tools and
  navigation.
* Edit nodes from the Arrange menu, the context menu or by double-clicking a shape; text is no longer converted
  to curves behind your back.
* The node-editing bar hides when another window such as the preview is clicked.

**Project**
* SUPPORT.md and a Buy me a coffee banner; interface icons are collected from `assets/ui-icons` by the build
  script, with a test that every requested icon exists.

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
