🌐 [English](Installation) · **Polski**

# Instalacja

Każde wydanie na [stronie Releases](https://github.com/michalsarna/LightCreator/releases) zawiera gotowe pakiety i archiwa dla każdego systemu oraz `SHA256SUMS.txt` do sprawdzenia pobranych plików (`sha256sum -c SHA256SUMS.txt` w Linuksie, `shasum -a 256 -c SHA256SUMS.txt` w macOS, `Get-FileHash` w PowerShellu). Zamiast `vX.Y.Z` wstaw pobraną wersję.

| System | Instalator / pakiet | Archiwum |
|---|---|---|
| Windows 10/11 (64-bit) | `lightcreator-vX.Y.Z-windows-x86_64-setup.exe` | `lightcreator-vX.Y.Z-windows-x86_64.zip` |
| macOS, Apple silicon (M1 i nowsze) | `lightcreator-vX.Y.Z-macos-arm64.dmg` | – |
| macOS, Intel | `lightcreator-vX.Y.Z-macos-x86_64.dmg` | – |
| Debian / Ubuntu i pochodne, x86_64 | `lightcreator-vX.Y.Z-linux-amd64.deb` | `lightcreator-vX.Y.Z-linux-x86_64.tar.gz` |
| Debian / Ubuntu i pochodne, ARM64 (np. Raspberry Pi 4/5, system 64-bitowy) | `lightcreator-vX.Y.Z-linux-arm64.deb` | `lightcreator-vX.Y.Z-linux-arm64.tar.gz` |
| Inny Linux (Fedora, Arch, openSUSE…) | – | powyższe `.tar.gz` |

## Windows

**Instalator (zalecany).** Uruchom `lightcreator-vX.Y.Z-windows-x86_64-setup.exe`. Dodaje wpis w menu Start i deinstalator (*Ustawienia → Aplikacje*). Instalator nie jest podpisany, więc SmartScreen może napisać „System Windows ochronił ten komputer”: wybierz *Więcej informacji → Uruchom mimo to*.

**Wersja przenośna (zip).** Rozpakuj `lightcreator-vX.Y.Z-windows-x86_64.zip` gdziekolwiek i uruchom `lightcreator.exe`. Nic więcej nie jest instalowane; aby usunąć program, skasuj folder.

**Połączenie z laserem.** Jeśli w zakładce Device nie widać portu, zainstaluj sterownik układu USB-serial kontrolera (CH340 lub CP210x). Porty to `COM3`, `COM4`… Windows 7 nie jest obsługiwany.

## macOS

**Obraz dysku (zalecany).** Otwórz `.dmg` pasujący do Maca (*menu Apple → Ten Mac* pokaże, czy to Apple silicon, czy Intel) i przeciągnij *LightCreator* do *Programów*.

Aplikacja jest tylko podpisana ad hoc, bez notaryzacji, więc za pierwszym razem użyj **prawy przycisk → Otwórz → Otwórz**. Jeśli macOS pisze, że aplikacja jest „uszkodzona”, jednorazowo usuń znacznik kwarantanny:

```sh
xattr -cr /Applications/LightCreator.app
```


**Połączenie z laserem.** Porty to `/dev/cu.usbserial-*`; jeśli żaden się nie pojawia, zainstaluj sterownik CH340 / CP210x.

## Linux

**Debian, Ubuntu, Linux Mint, Raspberry Pi OS (64-bit).**

```sh
sudo apt install ./lightcreator-vX.Y.Z-linux-amd64.deb     # albo ...-linux-arm64.deb
```

`apt` doinstaluje potrzebne biblioteki (`libxkbcommon-x11-0`, `libgl1`), doda wpis w menu i umieści `lightcreator` w ścieżce. Usunięcie: `sudo apt remove lightcreator`.

**Inne dystrybucje (tar.gz).**

```sh
mkdir -p ~/.local/bin
mkdir lightcreator-tmp && tar xzf lightcreator-vX.Y.Z-linux-x86_64.tar.gz -C lightcreator-tmp
install -Dm755 lightcreator-tmp/lightcreator ~/.local/bin/lightcreator && rm -r lightcreator-tmp
lightcreator
```

Program potrzebuje bibliotek klawiatury X11 i OpenGL: `libxkbcommon-x11` oraz `mesa-libGL` / `libgl1` (Fedora: `sudo dnf install libxkbcommon-x11 mesa-libGL`; Arch: `sudo pacman -S libxkbcommon-x11 mesa`). Aby mieć wpis w menu, skopiuj z repozytorium `assets/lightcreator.desktop` i `assets/icons/icon-256.png` do `~/.local/share/applications/` oraz `~/.local/share/icons/hicolor/256x256/apps/lightcreator.png`.

**Połączenie z laserem.** Twój użytkownik musi mieć dostęp do portu szeregowego: `sudo usermod -aG dialout $USER` (grupa `uucp` w Arch), potem zaloguj się ponownie. Porty to `/dev/ttyUSB0` lub `/dev/ttyACM0`.

**Czcionki.** Chiński i hindi wymagają w Linuksie czcionek Noto (`fonts-noto-cjk fonts-noto-core` w Debianie / Ubuntu); pozostałe języki działają bez nich.

## Aktualizacja

LightCreator przy starcie sprawdza, czy jest nowsze wydanie (*Ustawienia → Opcje programu…* wyłącza to); *Pomoc → Sprawdź aktualizacje…* robi to ręcznie i otwiera stronę pobierania. Zainstaluj nowy pakiet na starym: ustawienia, profile urządzeń i clipart zostają. *Pomoc → Pobierz kopię zapasową…* zapisze je wcześniej do pliku zip.

## Dla developera: budowanie ze źródeł

Potrzebujesz aktualnego stabilnego Rusta, Gita i, w Linuksie, kilku bibliotek systemowych.

### macOS
```sh
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/michalsarna/LightCreator.git && cd LightCreator
cargo run --release -p lightcreator
```
Opcjonalnie pakiet aplikacji: `cargo install cargo-bundle && cd crates/lightcreator && cargo bundle --release`.

### Windows
1. Zainstaluj *Build Tools for Visual Studio* z pakietem *Desktop development with C++* (`winget install Microsoft.VisualStudio.2022.BuildTools`).
2. Zainstaluj Git (`winget install Git.Git`) i Rusta z <https://rustup.rs/>.
3. W PowerShellu: `git clone https://github.com/michalsarna/LightCreator.git; cd LightCreator; cargo run --release -p lightcreator`

Plik wykonywalny to `target\release\lightcreator.exe`.

### Linux
```sh
# Debian / Ubuntu
sudo apt install build-essential pkg-config git curl libudev-dev libxkbcommon-x11-0 libgl1 libwayland-dev libx11-dev
# Fedora
sudo dnf install gcc pkgconf-pkg-config git curl systemd-devel libxkbcommon-x11 mesa-libGL wayland-devel libX11-devel
# Arch
sudo pacman -S base-devel git curl systemd-libs libxkbcommon-x11 mesa wayland libx11
```
Potem zainstaluj Rusta jak wyżej i `cargo run --release -p lightcreator`.

### Opcjonalnie: żywy obraz z kamery
```sh
cargo run --release -p lightcreator --features camera
```
macOS prosi o zgodę na kamerę przy pierwszym użyciu; w Linuksie potrzebny jest `/dev/video*` i grupa `video`; w Windows zezwól aplikacjom klasycznym w ustawieniach prywatności kamery. Sprawdzone tylko kompilacją w macOS. Zdjęcia działają w każdej wersji.

### Testy i wersja release
```sh
cargo test --workspace
cargo build --release -p lightcreator
```

Dalej: [Szybki start](Szybki-start).
