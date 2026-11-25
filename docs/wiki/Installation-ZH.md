🌐 [English](Installation) · [Polski](Instalacja) · **简体中文** · [한국어](Installation-KO)

> ⚠️ 本页由英文版机器翻译而成，可能有不准确之处。如有疑问，请以[英文版](Installation)为准。欢迎提出修改建议。

# 安装

[Releases 页面](https://github.com/michalsarna/LightCreator/releases) 上的每个版本都包含适用于各个系统的现成安装包和压缩包，另外还有用于校验下载文件的 `SHA256SUMS.txt`（Linux 上用 `sha256sum -c SHA256SUMS.txt`，macOS 上用 `shasum -a 256 -c SHA256SUMS.txt`，PowerShell 中用 `Get-FileHash`）。请将下文中的 `vX.Y.Z` 替换为你下载的版本号。

| 系统 | 安装程序 / 软件包 | 压缩包 |
|---|---|---|
| Windows 10/11（64 位） | `lightcreator-vX.Y.Z-windows-x86_64-setup.exe` | `lightcreator-vX.Y.Z-windows-x86_64.zip` |
| macOS，Apple 芯片（M1 及更新） | `lightcreator-vX.Y.Z-macos-arm64.dmg` | – |
| macOS，Intel | `lightcreator-vX.Y.Z-macos-x86_64.dmg` | – |
| Debian / Ubuntu 及其衍生版，x86_64 | `lightcreator-vX.Y.Z-linux-amd64.deb` | `lightcreator-vX.Y.Z-linux-x86_64.tar.gz` |
| Debian / Ubuntu 及其衍生版，ARM64（例如 Raspberry Pi 4/5，64 位系统） | `lightcreator-vX.Y.Z-linux-arm64.deb` | `lightcreator-vX.Y.Z-linux-arm64.tar.gz` |
| Fedora、openSUSE，x86_64 | `lightcreator-vX.Y.Z-linux-x86_64.rpm` | `lightcreator-vX.Y.Z-linux-x86_64.tar.gz` |
| Fedora、openSUSE，ARM64 | `lightcreator-vX.Y.Z-linux-aarch64.rpm` | `lightcreator-vX.Y.Z-linux-arm64.tar.gz` |
| 其他 Linux（Arch…） | – | 上面的 `.tar.gz` |
| FreeBSD，x86_64 | – | `lightcreator-vX.Y.Z-freebsd-x86_64.tar.gz` |

## 包管理器

每个版本也可以通过包管理器获取，包管理器会安装最新版本并负责更新：

| 系统 | 命令 |
|---|---|
| macOS（Homebrew） | `brew install --cask michalsarna/tap/lightcreator`（之后用 `brew upgrade`） |
| Windows（Scoop） | `scoop bucket add creator https://github.com/michalsarna/scoop-bucket`，然后 `scoop install lightcreator`（之后用 `scoop update lightcreator`） |
| Windows（winget） | 待 Microsoft 将该软件包收录进其目录后使用 `winget install MichalSarna.LightCreator`（已提交，等待审核） |

Homebrew cask 和 Scoop 清单会自动跟随每个版本更新。macOS 应用未经公证（notarized），因此首次运行时请参阅下方 *macOS* 一节中的说明。Arch Linux（AUR）软件包已准备好，但尚未发布。

## Windows

**安装程序（推荐）。**运行 `lightcreator-vX.Y.Z-windows-x86_64-setup.exe`。它会添加一个开始菜单项和一个卸载程序（*设置 → 应用*（*Settings → Apps*））。安装程序没有代码签名，因此 Windows SmartScreen 可能会提示“Windows 已保护你的电脑”（"Windows protected your PC"）：请选择 *更多信息 → 仍要运行*（*More info → Run anyway*）。

**便携版 zip。**将 `lightcreator-vX.Y.Z-windows-x86_64.zip` 解压到任意位置，然后启动 `lightcreator.exe`。不会安装任何其他内容；删除该文件夹即可卸载。

**激光器连接。**如果“设备”（Device）标签页中没有出现端口，请安装控制器 USB 串口芯片（CH340 或 CP210x）的驱动程序。端口显示为 `COM3`、`COM4`…… 不支持 Windows 7。

## macOS

**磁盘映像（推荐）。**打开与你的 Mac 对应的 `.dmg`（*苹果菜单 → 关于本机*（*Apple menu → About This Mac*）会告诉你它是 Apple 芯片还是 Intel），然后将 *LightCreator* 拖到 *应用程序*（*Applications*）中。

该应用只有临时（ad-hoc）签名，未经公证，因此首次运行时请使用 **右键点按 → 打开 → 打开**（**right-click → Open → Open**）。如果 macOS 提示该应用“已损坏”（"damaged"），请执行一次以下命令移除隔离标记：

```sh
xattr -cr /Applications/LightCreator.app
```


**激光器连接。**端口显示为 `/dev/cu.usbserial-*`；如果没有出现任何端口，请安装 CH340 / CP210x 驱动程序。

## Linux

**Debian、Ubuntu、Linux Mint、Raspberry Pi OS（64 位）。**

```sh
sudo apt install ./lightcreator-vX.Y.Z-linux-amd64.deb     # or ...-linux-arm64.deb
```

`apt` 会安装所需的库（`libxkbcommon-x11-0`、`libgl1`），添加菜单项，并将 `lightcreator` 放入 PATH。使用 `sudo apt remove lightcreator` 卸载。

**Fedora、openSUSE 以及带有较新 glibc 的类 RHEL 系统。**

```sh
sudo dnf install ./lightcreator-vX.Y.Z-linux-x86_64.rpm     # or ...-linux-aarch64.rpm; openSUSE: sudo zypper install ./...
```

`dnf` 会安装所需的库，添加菜单项，并将 `lightcreator` 放入 PATH。使用 `sudo dnf remove lightcreator` 卸载。本程序是在较新的 Ubuntu 上构建的，因此较旧的系统（RHEL / Alma / Rocky 9）可能缺少所需的 glibc。

**其他发行版（tar.gz）。**

```sh
mkdir -p ~/.local/bin
mkdir lightcreator-tmp && tar xzf lightcreator-vX.Y.Z-linux-x86_64.tar.gz -C lightcreator-tmp
install -Dm755 lightcreator-tmp/lightcreator ~/.local/bin/lightcreator && rm -r lightcreator-tmp
lightcreator
```

本程序需要 X11 键盘库和 OpenGL 库：`libxkbcommon-x11` 以及 `mesa-libGL` / `libgl1`（Fedora：`sudo dnf install libxkbcommon-x11 mesa-libGL`；Arch：`sudo pacman -S libxkbcommon-x11 mesa`）。如需菜单项，请将仓库中的 `assets/lightcreator.desktop` 和 `assets/icons/icon-256.png` 分别复制到 `~/.local/share/applications/` 和 `~/.local/share/icons/hicolor/256x256/apps/lightcreator.png`。

**激光器连接。**你的用户需要有访问串口设备的权限：`sudo usermod -aG dialout $USER`（Arch 上为 `uucp` 组），然后重新登录。端口为 `/dev/ttyUSB0` 或 `/dev/ttyACM0`。

**字体。**在 Linux 上，中文和印地语需要 Noto 字体（Debian / Ubuntu 上为 `fonts-noto-cjk fonts-noto-core`）；其他语言不受影响。

## FreeBSD
解压 `lightcreator-vX.Y.Z-freebsd-x86_64.tar.gz` 并运行 `lightcreator`。它需要几个软件包以及 X11 或 Wayland 桌面（以 root 身份安装）：

```sh
pkg install libudev-devd libxkbcommon mesa-libs ffmpeg
```

`ffmpeg` 用于网络（IP）摄像头；USB 摄像头目前在 FreeBSD 上还不能使用（摄像头界面会给出提示）。USB 上的激光器为 `/dev/cuaU0`：请将自己加入 `dialer` 组（`pw groupmod dialer -m $USER`，然后重新登录）。本程序已在 FreeBSD 15.0 上构建并运行了测试（仅 x86_64）；尚未在 FreeBSD 桌面上实际试用。如需在该系统上从源码构建：请先执行 `pkg install rust pkgconf libudev-devd evdev-proto`。

## 更新

LightCreator 在启动时会检查是否有更新的版本（可在 **设置 → 程序选项…**（**Settings → Program options…**）中关闭）；**帮助 → 检查更新…**（**Help → Check for updates…**）可手动检查并打开下载页面。直接在旧版本之上安装新软件包即可：设置、设备配置文件和你的剪贴画都会保留。如果想更稳妥，可以先用 **帮助 → 下载备份…**（**Help → Download backup…**）将它们保存为一个 zip 文件。

## 面向开发者：从源码构建

你需要较新的稳定版 Rust 工具链、Git，在 Linux 上还需要一些系统库。

### macOS
```sh
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/michalsarna/LightCreator.git && cd LightCreator
cargo run --release -p lightcreator
```
可选的应用包（app bundle）：`cargo install cargo-bundle && cd crates/lightcreator && cargo bundle --release`。

### Windows
1. 安装 *Build Tools for Visual Studio*，并选择 *使用 C++ 的桌面开发*（*Desktop development with C++*）工作负载（`winget install Microsoft.VisualStudio.2022.BuildTools`）。
2. 安装 Git（`winget install Git.Git`），并从 <https://rustup.rs/> 安装 Rust。
3. 在 PowerShell 中：`git clone https://github.com/michalsarna/LightCreator.git; cd LightCreator; cargo run --release -p lightcreator`

可执行文件为 `target\release\lightcreator.exe`。

### Linux
```sh
# Debian / Ubuntu
sudo apt install build-essential pkg-config git curl libudev-dev libxkbcommon-x11-0 libgl1 libwayland-dev libx11-dev
# Fedora
sudo dnf install gcc pkgconf-pkg-config git curl systemd-devel libxkbcommon-x11 mesa-libGL wayland-devel libX11-devel
# Arch
sudo pacman -S base-devel git curl systemd-libs libxkbcommon-x11 mesa wayland libx11
```
然后按上述方法安装 Rust，并执行 `cargo run --release -p lightcreator`。

### 可选：实时摄像头
```sh
cargo run --release -p lightcreator --features camera
```
macOS 会在首次使用时请求摄像头权限；在 Linux 上你需要 `/dev/video*` 和 `video` 组；在 Windows 上请在摄像头隐私设置中允许桌面应用访问。目前仅在 macOS 上做过编译检查。照片功能在所有构建中都可用。

### 测试和发布构建
```sh
cargo test --workspace
cargo build --release -p lightcreator
```

下一步：[快速入门](Quick-Start-ZH)。
