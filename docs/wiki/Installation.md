🌐 **English** · [Polski](Instalacja)

# Installation

## Pre-built packages

Every tagged release on the [Releases page](https://github.com/michalsarna/LightCreator/releases) contains:

| Platform | File | Install |
|---|---|---|
| Windows | `lightcreator-vX.Y.Z-windows-x86_64-setup.exe` | run the installer (Start-menu entry + uninstaller); the `.zip` is a portable copy |
| macOS (Apple silicon) | `lightcreator-vX.Y.Z-macos-arm64.dmg` | drag *LightCreator* to *Applications*; the app is unsigned, so right-click → *Open* the first time |
| Debian / Ubuntu | `lightcreator-vX.Y.Z-linux-amd64.deb` | `sudo apt install ./lightcreator-*.deb`; the `.tar.gz` is a plain binary for other distributions |

Verify downloads with `SHA256SUMS.txt`. On Linux add yourself to the serial-port group to reach the laser: `sudo usermod -aG dialout $USER` (group `uucp` on Arch), then log in again.

## Build from source

You need a recent stable Rust toolchain, Git and, on Linux, a few system libraries.

### macOS
```sh
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/michalsarna/LightCreator.git && cd LightCreator
cargo run --release -p lightcreator
```
Optional app bundle: `cargo install cargo-bundle && cd crates/lightcreator && cargo bundle --release`. Serial ports appear as `/dev/cu.usbserial-*`; install the CH340 / CP210x driver if none shows up.

### Windows
1. Install *Build Tools for Visual Studio* with the *Desktop development with C++* workload (`winget install Microsoft.VisualStudio.2022.BuildTools`).
2. Install Git (`winget install Git.Git`) and Rust from <https://rustup.rs/>.
3. In PowerShell: `git clone https://github.com/michalsarna/LightCreator.git; cd LightCreator; cargo run --release -p lightcreator`

The executable is `target\release\lightcreator.exe`. Ports show up as `COM3`, `COM4`… Windows 7 is not supported.

### Linux
```sh
# Debian / Ubuntu
sudo apt install build-essential pkg-config git curl libudev-dev libxkbcommon-x11-0 libgl1 libwayland-dev libx11-dev
# Fedora
sudo dnf install gcc pkgconf-pkg-config git curl systemd-devel libxkbcommon-x11 mesa-libGL wayland-devel libX11-devel
# Arch
sudo pacman -S base-devel git curl systemd-libs libxkbcommon-x11 mesa wayland libx11
```
Then install Rust as above and `cargo run --release -p lightcreator`. Ports appear as `/dev/ttyUSB0` or `/dev/ttyACM0`.

Desktop entry: `install -Dm755 target/release/lightcreator ~/.local/bin/lightcreator`, plus `assets/lightcreator.desktop` and `assets/icons/icon-256.png` into `~/.local/share/`.

Chinese and Hindi need Noto fonts on Linux (`fonts-noto-cjk fonts-noto-core`); other languages are unaffected.

## Optional: live camera

```sh
cargo run --release -p lightcreator --features camera
```
macOS asks for camera permission on first use; on Linux you need `/dev/video*` and the `video` group; on Windows allow desktop apps in the camera privacy settings. Only compile-checked on macOS so far. Photos work on every build.

## Tests and release build

```sh
cargo test --workspace
cargo build --release -p lightcreator
```

Next: [Quick Start](Quick-Start).
