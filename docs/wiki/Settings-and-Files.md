🌐 **English** · [Polski](Ustawienia-i-pliki)

# Settings and Files

Language, colour scheme, device profiles, your material presets, the last used profile and the window size are stored in a single file, `app.ron`:

| System | Location |
|---|---|
| macOS | `~/Library/Application Support/lightcreator/app.ron` |
| Windows | `%APPDATA%\lightcreator\data\app.ron` |
| Linux | `$XDG_DATA_HOME/lightcreator/app.ron`, by default `~/.local/share/lightcreator/app.ron` |

* Your clipart sits in the `clipart` folder next to it, with an `index.json` recording name, category, source, licence and author.
* The file is written when the application closes. **Delete it to reset everything**; the start window then asks for a device profile again.
* Projects (`.lcr`), SVG and G-code are saved wherever you choose. **File → Open recent** lists the last 10 projects and SVG files you opened or saved (also kept in `app.ron`; *Clear list* empties it).
* After saving by hand a **File saved** message shows for 5 seconds. **Settings → Program options…** sets **auto-save** (every 120 seconds by default): work without a file name is written to a temporary folder, a named file is saved in place. Closing the program with unsaved work asks **Save**, **Discard** (deletes the temporary file and quits) or **Cancel** (back to the program).
* **Help → Download backup…** asks where to save a zip with your settings (`app.ron`, as they are now) and the whole `clipart` folder, with a README on how to restore it (copy both back into the folder above while the program is closed).
* **Updates:** at start the program looks for a newer release on GitHub (**Settings → Program options…** switches that off); **Help → Check for updates…** does it by hand and a newer version opens the page with the packages.
* The window has its **own flat, square frame**: a title bar with the icon, the title and the open file, and minimise / maximise / close buttons. Drag the bar to move the window, double-click it to maximise, resize from the edges (on macOS from the right and bottom edges).
* **Settings** menu: language (English default) and colour scheme (Dark, Light Dark, Medium Light, Light; default Light Dark), plus view options (grids, background, line thickness).
* Interface languages: English, Polski, Deutsch, Italiano, Suomi, 中文, हिन्दी. Icons are SVG files and fonts (Noto Sans, Mono, SC, Devanagari) are embedded from `assets/`, so the look is identical on every system.
