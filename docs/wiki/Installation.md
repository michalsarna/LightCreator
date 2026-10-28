🌐 **English** · [Polski](Instalacja)

# Installation

Every release on the [Releases page](https://github.com/michalsarna/LightCreator/releases) holds ready-made packages and archives for each system, plus `SHA256SUMS.txt` to verify the downloads (`sha256sum -c SHA256SUMS.txt` on Linux, `shasum -a 256 -c SHA256SUMS.txt` on macOS, `Get-FileHash` in PowerShell). Replace `vX.Y.Z` below with the version you downloaded.

| System | Installer / package | Archive |
|---|---|---|
| Windows 10/11 (64-bit) | `lightcreator-vX.Y.Z-windows-x86_64-setup.exe` | `lightcreator-vX.Y.Z-windows-x86_64.zip` |
| macOS, Apple silicon (M1 and later) | `lightcreator-vX.Y.Z-macos-arm64.dmg` | – |
| macOS, Intel | `lightcreator-vX.Y.Z-macos-x86_64.dmg` | – |
| Debian / Ubuntu and derivatives, x86_64 | `lightcreator-vX.Y.Z-linux-amd64.deb` | `lightcreator-vX.Y.Z-linux-x86_64.tar.gz` |
| Debian / Ubuntu and derivatives, ARM64 (e.g. Raspberry Pi 4/5, 64-bit OS) | `lightcreator-vX.Y.Z-linux-arm64.deb` | `lightcreator-vX.Y.Z-linux-arm64.tar.gz` |
| Other Linux (Fedora, Arch, openSUSE…) | – | the `.tar.gz` above |

## Package managers

The release is also available through package managers, which install the newest version and update it:

| System | Command |
|---|---|
| macOS (Homebrew) | `brew install --cask michalsarna/tap/lightcreator` (later `brew upgrade`) |
| Windows (Scoop) | `scoop bucket add creator https://github.com/michalsarna/scoop-bucket`, then `scoop install lightcreator` (later `scoop update lightcreator`) |
| Windows (winget) | `winget install MichalSarna.LightCreator` once Microsoft has accepted the package into its catalogue (submitted, waiting for review) |

The Homebrew cask and the Scoop manifest follow every release by themselves. The macOS app is not notarized, so the first time see the note in the *macOS* section below. An Arch Linux (AUR) package is prepared but not published yet.

## Windows

**Installer (recommended).** Run `lightcreator-vX.Y.Z-windows-x86_64-setup.exe`. It adds a Start-menu entry and an uninstaller (*Settings → Apps*). The installer is not code-signed, so Windows SmartScreen may say "Windows protected your PC": choose *More info → Run anyway*.

**Portable zip.** Unpack `lightcreator-vX.Y.Z-windows-x86_64.zip` anywhere and start `lightcreator.exe`. Nothing else is installed; delete the folder to remove it.

**Laser connection.** If no port appears in the Device tab, install the driver of your controller's USB-serial chip (CH340 or CP210x). Ports show up as `COM3`, `COM4`… Windows 7 is not supported.

## macOS

**Disk image (recommended).** Open the `.dmg` for your Mac (*Apple menu → About This Mac* tells you whether it is Apple silicon or Intel) and drag *LightCreator* to *Applications*.

The app is only ad-hoc signed, not notarized, so the first time use **right-click → Open → Open**. If macOS says the app is "damaged", remove the quarantine flag once:

```sh
xattr -cr /Applications/LightCreator.app
```


**Laser connection.** Ports appear as `/dev/cu.usbserial-*`; install the CH340 / CP210x driver if none shows up.

## Linux

**Debian, Ubuntu, Linux Mint, Raspberry Pi OS (64-bit).**

```sh
sudo apt install ./lightcreator-vX.Y.Z-linux-amd64.deb     # or ...-linux-arm64.deb
```

`apt` installs the libraries it needs (`libxkbcommon-x11-0`, `libgl1`), adds a menu entry and puts `lightcreator` on the path. Remove it with `sudo apt remove lightcreator`.

**Other distributions (tar.gz).**

```sh
mkdir -p ~/.local/bin
mkdir lightcreator-tmp && tar xzf lightcreator-vX.Y.Z-linux-x86_64.tar.gz -C lightcreator-tmp
install -Dm755 lightcreator-tmp/lightcreator ~/.local/bin/lightcreator && rm -r lightcreator-tmp
lightcreator
```

The program needs the X11 keyboard and OpenGL libraries: `libxkbcommon-x11` and `mesa-libGL` / `libgl1` (Fedora: `sudo dnf install libxkbcommon-x11 mesa-libGL`; Arch: `sudo pacman -S libxkbcommon-x11 mesa`). For a menu entry, copy `assets/lightcreator.desktop` and `assets/icons/icon-256.png` from the repository into `~/.local/share/applications/` and `~/.local/share/icons/hicolor/256x256/apps/lightcreator.png`.

**Laser connection.** Your user needs access to the serial device: `sudo usermod -aG dialout $USER` (group `uucp` on Arch), then log in again. Ports are `/dev/ttyUSB0` or `/dev/ttyACM0`.

**Fonts.** Chinese and Hindi need Noto fonts on Linux (`fonts-noto-cjk fonts-noto-core` on Debian / Ubuntu); the other languages are unaffected.

## Updating

LightCreator looks for a newer release when it starts (*Settings → Program options…* turns that off); *Help → Check for updates…* does it by hand and opens the download page. Install the new package over the old one: settings, device profiles and your clipart are kept. *Help → Download backup…* saves them to a zip first if you want to be safe.

## For developers: build from source

You need a recent stable Rust toolchain, Git and, on Linux, a few system libraries.

### macOS
```sh
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/michalsarna/LightCreator.git && cd LightCreator
cargo run --release -p lightcreator
```
Optional app bundle: `cargo install cargo-bundle && cd crates/lightcreator && cargo bundle --release`.

### Windows
1. Install *Build Tools for Visual Studio* with the *Desktop development with C++* workload (`winget install Microsoft.VisualStudio.2022.BuildTools`).
2. Install Git (`winget install Git.Git`) and Rust from <https://rustup.rs/>.
3. In PowerShell: `git clone https://github.com/michalsarna/LightCreator.git; cd LightCreator; cargo run --release -p lightcreator`

The executable is `target\release\lightcreator.exe`.

### Linux
```sh
# Debian / Ubuntu
sudo apt install build-essential pkg-config git curl libudev-dev libxkbcommon-x11-0 libgl1 libwayland-dev libx11-dev
# Fedora
sudo dnf install gcc pkgconf-pkg-config git curl systemd-devel libxkbcommon-x11 mesa-libGL wayland-devel libX11-devel
# Arch
sudo pacman -S base-devel git curl systemd-libs libxkbcommon-x11 mesa wayland libx11
```
Then install Rust as above and `cargo run --release -p lightcreator`.

### Optional: live camera
```sh
cargo run --release -p lightcreator --features camera
```
macOS asks for camera permission on first use; on Linux you need `/dev/video*` and the `video` group; on Windows allow desktop apps in the camera privacy settings. Only compile-checked on macOS so far. Photos work on every build.

### Tests and release build
```sh
cargo test --workspace
cargo build --release -p lightcreator
```

Next: [Quick Start](Quick-Start).
