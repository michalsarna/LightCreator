🌐 [English](Installation) · [Polski](Instalacja) · [简体中文](Installation-ZH) · **한국어**

> ⚠️ 이 페이지는 영어판을 기계 번역한 것으로, 부정확한 부분이 있을 수 있습니다. 의심스러우면 [영어판](Installation)을 기준으로 하십시오. 수정 제안을 환영합니다.

# 설치

[릴리스 페이지](https://github.com/michalsarna/LightCreator/releases)의 모든 릴리스에는 각 시스템용으로 미리 만들어진 패키지와 압축 파일, 그리고 다운로드를 검증하기 위한 `SHA256SUMS.txt`가 들어 있습니다(Linux에서는 `sha256sum -c SHA256SUMS.txt`, macOS에서는 `shasum -a 256 -c SHA256SUMS.txt`, PowerShell에서는 `Get-FileHash`). 아래의 `vX.Y.Z`는 다운로드한 버전으로 바꾸십시오.

| 시스템 | 설치 프로그램 / 패키지 | 압축 파일 |
|---|---|---|
| Windows 10/11 (64비트) | `lightcreator-vX.Y.Z-windows-x86_64-setup.exe` | `lightcreator-vX.Y.Z-windows-x86_64.zip` |
| macOS, Apple silicon (M1 이후) | `lightcreator-vX.Y.Z-macos-arm64.dmg` | – |
| macOS, Intel | `lightcreator-vX.Y.Z-macos-x86_64.dmg` | – |
| Debian / Ubuntu 및 파생 배포판, x86_64 | `lightcreator-vX.Y.Z-linux-amd64.deb` | `lightcreator-vX.Y.Z-linux-x86_64.tar.gz` |
| Debian / Ubuntu 및 파생 배포판, ARM64 (예: Raspberry Pi 4/5, 64비트 OS) | `lightcreator-vX.Y.Z-linux-arm64.deb` | `lightcreator-vX.Y.Z-linux-arm64.tar.gz` |
| Fedora, openSUSE, x86_64 | `lightcreator-vX.Y.Z-linux-x86_64.rpm` | `lightcreator-vX.Y.Z-linux-x86_64.tar.gz` |
| Fedora, openSUSE, ARM64 | `lightcreator-vX.Y.Z-linux-aarch64.rpm` | `lightcreator-vX.Y.Z-linux-arm64.tar.gz` |
| 기타 Linux (Arch…) | – | 위의 `.tar.gz` |
| FreeBSD, x86_64 | – | `lightcreator-vX.Y.Z-freebsd-x86_64.tar.gz` |

## 패키지 관리자

릴리스는 패키지 관리자를 통해서도 받을 수 있으며, 패키지 관리자가 최신 버전을 설치하고 업데이트합니다.

| 시스템 | 명령 |
|---|---|
| macOS (Homebrew) | `brew install --cask michalsarna/tap/lightcreator` (이후 `brew upgrade`) |
| Windows (Scoop) | `scoop bucket add creator https://github.com/michalsarna/scoop-bucket` 후 `scoop install lightcreator` (이후 `scoop update lightcreator`) |
| Windows (winget) | Microsoft가 패키지를 카탈로그에 승인한 뒤 `winget install MichalSarna.LightCreator` (제출됨, 검토 대기 중) |

Homebrew cask와 Scoop 매니페스트는 매 릴리스를 자동으로 따라갑니다. macOS 앱은 공증(notarize)되지 않았으므로 처음 실행할 때는 아래 *macOS* 섹션의 안내를 참고하십시오. Arch Linux(AUR) 패키지는 준비되어 있지만 아직 게시되지 않았습니다.

## Windows

**설치 프로그램(권장).** `lightcreator-vX.Y.Z-windows-x86_64-setup.exe`를 실행합니다. 시작 메뉴 항목과 제거 프로그램(*설정 → 앱*(Settings → Apps))이 추가됩니다. 설치 프로그램은 코드 서명되지 않았으므로 Windows SmartScreen이 "Windows의 PC 보호"(Windows protected your PC)라고 표시할 수 있습니다. 이때 *추가 정보 → 실행*(More info → Run anyway)을 선택하십시오.

**포터블 zip.** `lightcreator-vX.Y.Z-windows-x86_64.zip`을 원하는 곳에 풀고 `lightcreator.exe`를 실행합니다. 그 밖에는 아무것도 설치되지 않으며, 제거하려면 폴더를 삭제하면 됩니다.

**레이저 연결.** 장치(Device) 탭에 포트가 나타나지 않으면 컨트롤러의 USB-시리얼 칩(CH340 또는 CP210x) 드라이버를 설치하십시오. 포트는 `COM3`, `COM4`… 형태로 표시됩니다. Windows 7은 지원하지 않습니다.

## macOS

**디스크 이미지(권장).** 사용 중인 Mac에 맞는 `.dmg`를 열고(*Apple 메뉴 → 이 Mac에 관하여*(Apple menu → About This Mac)에서 Apple silicon인지 Intel인지 확인할 수 있습니다) *LightCreator*를 *응용 프로그램*(Applications)으로 끌어다 놓습니다.

앱은 애드혹(ad-hoc) 서명만 되어 있고 공증되지 않았으므로, 처음에는 **마우스 오른쪽 클릭 → 열기 → 열기**(right-click → Open → Open)를 사용하십시오. macOS가 앱이 "손상되었다"(damaged)고 표시하면 격리 플래그를 한 번 제거하십시오.

```sh
xattr -cr /Applications/LightCreator.app
```


**레이저 연결.** 포트는 `/dev/cu.usbserial-*`로 나타납니다. 아무것도 보이지 않으면 CH340 / CP210x 드라이버를 설치하십시오.

## Linux

**Debian, Ubuntu, Linux Mint, Raspberry Pi OS (64비트).**

```sh
sudo apt install ./lightcreator-vX.Y.Z-linux-amd64.deb     # or ...-linux-arm64.deb
```

`apt`가 필요한 라이브러리(`libxkbcommon-x11-0`, `libgl1`)를 설치하고, 메뉴 항목을 추가하며, `lightcreator`를 경로(path)에 넣습니다. 제거하려면 `sudo apt remove lightcreator`를 사용합니다.

**Fedora, openSUSE, 최신 glibc를 사용하는 RHEL 계열 시스템.**

```sh
sudo dnf install ./lightcreator-vX.Y.Z-linux-x86_64.rpm     # or ...-linux-aarch64.rpm; openSUSE: sudo zypper install ./...
```

`dnf`가 필요한 라이브러리를 설치하고, 메뉴 항목을 추가하며, `lightcreator`를 경로에 넣습니다. 제거하려면 `sudo dnf remove lightcreator`를 사용합니다. 프로그램은 최신 Ubuntu에서 빌드되므로, 오래된 시스템(RHEL / Alma / Rocky 9)에는 필요한 glibc가 없을 수 있습니다.

**기타 배포판 (tar.gz).**

```sh
mkdir -p ~/.local/bin
mkdir lightcreator-tmp && tar xzf lightcreator-vX.Y.Z-linux-x86_64.tar.gz -C lightcreator-tmp
install -Dm755 lightcreator-tmp/lightcreator ~/.local/bin/lightcreator && rm -r lightcreator-tmp
lightcreator
```

프로그램에는 X11 키보드 및 OpenGL 라이브러리가 필요합니다: `libxkbcommon-x11`과 `mesa-libGL` / `libgl1` (Fedora: `sudo dnf install libxkbcommon-x11 mesa-libGL`; Arch: `sudo pacman -S libxkbcommon-x11 mesa`). 메뉴 항목을 만들려면 저장소의 `assets/lightcreator.desktop`과 `assets/icons/icon-256.png`를 각각 `~/.local/share/applications/`와 `~/.local/share/icons/hicolor/256x256/apps/lightcreator.png`로 복사하십시오.

**레이저 연결.** 사용자에게 시리얼 장치에 대한 접근 권한이 필요합니다: `sudo usermod -aG dialout $USER` (Arch에서는 `uucp` 그룹)를 실행한 다음 다시 로그인하십시오. 포트는 `/dev/ttyUSB0` 또는 `/dev/ttyACM0`입니다.

**글꼴.** Linux에서 중국어와 힌디어에는 Noto 글꼴이 필요합니다(Debian / Ubuntu에서는 `fonts-noto-cjk fonts-noto-core`). 다른 언어는 영향을 받지 않습니다.

## FreeBSD
`lightcreator-vX.Y.Z-freebsd-x86_64.tar.gz`를 풀고 `lightcreator`를 실행합니다. 몇 가지 패키지와 X11 또는 Wayland 데스크톱이 필요합니다(root로 실행).

```sh
pkg install libudev-devd libxkbcommon mesa-libs ffmpeg
```

`ffmpeg`는 네트워크(IP) 카메라용입니다. USB 카메라는 FreeBSD에서 아직 동작하지 않습니다(카메라 화면에 그렇게 표시됩니다). USB로 연결된 레이저는 `/dev/cuaU0`입니다. 자신을 `dialer` 그룹에 추가하고(`pw groupmod dialer -m $USER`) 다시 로그인하십시오. FreeBSD 15.0(x86_64 전용)에서 빌드되고 테스트가 실행되었지만, 아직 FreeBSD 데스크톱에서 직접 사용해 보지는 않았습니다. 그곳에서 소스로 빌드하려면 먼저 `pkg install rust pkgconf libudev-devd evdev-proto`를 실행하십시오.

## 업데이트

LightCreator는 시작할 때 새 릴리스가 있는지 확인합니다(**설정 → 프로그램 옵션…**(Settings → Program options…)에서 끌 수 있습니다). **도움말 → 업데이트 확인…**(Help → Check for updates…)은 이를 수동으로 수행하고 다운로드 페이지를 엽니다. 새 패키지를 기존 패키지 위에 설치하면 설정, 장치 프로필, 클립아트가 유지됩니다. 안전하게 하고 싶다면 **도움말 → 백업 다운로드…**(Help → Download backup…)로 먼저 zip 파일에 저장하십시오.

## 개발자용: 소스에서 빌드

최신 안정 버전의 Rust 툴체인, Git, 그리고 Linux에서는 몇 가지 시스템 라이브러리가 필요합니다.

### macOS
```sh
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/michalsarna/LightCreator.git && cd LightCreator
cargo run --release -p lightcreator
```
선택 사항인 앱 번들: `cargo install cargo-bundle && cd crates/lightcreator && cargo bundle --release`.

### Windows
1. *Desktop development with C++* 워크로드와 함께 *Build Tools for Visual Studio*를 설치합니다(`winget install Microsoft.VisualStudio.2022.BuildTools`).
2. Git(`winget install Git.Git`)과 <https://rustup.rs/>에서 Rust를 설치합니다.
3. PowerShell에서: `git clone https://github.com/michalsarna/LightCreator.git; cd LightCreator; cargo run --release -p lightcreator`

실행 파일은 `target\release\lightcreator.exe`입니다.

### Linux
```sh
# Debian / Ubuntu
sudo apt install build-essential pkg-config git curl libudev-dev libxkbcommon-x11-0 libgl1 libwayland-dev libx11-dev
# Fedora
sudo dnf install gcc pkgconf-pkg-config git curl systemd-devel libxkbcommon-x11 mesa-libGL wayland-devel libX11-devel
# Arch
sudo pacman -S base-devel git curl systemd-libs libxkbcommon-x11 mesa wayland libx11
```
그런 다음 위와 같이 Rust를 설치하고 `cargo run --release -p lightcreator`를 실행합니다.

### 선택 사항: 실시간 카메라
```sh
cargo run --release -p lightcreator --features camera
```
macOS는 처음 사용할 때 카메라 권한을 요청합니다. Linux에서는 `/dev/video*`와 `video` 그룹이 필요하고, Windows에서는 카메라 개인 정보 설정에서 데스크톱 앱을 허용해야 합니다. 지금까지는 macOS에서 컴파일 확인만 했습니다. 사진 기능은 모든 빌드에서 동작합니다.

### 테스트와 릴리스 빌드
```sh
cargo test --workspace
cargo build --release -p lightcreator
```

다음: [빠른 시작](Quick-Start-KO).
