🌐 [English](Installation) · **Polski**

# Instalacja

## Gotowe pakiety

Każde oznaczone wydanie na stronie [Releases](https://github.com/michalsarna/LightCreator/releases) zawiera:

| System | Plik | Instalacja |
|---|---|---|
| Windows | `lightcreator-vX.Y.Z-windows-x86_64-setup.exe` | uruchom instalator (wpis w menu Start + deinstalator); `.zip` to wersja przenośna |
| macOS (Apple silicon) | `lightcreator-vX.Y.Z-macos-arm64.dmg` | przeciągnij *LightCreator* do *Programów*; aplikacja nie jest podpisana, więc za pierwszym razem kliknij prawym przyciskiem → *Otwórz* |
| Debian / Ubuntu | `lightcreator-vX.Y.Z-linux-amd64.deb` | `sudo apt install ./lightcreator-*.deb`; `.tar.gz` to goły plik wykonywalny dla innych dystrybucji |

Pobrane pliki zweryfikujesz przez `SHA256SUMS.txt`. W Linuksie dodaj swojego użytkownika do grupy portów szeregowych: `sudo usermod -aG dialout $USER` (na Archu grupa `uucp`) i zaloguj się ponownie.

## Budowanie ze źródeł

Potrzebujesz aktualnego stabilnego Rusta, Gita, a w Linuksie kilku bibliotek systemowych.

### macOS
```sh
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/michalsarna/LightCreator.git && cd LightCreator
cargo run --release -p lightcreator
```
Opcjonalnie paczka aplikacji: `cargo install cargo-bundle && cd crates/lightcreator && cargo bundle --release`. Porty widać jako `/dev/cu.usbserial-*`; jeśli żadnego nie ma, zainstaluj sterownik CH340 / CP210x.

### Windows
1. Zainstaluj *Build Tools for Visual Studio* z pakietem *Desktop development with C++* (`winget install Microsoft.VisualStudio.2022.BuildTools`).
2. Zainstaluj Git (`winget install Git.Git`) i Rusta z <https://rustup.rs/>.
3. W PowerShellu: `git clone https://github.com/michalsarna/LightCreator.git; cd LightCreator; cargo run --release -p lightcreator`

Plik wykonywalny to `target\release\lightcreator.exe`. Porty to `COM3`, `COM4`… Windows 7 nie jest obsługiwany.

### Linux
```sh
# Debian / Ubuntu
sudo apt install build-essential pkg-config git curl libudev-dev libxkbcommon-x11-0 libgl1 libwayland-dev libx11-dev
# Fedora
sudo dnf install gcc pkgconf-pkg-config git curl systemd-devel libxkbcommon-x11 mesa-libGL wayland-devel libX11-devel
# Arch
sudo pacman -S base-devel git curl systemd-libs libxkbcommon-x11 mesa wayland libx11
```
Potem zainstaluj Rusta jak wyżej i `cargo run --release -p lightcreator`. Porty to `/dev/ttyUSB0` lub `/dev/ttyACM0`.

Wpis w menu: `install -Dm755 target/release/lightcreator ~/.local/bin/lightcreator`, a także `assets/lightcreator.desktop` i `assets/icons/icon-256.png` do `~/.local/share/`.

Chiński i hindi wymagają w Linuksie czcionek Noto (`fonts-noto-cjk fonts-noto-core`); pozostałe języki działają bez nich.

## Opcjonalnie: kamera na żywo

```sh
cargo run --release -p lightcreator --features camera
```
macOS poprosi o zgodę na kamerę przy pierwszym użyciu; w Linuksie potrzebne jest `/dev/video*` i grupa `video`; w Windows zezwól aplikacjom klasycznym w ustawieniach prywatności kamery. Na razie sprawdzono tylko kompilację na macOS. Zdjęcia działają w każdej wersji.

## Testy i wersja release

```sh
cargo test --workspace
cargo build --release -p lightcreator
```

Dalej: [Szybki start](Szybki-start).
