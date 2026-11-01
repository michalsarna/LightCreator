# Credits

LightCreator stands on the work of many people. Thank you to everyone below, and to the countless contributors of
the projects they maintain. Licences are named for orientation; the exact terms are in each project's repository.

## Language and interface

* **[baseCreator](https://github.com/michalsarna/baseCreator)** (MIT / Apache-2.0): the code LightCreator shares with MillCreator.
* **[Rust](https://www.rust-lang.org/)** and **Cargo**: the language, compiler and package manager everything is written in.
* **[egui / eframe](https://github.com/emilk/egui)** by Emil Ernerfeldt and contributors (MIT / Apache-2.0): the immediate-mode
  interface toolkit, including `egui_extras` and the `egui_kittest` harness that renders the documentation screenshots.
* **[muda](https://github.com/tauri-apps/muda)** (Tauri, MIT / Apache-2.0) and **[objc2](https://github.com/madsmtm/objc2)**
  (MIT): the native macOS menu bar and the Dock icon.
* **[rfd](https://github.com/PolyMeilex/rfd)** (MIT): native file dialogs.

## Drawing, files and geometry

* **[resvg / usvg](https://github.com/linebender/resvg)** (MPL-2.0): reading and rendering SVG.
* **[lopdf](https://github.com/J-F-Liu/lopdf)** (MIT): reading Illustrator / PDF files.
* **[i_overlay](https://github.com/iShape-Rust/iOverlay)** (MIT): boolean operations on shapes.
* **[ttf-parser](https://github.com/harfbuzz/ttf-parser)** (MIT / Apache-2.0): outlines of text for engraving.
* **[image](https://github.com/image-rs/image)** (MIT / Apache-2.0): PNG, JPEG, BMP, GIF and WebP pictures.
* **[serde](https://serde.rs/)**, **serde_json** and **[ron](https://github.com/ron-rs/ron)** (MIT / Apache-2.0): project,
  library and settings files.
* **[zip](https://github.com/zip-rs/zip2)** (MIT) and **base64** (MIT / Apache-2.0): backups and embedded data.

## Machines and network

* **[serialport](https://github.com/serialport/serialport-rs)** (MPL-2.0): talking to the laser controller.
* **[ureq](https://github.com/algesten/ureq)** (MIT / Apache-2.0): the camera stream, online clipart and the update check.
* **[webbrowser](https://github.com/amodm/webbrowser-rs)** (MIT / Apache-2.0): opening links (the download page, GitHub) in the default browser.
* **[nokhwa](https://github.com/l1npengtul/nokhwa)** (Apache-2.0): optional live camera capture.
* **[GRBL](https://github.com/gnea/grbl)** and **[Marlin](https://marlinfw.org/)**: the controller firmwares whose G-code
  dialects LightCreator speaks. **[ser2net](https://github.com/cminyard/ser2net)** by Corey Minyard: sharing a serial port
  over the network.

## Fonts, icons and pictures

* **[Noto fonts](https://fonts.google.com/noto)** (Google and the Noto Authors, SIL Open Font License 1.1): the interface font
  and the Chinese and Hindi fonts.
* **[Lucide](https://lucide.dev)** (ISC): the interface icons.
* **[Material Design Icons](https://github.com/Templarian/MaterialDesign-SVG)** by Pictogrammers (Apache-2.0): the clipart gallery.
* See [`assets/LICENSES.md`](assets/LICENSES.md) for the file-by-file list.

## Packaging and build services

* **[cargo-bundle](https://github.com/burtonageo/cargo-bundle)** (macOS app bundle), **[cargo-deb](https://github.com/kornelski/cargo-deb)**
  (Debian packages) and **[NSIS](https://nsis.sourceforge.io/)** (Windows installer).
* **[GitHub Actions](https://github.com/features/actions)**, the Rust toolchain action by dtolnay, **rust-cache** by Swatinem
  and **CodeQL**: building, testing and releasing.

## Inspiration and people

* **[Claude](https://claude.com/claude-code)** by Anthropic, the AI coding assistant that wrote most of the code, the tests, the
  documentation and the translations together with the author (the commits say which model did the work).
* **[LightBurn](https://lightburnsoftware.com/)**: the workflow LightCreator aims to offer as open-source software.
* **[Universal G-code Sender](https://github.com/winder/Universal-G-Code-Sender)**: the format of its Designer files (`.ugsd`) is read for import; the program contains none of its code.
* **VectorCraft**: the interface that inspired the look and feel.
* Everyone who tested the program, reported problems and corrected the translations. Thank you!
* **Jakub ([CNC KpK](https://www.youtube.com/@CNC-KpK))**: ideas and suggestions for improvements, from the point of view of industrial CNC machines.
