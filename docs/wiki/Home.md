🌐 **English** · [Polski](Home-PL)

<p align="center"><img src="https://raw.githubusercontent.com/michalsarna/LightCreator/main/assets/icons/icon-256.png" alt="LightCreator" width="128"></p>

# LightCreator

An open-source alternative to [LightBurn](https://lightburnsoftware.com/) for CNC laser engravers and cutters, written in Rust. Design, lay out cuts and engravings, preview the toolpath and stream it to your machine, on macOS, Windows and Linux.

![Editor](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/02-editor-properties.png)

## Where to start

| I want to… | Go to |
|---|---|
| Install the program | [Installation](Installation) |
| Make my first burn | [Quick Start](Quick-Start) |
| Draw, edit and import | [Design Tools](Design-Tools) |
| Set speed, power, fill, images | [Cuts and Layers](Cuts-and-Layers) |
| Connect a laser or camera | [Devices and Controllers](Devices-and-Controllers) |
| Use ready-made pictures | [Clipart Gallery](Clipart-Gallery) |
| Look up a key | [Keyboard Shortcuts](Keyboard-Shortcuts) |
| Find the settings file | [Settings and Files](Settings-and-Files) |
| Read about the code | [Architecture](Architecture) |
| Test without a laser | [GRBL Simulator](GRBL-Simulator) |
| Know what is tested | [Safety and Limitations](Safety-and-Limitations) |

## At a glance

* **Design:** shapes, Bézier node editing, text, boolean operations, offset, round corners, grouping, locking; SVG / Illustrator / PDF and bitmap import, image tracing.
* **Engraving:** line, fill, offset fill and raster images with six dithering methods and grayscale power; 30 LightBurn-style cut layers; preview with time estimate.
* **Devices:** per-machine profiles; GRBL and Marlin over serial or TCP (ser2net); Ruida and Trocen via PLT / DXF export.
* **Helpers:** material library, camera overlay with four-corner alignment, 310-picture clipart gallery with online search.
* **Interface:** seven languages (English, Polski, Deutsch, Italiano, Suomi, 中文, हिन्दी) and four colour schemes.

> ⚠️ **Safety:** lasers are dangerous. Test G-code at low power, wear eye protection, never leave a running machine unattended. See [Safety and Limitations](Safety-and-Limitations).

Latest release: **v0.0.5** · [Releases](https://github.com/michalsarna/LightCreator/releases) · [Issues](https://github.com/michalsarna/LightCreator/issues) · MIT licence
