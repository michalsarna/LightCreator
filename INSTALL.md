# Running LightCreator from source

LightCreator is written in Rust and runs on macOS, Windows and Linux. Nothing needs to be installed except a
Rust toolchain (and, on Linux, a few system libraries).

## 1. Install Rust

Install the stable toolchain with [rustup](https://rustup.rs/):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # macOS / Linux
# Windows: download and run rustup-init.exe from https://rustup.rs/
```

A recent stable release is required (`rustup update stable`). On Windows the installer also asks for the
Visual Studio C++ build tools; accept that.

## 2. Linux only: system libraries

Debian / Ubuntu:

```sh
sudo apt install build-essential pkg-config libudev-dev libxkbcommon-x11-0 libgl1 libwayland-dev libx11-dev
```

Fedora:

```sh
sudo dnf install gcc pkgconf-pkg-config systemd-devel libxkbcommon-x11 mesa-libGL wayland-devel libX11-devel
```

`libudev` is needed for serial-port discovery. For Chinese and Hindi text install Noto fonts
(`fonts-noto-cjk`, `fonts-noto-core`), see "Fonts" below.

## 3. Get the code and run it

```sh
git clone https://github.com/michalsarna/LightCreator.git
cd LightCreator
cargo run --release -p lightcreator                        # start with an empty design
cargo run --release -p lightcreator -- examples/badge.svg  # open a file
```

The first build downloads and compiles the dependencies and takes a few minutes. Run the tests with
`cargo test --workspace`.

To build a binary you can copy elsewhere: `cargo build --release -p lightcreator`, the result is
`target/release/lightcreator` (`lightcreator.exe` on Windows).

## Optional: macOS app bundle

```sh
cargo install cargo-bundle
cd crates/lightcreator && cargo bundle --release
```

This produces `LightCreator.app` (with the icon) under `target/release/bundle/osx/`. Launching the plain
binary also works; the Dock icon is then set at run time.

## Optional: Linux desktop entry

```sh
install -Dm755 target/release/lightcreator ~/.local/bin/lightcreator
install -Dm644 assets/lightcreator.desktop ~/.local/share/applications/lightcreator.desktop
install -Dm644 assets/icons/icon-256.png ~/.local/share/icons/hicolor/256x256/apps/lightcreator.png
```

## Serial port access (laser control)

* **Linux:** add your user to the serial group, then log in again: `sudo usermod -aG dialout $USER`
  (`uucp` on Arch).
* **macOS / Windows:** the USB-serial driver of your controller (CH340, CP210x) may need to be installed.

## Language and colour scheme

Use the **Settings** menu: *Language* (English by default, Polski, Deutsch, Italiano, Suomi, 中文, हिन्दी) and
*Colour scheme* (Dark, Light Dark, Medium Light, Light). Both choices are remembered between runs.

## Fonts

Chinese and Hindi text uses fonts that are already on your system (macOS and Windows ship them, on Linux
install the Noto fonts above). If none is found the interface still works, those two languages just show
placeholder boxes.
