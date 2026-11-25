🌐 [English](Settings-and-Files) · [Polski](Ustawienia-i-pliki) · **简体中文** · [한국어](Settings-and-Files-KO)

> ⚠️ 本页由英文版机器翻译而成，可能有不准确之处。如有疑问，请以[英文版](Settings-and-Files)为准。欢迎提出修改建议。

# 设置与文件

语言、配色方案、设备配置文件、上次使用的配置文件以及窗口大小都保存在一个文件 `app.ron` 中：

| 系统 | 位置 |
|---|---|
| macOS | `~/Library/Application Support/baseCreator/apps/lightcreator/app.ron` |
| Windows | `%APPDATA%\baseCreator\apps\lightcreator\app.ron` |
| Linux | `$XDG_DATA_HOME/baseCreator/apps/lightcreator/app.ron`，默认为 `~/.local/share/baseCreator/apps/lightcreator/app.ron` |

* **共享文件夹。**LightCreator、MillCreator 和 PenCreator 将数据保存在同一个文件夹 `baseCreator` 中（上表显示的是其中本程序自己的设置）。**剪贴画库是共享的**：在一个程序中保存或下载的图片会出现在其他程序中（`apps` 旁边的 `clipart` 文件夹，其中的 `index.json` 记录名称、类别、来源、许可证和作者）。每个程序在 `apps/<program>/app.ron` 中保存自己的设置、设备配置文件、材料和刀具。设置环境变量 `BASECREATOR_DIR` 可以使用其他文件夹。
* 该文件在应用程序关闭时写入。**删除它即可重置所有内容**；之后启动窗口会再次要求选择设备配置文件。
* 项目（`.cre`）、SVG 和 G-code 保存在你选择的任何位置。`.cre` 文件与 MillCreator 以及其他基于 baseCreator 的程序共用：它包含绘图、描述以及每个程序各自的一个部分。LightCreator 写入它自己的 `laser` 部分（项目所针对机器的名称和工作区 X/Y/Z、所选材料及其图层、图层顺序），并在保存时保持其他程序的部分不变。它从不写入控制器、连接、端口、摄像头或其他设备设置；打开为不同工作区的机器制作的项目时会显示警告，你所选择的设备继续使用。**文件 → 最近打开**（**File → Open recent**）列出你最近打开或保存的 10 个项目和 SVG 文件（同样保存在 `app.ron` 中；**清除列表**（**Clear list**）会清空它；启动窗口显示前五个）。项目文件还保存 **描述**（**Description**）标签页中的备注（会以 `; Note:` 注释写入 G-code）以及摄像头叠加层的叠加图片。
* 手动保存后，会显示 **文件已保存**（**File saved**）消息 5 秒。**设置 → 程序选项…**（**Settings → Program options…**）用于设置 **自动保存**（**auto-save**）（默认每 120 秒一次）：没有文件名的工作会写入一个临时文件夹，有名称的文件会原地保存。每个打开的绘图在工作区上方都有一个 **标签页**（**tab**）（**文件 → 新建**（**File → New**）、**打开…**（**Open…**）以及标签栏上的 **+** 会创建标签页，最多 10 个；圆点标记有未保存工作的标签页；Ctrl+1 … Ctrl+0 用于显示它们，Ctrl+W 关闭一个）。工作区可以拆分为最多三个标签页窗格，参见 [设计工具](Design-Tools-ZH)。在有未保存工作时关闭程序，会逐个标签页询问 **保存**（**Save**）、**放弃**（**Discard**）（删除临时文件并退出）或 **取消**（**Cancel**）（返回程序）。
* **帮助 → 下载备份…**（**Help → Download backup…**）会询问将一个 zip 文件保存在哪里，其中包含你的设置（`app.ron`，即当前状态）和整个共享 `clipart` 图库，并附有 README。**帮助 → 从备份恢复…**（**Help → Restore from backup…**）读回这样的文件（设置会在确认后被替换，图片会 **添加** 到图库中，因此 MillCreator 或 PenCreator 在此期间保存的内容不会丢失；MillCreator 和 PenCreator 也有同样的两个菜单项，并会拒绝由其他程序制作的备份）；如果启动时找不到配置文件，启动窗口会用一个绿色按钮提供此功能。图层预设也包含在 zip 中；恢复时会用它们替换当前列表，不包含它们的较旧备份则不会改变列表。控制设备也包含在 zip 中；恢复时会在所有三个程序中用它们替换当前的控制设备。
* **帮助 → 导出诊断信息 (zip)…**（**Help → Export diagnostics (zip)…**）会询问将一个用于附加到错误报告的 zip 文件保存在哪里：`info.txt`（版本、系统、设备、连接）、`device.json`（设备信息，地址中的密码已隐藏）、`settings.txt`（不含 Web 服务器密钥和最近文件）、`console.txt`（与机器之间通信的内容，不含状态轮询）以及 `project.cre`（打开的项目）。发送前请检查一遍：其中包含你的设计。
* **更新：**启动时程序会在 GitHub 上查找更新的版本（**设置 → 程序选项…** 可关闭此功能）；**帮助 → 检查更新…**（**Help → Check for updates…**）可手动检查，如有更新版本会打开安装包页面。
* 窗口有 **自己的扁平方形边框**：标题栏带有图标、标题和打开的文件，以及最小化 / 最大化 / 关闭按钮。拖动标题栏可移动窗口，双击可最大化，从边缘调整大小（在 macOS 上从右边缘和下边缘）。
* **设置**（**Settings**）菜单：**视图选项…**（**View options…**）（网格、工作区背景、工作区周围空间的颜色、线条粗细、图层名称、变量名称、保存后预览 G-code）、**程序选项…**（**Program options…**）（自动保存、更新检查、Web 服务器和 API）、**插件…**（**Plugins…**）、**语言**（**Language**）（默认 English）和 **配色方案**（**Colour scheme**）（深色、浅深色、中浅色、浅色，以及蓝色、绿色、紫色、棕色和橙色的深色和浅色方案各一个；默认为浅深色；在圣诞节、新年夜、万圣节、复活节和农历新年前后，会为该节日添加一个深色和一个浅色的节日方案）。参见 [菜单](Menus-ZH)。
* 界面语言（38 种）：English、Polski、Deutsch、Italiano、Suomi、简体中文、हिन्दी；日本語、한국어、繁體中文、Türkçe、Tiếng Việt、Bahasa Indonesia、Српски、Bosanski；以及另外 23 种欧洲语言：Български、Català、Hrvatski、Čeština、Dansk、Nederlands、Eesti、Français、Ελληνικά、Magyar、Íslenska、Gaeilge、Latviešu、Lietuvių、Malti、Norsk bokmål、Português、Română、Slovenčina、Slovenščina、Español、Svenska、Українська。欧洲语言的文本位于 `assets/lang/`（每种语言一个文件），是在机器辅助下准备的，因此欢迎母语人士提供更正。图标是 SVG 文件，字体（Noto Sans、Mono、SC、KR、Devanagari）从 `assets/` 中嵌入，因此在每个系统上的外观都完全相同。

