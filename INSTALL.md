# Running LightCreator from source

LightCreator is written in Rust and runs on macOS, Windows and Linux. The only things you need are a Rust
toolchain, Git and (on Linux) a few system libraries. Jump to your system:
[macOS](#macos) · [Windows](#windows) · [Linux](#linux). Afterwards see [Using the app](#using-the-app).

Each section gives the same result: a working `lightcreator` program built from the repository.

## Pre-built packages

Every tagged release on the [Releases page](https://github.com/michalsarna/LightCreator/releases) contains:

| Platform | File | Install |
|---|---|---|
| Windows | `lightcreator-vX.Y.Z-windows-x86_64-setup.exe` | run the installer (Start-menu entry + uninstaller); the `.zip` is a portable copy |
| macOS (Apple silicon / Intel) | `lightcreator-vX.Y.Z-macos-arm64.dmg` / `-macos-x86_64.dmg` | drag *LightCreator* to *Applications*; the app is only ad-hoc signed (not notarized), so the first time right-click → *Open*; if macOS says it is "damaged", run `xattr -cr /Applications/LightCreator.app` |
| Debian / Ubuntu (x86_64 / ARM64) | `lightcreator-vX.Y.Z-linux-amd64.deb` / `-linux-arm64.deb` | `sudo apt install ./lightcreator-*.deb`; the `.tar.gz` is a plain binary for other distributions |

Verify downloads with `SHA256SUMS.txt`. On Linux, add yourself to the serial-port group to reach the laser
(`sudo usermod -aG dialout $USER`, then log in again). The sections below build from source instead.

## macOS

1. **Command line tools and Git** (skip if `git --version` works):

   ```sh
   xcode-select --install
   ```

2. **Rust** (stable):

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   ```

3. **Get the code and run it:**

   ```sh
   git clone https://github.com/michalsarna/LightCreator.git
   cd LightCreator
   cargo run --release -p lightcreator                        # start the app
   cargo run --release -p lightcreator -- examples/badge.svg  # open a file
   ```

4. **Optional, app bundle with icon** (the plain binary also works; it sets its Dock icon at run time):

   ```sh
   cargo install cargo-bundle
   cd crates/lightcreator && cargo bundle --release
   ```

   The result is `LightCreator.app` under `target/release/bundle/osx/`.

5. **Laser connection:** most diode and CO2 controllers use a CH340 or CP210x USB-serial chip. Install the
   vendor driver if no port shows up in the Device tab. Ports appear as `/dev/cu.usbserial-*`.

The menu is the native macOS menu bar. Chinese and Hindi text uses fonts that ship with macOS.

## Windows

1. **Visual Studio C++ build tools** (needed by Rust): install "Build Tools for Visual Studio" and select
   the *Desktop development with C++* workload, or run `winget install Microsoft.VisualStudio.2022.BuildTools`.

2. **Git:** `winget install Git.Git` (or download from <https://git-scm.com/>).

3. **Rust** (stable): download and run `rustup-init.exe` from <https://rustup.rs/>, accept the defaults, then
   open a new terminal.

4. **Get the code and run it** (PowerShell):

   ```powershell
   git clone https://github.com/michalsarna/LightCreator.git
   cd LightCreator
   cargo run --release -p lightcreator                        # start the app
   cargo run --release -p lightcreator -- examples\badge.svg  # open a file
   ```

   The executable is `target\release\lightcreator.exe`; it carries the application icon and can be copied
   anywhere.

5. **Laser connection:** install the USB-serial driver of your controller (CH340 or CP210x) if the port does
   not appear. It shows up as `COM3`, `COM4`, and so on. Windows 7 is not supported.

The menu bar is inside the window. Chinese and Hindi text uses the fonts that come with Windows
(Microsoft YaHei, Nirmala UI).

## Linux

1. **Build tools and libraries.**

   Debian / Ubuntu:

   ```sh
   sudo apt install build-essential pkg-config git curl libudev-dev libxkbcommon-x11-0 libgl1 libwayland-dev libx11-dev
   ```

   Fedora:

   ```sh
   sudo dnf install gcc pkgconf-pkg-config git curl systemd-devel libxkbcommon-x11 mesa-libGL wayland-devel libX11-devel
   ```

   Arch:

   ```sh
   sudo pacman -S base-devel git curl systemd-libs libxkbcommon-x11 mesa wayland libx11
   ```

   `libudev` is needed to list serial ports.

2. **Rust** (stable):

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   ```

3. **Get the code and run it:**

   ```sh
   git clone https://github.com/michalsarna/LightCreator.git
   cd LightCreator
   cargo run --release -p lightcreator                        # start the app
   cargo run --release -p lightcreator -- examples/badge.svg  # open a file
   ```

4. **Optional, desktop entry and icon:**

   ```sh
   install -Dm755 target/release/lightcreator ~/.local/bin/lightcreator
   install -Dm644 assets/lightcreator.desktop ~/.local/share/applications/lightcreator.desktop
   install -Dm644 assets/icons/icon-256.png ~/.local/share/icons/hicolor/256x256/apps/lightcreator.png
   ```

5. **Laser connection:** your user needs access to the serial device. Add it to the serial group and log in
   again: `sudo usermod -aG dialout $USER` (the group is `uucp` on Arch). Ports appear as `/dev/ttyUSB0` or
   `/dev/ttyACM0`.

6. **Fonts for Chinese and Hindi:** install Noto fonts, for example `sudo apt install fonts-noto-cjk
   fonts-noto-core` (Debian / Ubuntu) or `sudo dnf install google-noto-sans-cjk-fonts
   google-noto-sans-devanagari-fonts` (Fedora). Without them these two languages show empty boxes; the other
   five are unaffected.

The menu bar is inside the window on both X11 and Wayland.

## Using the app

* On start the **start window** asks which device to work with. Create at least one profile first (name,
  units, work area, machine zero, GRBL settings); the editor opens only once a profile is selected.
* **Settings** menu: *Language* (English by default; Polski, Deutsch, Italiano, Suomi, 中文, हिन्दी) and
  *Colour scheme* (Dark, Light Dark, Medium Light, Light; default Light Dark). Both are remembered.
* **Laser > Switch device…** goes back to the start window; **Laser > Device settings…** edits the active
  profile.
* Settings are stored in `app.ron`, see the "Settings and configuration files" section of the
  [README](README.md) for its location on each system. Delete the file to reset the app.

## Optional: laser behind a Raspberry Pi (ser2net)

If LightCreator itself runs on the Linux computer or Pi that the laser is plugged into, open
*Laser > Share over network (ser2net)…*: it writes the ser2net configuration for you and can install it and
restart the service (administrator rights are requested). Install ser2net first (`sudo apt install ser2net`).
By hand, on the computer with the laser:

Install `ser2net` on the Pi and expose the laser's serial port as a TCP port, for example in
`/etc/ser2net.yaml`:

```yaml
connection: &laser
  accepter: tcp,3333
  connector: serialdev,/dev/ttyUSB0,115200n81,local
```

In LightCreator open the device configuration, set *Connection type* to *Network (TCP, e.g. ser2net)* and enter
the Pi's host name or address and port 3333. The baud rate is the one in the ser2net line. A camera on the same
Pi (for example `mjpg-streamer`) can be added as the device's *Camera URL*.

## Optional: live camera

The camera overlay can load a photo on every build. Live capture is an optional feature:

```sh
cargo run --release -p lightcreator --features camera
```

macOS asks for camera permission the first time (for the terminal or the app bundle). On Linux the camera
needs a working `/dev/video*` device and your user in the `video` group; on Windows the camera privacy
setting must allow desktop apps. This feature has only been compile-checked on macOS so far.

## Tests and release build (all systems)

```sh
cargo test --workspace                       # unit tests
cargo build --release -p lightcreator        # binary in target/release/
```
