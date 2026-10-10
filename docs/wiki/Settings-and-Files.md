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
* Projects (`.lcr`), SVG and G-code are saved wherever you choose.
* **Settings** menu: language (English default) and colour scheme (Dark, Light Dark, Medium Light, Light; default Light Dark), plus view options (grids, background, line thickness).
* Interface languages: English, Polski, Deutsch, Italiano, Suomi, 中文, हिन्दी. Icons are SVG files and fonts (Noto Sans, Mono, SC, Devanagari) are embedded from `assets/`, so the look is identical on every system.