## 状态栏和按钮颜色

底部的状态栏显示设备的状态（带有一个圆点：连接时为绿色）以及 Web 服务器的状态（关闭、开启并显示其端口，或错误）。保存按钮为 **绿色**，导入按钮为 **蓝色**，删除按钮为带垃圾桶图标的 **红色**。

## 打印
**文件 → 打印…**（**File → Print…**）（Ctrl+P）在纸张打印机上打印设计。页面是由 LightCreator 生成的单页 PDF：按比例打印整个工作区（虚线边框），或只打印对象并在页面允许的范围内尽量放大；仅打印可见图层上的对象，或打印全部对象；A4 或 Letter；0–40 mm 的页边距。当打印部分宽度大于高度时，页面会转为横向，底部的一行文字给出比例。图层以其颜色绘制，填充形状带有浅色色调，图像为灰色。当对象超出工作区时会出现警告（使用工作区选项时，它们会在工作区边缘被裁掉）。

**打印**（**Print**）将页面发送到系统打印机：在 macOS 和 Linux 上使用 `lp`（可从系统列出的打印机中选择），在 Windows 上使用默认 PDF 查看器的 *打印*（*Print*）动作（默认打印机）。**另存为 PDF…**（**Save as PDF…**）和 **打开 PDF**（**Open the PDF**）（在查看器中打印，选项更多）是替代方式。打印预览就是 PDF 本身。这里的功能尚未在真实的打印机上试验过。
