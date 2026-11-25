🌐 [English](Architecture) · [Polski](Architektura) · **简体中文** · [한국어](Architecture-KO)

> ⚠️ 本页由英文版机器翻译而成，可能有不准确之处。如有疑问，请以[英文版](Architecture)为准。欢迎提出修改建议。

# 架构

该仓库是一个包含两个 crate 的 Cargo 工作区。

| Crate | 用途 |
|---|---|
| `crates/lc-core` | 与界面无关、带单元测试的库 |
| `crates/lightcreator` | [egui](https://github.com/emilk/egui) 桌面应用程序 |

## baseCreator（与 MillCreator 共用）
不需要激光的代码位于 [baseCreator](https://github.com/michalsarna/baseCreator) 中，并在 `crates/lc-core/Cargo.toml` 和 `crates/lightcreator/Cargo.toml` 中固定到某个标签（tag）。
* **base-core**：`geom`、`bezier`、`arcs`、`fillet`、`snap`（几何、轮廓、圆角）· `ops`（布尔运算、偏移）· `text`（轮廓）· `image`、`trace`、`camimg` · `scene`（形状、编组、ID）· `svg`、`pdf`（读取器）· `clipart`（库）· `units` · `grbl_settings`、`ser2net`。
* **base-ui**：`theme`、`fonts`、`icons`、`i18n` / `texts` · `menu`、`native_menu`（菜单项、窗口内菜单栏和 macOS 菜单栏、标准编辑命令）· `window`（边框、弹出窗口、按钮）、`widgets` · `dialogs`（关于、快速指南、更新检查）· `clipart_ui` · `editor`（画布、工具、选择、节点编辑、编辑命令、图片对话框、视图选项、工具面板、排列工具栏、属性面板的各个部分，以及位于 `EditorHost` trait 之后的控制栏和状态栏组件）· `start`（启动界面标题、语言和配色方案行、窗口图标）· `doc_tabs`、`work_panes`（打开的绘图的标签页、标签页窗格以及标签页被移出后所在的窗口）· `gcode_text_ui`（以可编辑文本形式显示的项目 G-code）、`gcode_view_ui`（G-code 文件的分析）· `layer_bar`、`control_ui`（控制标签页）、`job_ui`（从某行开始）、`tools_ui`（校准）· `markdown_ui`、`units_ui`、`camera`、`camera_stream`、`camera_feed`、`camera_rec`（照片、视频、延时摄影）、`ffmpeg`。

`lc-core` 重新导出了 base-core 的模块，因此 `lc_core::geom` 等仍然可以使用。

## lc-core
`doc`（文档的激光部分：图层、设备、`.cre` 项目中的激光部分；场景和项目容器来自 base-core）· `gcode`、`export`（G-code、PLT / DXF）· `raster`、`cutopts` · `svg`、`pdf`、`clipart`（将导入的内容放到图层上）· `materials`、`profiles`、`zones`、`check`、`hooks` · `overlay_data`（项目文件中的叠加图片）、`print` · `controller`。

## lightcreator
`app`（主状态，实现 `EditorHost` 和 `MenuHost`）· `canvas`（工作台标记、禁入区、实时位置和预览，作为编辑器的钩子）、`shape_ops`、`overlay`（绘图下方的共享图片需要从激光部分获得的内容：随设备保存的角点、USB 摄像头）· `menu`（动作列表）、`icons`（每个动作的图标）、`native_menu`、`panels`、`i18n`（文本）、`guide`、`help_ui`、`update_ui`、`prefs`、`window_ui` · `laser`（用于 GRBL 和 Marlin 的串口 / TCP 工作线程）· `device_ui`、`ser2net_ui`、`stream_ui`（设备、摄像头）· `clipart_ui`（随程序附带的图片）、`materials_ui`、`preview_ui`、`print_ui`、`check_ui` · `panes_glue`（工作区的标签页）、`control_glue`（控制标签页和工作区上方的工具栏）、`workspaces`（打开的绘图）· `screenshots`（无界面渲染）。

`build.rs` 嵌入剪贴画。界面图标由 baseCreator 嵌入（`assets/ui-icons`）；有一个测试会检查每个动作所需的图标是否存在。

## 约定
* 坐标以 **毫米** 为单位，原点位于工作区的 **左上角**。输出时，对于零点在左前方的机器，Y 会被翻转（**机器 → 设备设置…**（**Machine → Device settings…**））。
* 存储的几何数据始终以 mm 为单位；显示单位按设备设置。

## 测试和 CI
* `cargo test --workspace` 覆盖几何、布尔运算、偏移、栅格、G-code、导出和翻译逻辑。
* 截图从真实界面无界面渲染：`cargo test -p lightcreator --release render_screenshots -- --ignored`。
* GRBL 流式传输在 [GRBL 模拟器](GRBL-Simulator-ZH) 中进行端到端测试。
* GitHub 工作流：`tests.yml`、`release.yml`（标签 `v` + `Cargo.toml` 版本号会构建 Windows / macOS / Linux 安装包）、`codeql.yml`。

## 参与贡献
欢迎提交 issue 和 pull request。有帮助的贡献包括：错误报告、在真实机器上的测试、翻译、剪贴画和材料。参见 [支持](Support-ZH)。
