🌐 **English** · [Polski](Ustawienia-i-pliki)

# Settings and Files

Language, colour scheme, device profiles, the last used profile and the window size are stored in a single file, `app.ron`:

| System | Location |
|---|---|
| macOS | `~/Library/Application Support/lightcreator/app.ron` |
| Windows | `%APPDATA%\lightcreator\data\app.ron` |
| Linux | `$XDG_DATA_HOME/lightcreator/app.ron`, by default `~/.local/share/lightcreator/app.ron` |

* Your clipart sits in the `clipart` folder next to it, with an `index.json` recording name, category, source, licence and author.
* The file is written when the application closes. **Delete it to reset everything**; the start window then asks for a device profile again.
* Projects (`.lcr`), SVG and G-code are saved wherever you choose. **File → Open recent** lists the last 10 projects and SVG files you opened or saved (also kept in `app.ron`; *Clear list* empties it).
* After saving by hand a **File saved** message shows for 5 seconds. **Settings → Program options…** sets **auto-save** (every 120 seconds by default): work without a file name is written to a temporary folder, a named file is saved in place. Closing the program with unsaved work asks **Save**, **Discard** (deletes the temporary file and quits) or **Cancel** (back to the program).
* **Help → Download backup…** asks where to save a zip with your settings (`app.ron`, as they are now) and the whole `clipart` folder, with a README. **Help → Restore from backup…** reads such a file back (settings and clipart are replaced after a confirmation); when no profile files are found at start, the start window offers it with a green button.
* **Updates:** at start the program looks for a newer release on GitHub (**Settings → Program options…** switches that off); **Help → Check for updates…** does it by hand and a newer version opens the page with the packages.
* The window has its **own flat, square frame**: a title bar with the icon, the title and the open file, and minimise / maximise / close buttons. Drag the bar to move the window, double-click it to maximise, resize from the edges (on macOS from the right and bottom edges).
* **Settings** menu: language (English default) and colour scheme (Dark, Light Dark, Medium Light, Light, and a dark and a light one in blue, green, purple, brown and orange; default Light Dark), plus view options (grids, background of the work area, colour of the workspace around it, line thickness).
* Interface languages (30): English, Polski, Deutsch, Italiano, Suomi, 中文, हिन्दी and 23 more European languages: Български, Català, Hrvatski, Čeština, Dansk, Nederlands, Eesti, Français, Ελληνικά, Magyar, Íslenska, Gaeilge, Latviešu, Lietuvių, Malti, Norsk bokmål, Português, Română, Slovenčina, Slovenščina, Español, Svenska, Українська. The European texts are in `assets/lang/` (one file per language) and were prepared with machine help, so corrections from native speakers are welcome. Icons are SVG files and fonts (Noto Sans, Mono, SC, Devanagari) are embedded from `assets/`, so the look is identical on every system.

## Status bar and button colours

The status bar at the bottom shows the state of the device (with a dot: green when connected) and of the web server (off, on with its port, or an error). Save buttons are **green**, import buttons **blue** and delete buttons **red** with a bin.

## Printing
*File → Print…* (Ctrl+P) prints the design on a paper printer. The page is a one-page PDF made by LightCreator: the whole work area to scale (dashed border) or only the objects as large as the page allows; objects on visible layers only, or all; A4 or Letter; a margin of 0–40 mm. The page turns to landscape when the printed part is wider than tall, and a line at the bottom gives the scale. Layers are drawn in their colours, filled shapes lightly tinted, images in grey. A warning appears when objects reach outside the work area (with the work-area option they are cut off at its edge).

*Print* sends the page to the system printer: `lp` on macOS and Linux (with a choice of the printers the system lists) and the *Print* verb of the default PDF viewer on Windows (the default printer). *Save as PDF…* and *Open the PDF* (print from the viewer with more options) are the alternatives. Print preview is the PDF itself. Nothing here has been tried on real printers yet.
