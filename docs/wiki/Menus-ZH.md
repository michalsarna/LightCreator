🌐 [English](Menus) · [Polski](Menu) · **简体中文** · [한국어](Menus-KO)

> ⚠️ 本页由英文版机器翻译而成，可能有不准确之处。如有疑问，请以[英文版](Menus)为准。欢迎提出修改建议。

# 菜单

每个菜单的每一项，按程序中显示的顺序排列，附有其快捷键以及对其作用的简短说明。链接指向详细说明该功能的页面。LightCreator、MillCreator 和 PenCreator 使用相同的菜单布局：

**文件 · 编辑 · 插入 · 排列 · 视图 · 机器 · 激光 · 设置 · 帮助**（**File · Edit · Insert · Arrange · View · Machine · Laser · Settings · Help**）

在 macOS 上请将 **Ctrl** 理解为 **Cmd**。所有快捷键都汇总在 [键盘快捷键](Keyboard-Shortcuts-ZH) 页面中。

**平台差异**

* **Windows 和 Linux：**菜单栏绘制在窗口内部。**退出**（**Quit**）是 **文件**（**File**）菜单的最后一项，**关于**（**About**）是 **帮助**（**Help**）菜单的最后一项。
* **macOS：**菜单位于系统菜单栏中。**文件** 左侧是 **LightCreator** 菜单，包含 **关于 LightCreator**（**About LightCreator**）、服务、隐藏、隐藏其他、全部显示和 **退出**（Cmd+Q）；**文件** 菜单中没有退出，**帮助** 菜单中没有关于。**帮助** 之后是 **窗口**（**Window**）菜单，包含最小化和最大化。系统菜单只在 **新建**（**New**）、**打开…**（**Open…**）、**保存**（**Save**）、**另存为…**（**Save as…**）和 **导入 SVG…**（**Import SVG…**）旁边显示快捷键，但下表中的所有快捷键在 macOS 上同样有效（使用 Cmd）。
* **仅限 Linux：****机器 → 通过网络共享 (ser2net)…**（**Machine → Share over network (ser2net)…**）。
* **在启动窗口中**只有以下菜单项有效：**新建**、**打开…**、**最近打开**（**Open recent**）、**查看 G-code 文件…**（**View G-code file…**）、**退出**、**关于**、**语言**（**Language**）、**配色方案**（**Colour scheme**）、**从备份恢复…**（**Restore from backup…**）和 **检查更新…**（**Check for updates…**）。其余菜单项要等到打开项目后才能使用。

当前无法使用的菜单项会显示为灰色（例如没有可撤销内容时的 **撤销**（**Undo**），未连接时的 **框选**（**Frame**））。开关类菜单项（**网格**（**Grid**）、**刀具路径预览**（**Toolpath preview**）…）在开启时会显示一个勾。

## 文件（File）

| 菜单项 | 快捷键 | 作用 |
|---|---|---|
| **新建**（**New**） | Ctrl+N | 在新标签页中新建一个空白绘图（最多 10 个标签页）。在启动窗口中，它会使用所选设备（如果没有设备则使用 Dummy）开始一个项目。参见 [设置与文件](Settings-and-Files-ZH)。 |
| **打开…**（**Open…**） | Ctrl+O | 在单独的标签页中打开项目（`.cre`）（空标签页会被直接使用；已经打开的项目只会被切换显示）。在这里选择的 SVG 或 DXF 文件会被添加到当前打开的绘图中。 |
| **最近打开**（**Open recent**）▸ | | 你最近打开或保存的 10 个项目和文件；**清除列表**（**Clear list**）会清空它。已不存在的文件会从列表中移除。 |
| **保存**（**Save**） | Ctrl+S | 保存项目；没有文件名的项目会要求输入文件名。 |
| **另存为…**（**Save as…**） | Ctrl+Shift+S | 以新名称保存项目。 |
| **项目信息…**（**Project information…**） | | 文件信息（名称、文件夹、大小、创建和修改时间）、项目的创建时间、创建者、贡献者和编辑记录。每次手动保存都会记录在 **设置 → 程序选项 → 作者**（**Settings → Program options → Author**）中设置的人员（姓名、电子邮件、其他；在所有程序中相同）、时间以及程序及其版本，每分钟最多记录一次。 创建者保存全部三项，贡献者只保存姓名和电子邮件（电子邮件与创建者相同时只保存姓名）。 |
| **导入**（**Import**）▸ **导入图像…**（**Import image…**） | | 将位图（PNG、JPEG、BMP、GIF、WebP）放到页面上，并显示调整对话框。参见 [设计工具](Design-Tools-ZH)。 |
| **导入** ▸ **导入 SVG…**（**Import SVG…**） | Ctrl+I | 将 SVG 文件中的形状添加到绘图中；颜色映射到图层。其中的图片也会一并导入，放在当前图层上。 |
| **导入** ▸ **导入 Adobe Illustrator / PDF / EPS / VectorCraft…**（**Import Adobe Illustrator / PDF / EPS / VectorCraft…**） | | 导入 PDF 或兼容 PDF 的 `.ai` 文件第一页中的矢量路径、旧式 PostScript 类型的 `.ai` 文件以及 EPS 文件中的矢量路径；导入 VectorCraft 绘图（`.vectorcraft`，或由 VectorCraft 0.8 及更高版本写入的 EPS）及其中的图片。 |
| **导入** ▸ **导入 DXF…**（**Import DXF…**） | | 导入 ASCII DXF（当前格式或 R12（*legacy*））中的直线、圆弧、圆、多段线、样条曲线和块。 |
| **导入** ▸ **导入 LightBurn / MillMage 项目…**（**Import LightBurn / MillMage project…**） | | 导入 LightBurn 的 `.lbrn2` / `.lbrn` 文件或 MillMage 的 `.mage` 文件中的形状和图层设置。 |
| **导入** ▸ **导入 UGS 设计…**（**Import UGS design…**） | | 导入 UGS Designer 文件（`.ugsd`）中的形状。 |
| **导入** ▸ **使用插件导入…**（**Import with a plugin…**） | | 使用插件的导入器读取文件。参见 [插件](Plugins-ZH)。 |
| **导入** ▸ **生成形状…**（**Generate shape…**） | | 插件形状生成器的表单（**插入**（**Insert**）菜单中也有同样的菜单项）。 |
| **导出**（**Export**）▸ **导出 G-code…**（**Export G-code…**） | | 检查作业并将 G-code 写入文件，经过你启用的后处理器。与 **保存 G-code**（**Save G-code**）按钮以及 **激光 → 导出 G-code…**（**Laser → Export G-code…**）相同。 |
| **导出** ▸ **导出 SVG…**（**Export SVG…**） | | 将绘图写为 SVG，尺寸与工作区相同，包含曲线和图片。 |
| **导出** ▸ **导出 PLT / DXF…**（**Export PLT / DXF…**） | | 检查作业，并为 Ruida 和 Trocen 控制器的软件写入 HPGL（`.plt`）或 DXF。参见 [设备与控制器](Devices-and-Controllers-ZH)。 |
| **导出** ▸ **导出 DXF…**（**Export DXF…**） | | 将整个绘图导出为当前格式的 DXF（AutoCAD 2007）；曲线会以 0.01 mm 的精度展平。 |
| **导出** ▸ **导出 DXF R12（旧版）…**（**Export DXF R12 (legacy)…**） | | 同上，但导出为 DXF R12，供较旧的程序和切割机使用（也称为 *legacy*）。 |
| **导出** ▸ **导出 Adobe Illustrator…**（**Export Adobe Illustrator…**） | | 将轮廓导出为 PostScript 类型的 Illustrator 文件，包含曲线，项目的每个图层对应一个 Illustrator 图层。图片不包括在内。 |
| **导出** ▸ **导出 LightBurn 项目…**（**Export LightBurn project…**） | | 导出 LightBurn 项目（`.lbrn2`，如果你将文件命名为 `.lbrn` 则为该格式）：包含形状及其曲线和图片，项目的每个图层对应一个 LightBurn 图层。 |
| **导出** ▸ **导出 MillMage 项目…**（**Export MillMage project…**） | | 同上，导出为 MillMage 项目（`.mage`），材料尺寸与工作区相同。 |
| **导出** ▸ **将所选内容导出为 SVG…**（**Export selection as SVG…**）/ **将所选内容导出为 DXF…**（**Export selection as DXF…**） | | 写入所选形状（未选中任何内容时写入全部形状）。 |
| **查看 G-code 文件…**（**View G-code file…**） | | 在 G-code 分析中打开 G-code 文件（一个显示其刀具路径的标签页）。 |
| **打印…**（**Print…**） | Ctrl+P | 在纸张打印机上打印设计，或将其保存为 PDF。参见 [设置与文件](Settings-and-Files-ZH)。 |
| **退出**（**Quit**） | Ctrl+Q | 关闭程序；未保存的工作会逐个标签页询问。仅限 Windows 和 Linux（在 macOS 上位于 LightCreator 菜单中）。 |

## 编辑（Edit）

| 菜单项 | 快捷键 | 作用 |
|---|---|---|
| **撤销**（**Undo**） | Ctrl+Z | 撤回上一次更改。 |
| **重做**（**Redo**） | Ctrl+Y（也可用 Ctrl+Shift+Z） | 重新执行已撤销的操作。 |
| **复制**（**Copy**） | Ctrl+C | 复制所选内容。 |
| **粘贴**（**Paste**） | Ctrl+V | 粘贴已复制的内容。 |
| **再制**（**Duplicate**） | Ctrl+D | 立即生成所选内容的一个副本。 |
| **删除**（**Delete**） | Delete 或 Backspace | 删除所选内容（在节点编辑中：删除所选节点）。已锁定的对象不会被删除。 |
| **全选**（**Select all**） | Ctrl+A | 选中所有对象。 |

## 插入（Insert）

左侧工具面板中用于在页面上放置新对象的工具；当前使用的工具会打勾。选择某一项等同于点击其按钮或按下其快捷键。参见 [设计工具](Design-Tools-ZH)。

| 菜单项 | 快捷键 | 作用 |
|---|---|---|
| **矩形 (R)**（**Rectangle (R)**） | R | 拖动绘制矩形（按住 Shift：正方形）。 |
| **椭圆 (E)**（**Ellipse (E)**） | E | 拖动绘制椭圆（按住 Shift：圆形）。 |
| **三角形 (Y)**（**Triangle (Y)**） | Y | 拖动绘制三角形。 |
| **星形 (S)**（**Star (S)**） | S | 拖动绘制五角星。 |
| **多边形 (G)**（**Polygon (G)**） | G | 拖动绘制正多边形；一个小窗口用于设置边数。 |
| **心形 (K)**（**Heart (K)**） | K | 拖动绘制心形。 |
| **螺旋线 (I)**（**Spiral (I)**） | I | 拖动绘制螺旋线；一个小窗口用于设置圈数。 |
| **直线 (L)**（**Line (L)**） | L | 拖动绘制直线。 |
| **折线 / 钢笔 (P)**（**Polyline / pen (P)**） | P | 逐点点击；右键单击、**Enter** 或双击结束该线条。 |
| **文字 (T)**（**Text (T)**） | T | 点击以放置文字。 |
| **生成形状…**（**Generate shape…**） | | 插件形状生成器的表单。参见 [插件](Plugins-ZH)。 |
| **在线剪贴画…**（**Online clipart…**） | | 打开 **剪贴画**（**Clipart**）标签页，可搜索在线图标集。参见 [剪贴画库](Clipart-Gallery-ZH)。 |

## 排列（Arrange）

这些命令作用于所选内容。其中大部分也出现在工作区的右键菜单以及工作区旁边的排列工具栏中。参见 [设计工具](Design-Tools-ZH)。

| 菜单项 | 快捷键 | 作用 |
|---|---|---|
| **左对齐**（**Align left**）、**水平居中对齐 (H)**（**Align centre (H)**）、**右对齐**（**Align right**）、**顶端对齐**（**Align top**）、**垂直居中对齐 (V)**（**Align centre (V)**）、**底端对齐**（**Align bottom**） | | 将所选对象按某一边缘或中线对齐。 |
| **在工作台上居中**（**Centre on bed**） | | 将所选内容移到工作区中央。 |
| **相互居中**（**Centre on each other**） | | 将所选对象的中心放到同一点上（一个编组算作一个对象）；需要两个或更多对象。 |
| **水平翻转**（**Flip horizontal**）/ **垂直翻转**（**Flip vertical**） | | 镜像所选内容。 |
| **顺时针旋转 90°**（**Rotate 90° CW**）/ **逆时针旋转 90°**（**Rotate 90° CCW**） | | 将所选内容旋转四分之一圈。 |
| **编组**（**Group**） | Ctrl+G | 将所选内容合并为一个编组。 |
| **取消编组**（**Ungroup**） | Ctrl+Shift+G | 拆开编组。 |
| **锁定**（**Lock**） | Ctrl+L | 已锁定的对象不能移动、编辑或删除，但可以选中和复制。 |
| **解锁**（**Unlock**） | Ctrl+Shift+L | 解锁所选内容。 |
| **移到最前**（**Bring to front**）/ **移到最后**（**Send to back**） | | 更改对象的顺序。 |
| **编辑节点**（**Edit nodes**） | N | 切换到节点工具。参见 [设计工具](Design-Tools-ZH)。 |
| **转换为路径**（**Convert to path**） | | 将所选形状转换为由直线段组成的普通路径（其旋转和尺寸成为轮廓的一部分）。 |
| **合并**（**Union**）、**相交**（**Intersection**）、**减去**（**Subtract**）、**异或**（**Exclusive or**） | | 对所选形状进行布尔运算。 |
| **偏移形状…**（**Offset shape…**） | | 扩大或缩小轮廓。 |
| **圆角…**（**Round corners…**） | | 以指定半径将尖角倒圆。 |
| **转换为曲线**（**Convert to curves**） | | 将所选形状（包括文字，不包括图片）转换为可编辑的贝塞尔曲线。 |
| **自动连接路径…**（**Auto-join paths…**） | | 连接端点距离小于容差的开放路径。 |
| **按容差闭合路径…**（**Close path with tolerance…**） | | 闭合两个端点距离在该范围内的路径。 |
| **狗骨 / T 骨角…**（**Dogbone / T-bone corners…**） | | 在每个尖角处开一个缺口（用于铣削）。 |
| **网格阵列…**（**Grid array…**） | | 按行和列复制所选内容。 |
| **环形阵列…**（**Circular array…**） | | 围绕所选内容的中心进行复制。 |
| **沿路径复制…**（**Copy along path…**） | | 沿路径进行复制（最后选择路径）。 |
| **路径文字…**（**Text on path…**） | | 将文字的轮廓沿路径弯曲（最后选择路径）。 |
| **按形状修剪…**（**Trim by shape…**） | | 保留线条位于最后选中的闭合形状内部（或外部）的部分。 |
| **删除重复项**（**Remove duplicates**） | | 删除同一图层上轮廓相同的形状。 |
| **拆分**（**Break apart**） | | 每个轮廓成为一个单独的形状。 |
| **简化…**（**Simplify…**） | | 删除距离线条在容差范围内的点。 |
| **序列文字…**（**Serial text…**） | | 生成编号递增的文字副本，或为 CSV 文件的每一行生成一个副本。 |
| **条形码 / 二维码…**（**Barcode / QR code…**） | | 以轮廓形式生成二维码或 Code 39 条形码。 |
| **排样形状…**（**Nest shapes…**） | | 将所选形状紧凑地排列在一个区域内。 |
| **测量…**（**Measure…**） | | 所选内容的尺寸、位置、面积和长度，以及两个形状之间的距离。 |
| **调整图像…**（**Adjust image…**） | | 所选图片的亮度、对比度、伽马值等。 |
| **描摹图像…**（**Trace image…**） | | 将所选图片转换为轮廓。 |
| **编辑图像…**（**Edit image…**） | | 所选图片的照片编辑器：画笔、橡皮擦、仿制图章、填充，以灰度进行。 |
| **转换为图像…**（**Convert to image…**） | | 将所选形状（不含图片）转换为一张图片，可设置其分辨率和尺寸。 |
| **风格描摹…**（**Style trace…**） | | 以 21 种风格之一将所选图片重新绘制为线条。参见 [风格描摹](Style-Trace-ZH)。 |
| **几何多边形…**（**Geometric polygons…**） | | 将所选图片转换为几何（低多边形，low poly）多边形，多边形之间以及轮廓周围留有间隙：即打开该风格的风格描摹窗口。参见 [风格描摹](Style-Trace-ZH)。 |
| **添加为禁入区**（**Add as no-go zone**） | | 每个未锁定的所选形状都会成为设备的一个禁入区（圆形的形状成为圆形，否则为包围它的矩形），并从绘图中移除。右键菜单中也有此项。 |

## 视图（View）

| 菜单项 | 快捷键 | 作用 |
|---|---|---|
| **网格**（**Grid**） | | 显示或隐藏网格。 |
| **吸附到网格**（**Snap to grid**） | | 移动和绘制的点会跳到网格上。 |
| **吸附到对象**（**Snap to objects**） | | 移动的所选内容会与附近对象的边缘和中线对齐。 |
| **放大**（**Zoom in**）/ **缩小**（**Zoom out**） | Ctrl++ / Ctrl+- | 以视图中心为基准缩放。 |
| **工作台适应窗口**（**Fit bed to window**） | Ctrl+Shift+0 | 使工作区适应窗口，并保持适应状态，直到你手动缩放。 |
| **显示全部对象**（**Fit all objects**） | Ctrl+Shift+9 | 缩放到显示全部对象。 |
| **工作区自动适应窗口**（**Auto-fit work area to window**） | | 开关：在窗口大小改变时保持工作区适应窗口。 |
| **拆分视图：并排**（**Split view: side by side**） | | 将工作区分成左右相邻的窗格（最多三个）。当只有一个窗格且其中有两个或更多标签页时，会创建第二个窗格。参见 [设计工具](Design-Tools-ZH)。 |
| **拆分视图：上下排列**（**Split view: one above the other**） | | 同上，但窗格上下堆叠。 |
| **单一视图**（**Single view**） | | 将所有标签页放回一个窗格中。 |
| **刀具路径预览**（**Toolpath preview**） | | 在绘图上叠加绘制刀具路径；与 **刀具路径**（**Toolpaths**）按钮相同。 |
| **摄像头视图**（**Camera view**） | | 显示设备摄像头的实时画面；需要在设备设置中填写摄像头地址。参见 [设备与控制器](Devices-and-Controllers-ZH)。 |
| **显示摄像头叠加层**（**Show camera overlay**） | | 显示或隐藏设计下方的叠加图片；没有叠加层时为灰色。 |
| **叠加来源…**（**Overlay from…**） | | 打开 **摄像头叠加**（**Camera overlay**）窗口：加载图片或启动摄像头，然后进行对齐。 |
| **显示禁入区**（**Show no-go zones**） | | 以红色绘制设备的禁入区；设备没有禁入区时为灰色。 |
| **在预览中显示尺寸**（**Dimensions in the preview**） | | 在预览上显示作业的尺寸（与预览中的 **尺寸**（**Dimensions**）开关相同），并打开预览。 |
| **在预览中显示圆柱（旋转）**（**Cylinder in the preview (rotary)**） | | 使用旋转附件时：预览以 3D 方式显示工件上的作业，关闭时则平面显示。与预览中的 **圆柱 (3D)**（**Cylinder (3D)**）/ **平面**（**Flat**）相同。 |
| **预览**（**Preview**） | | 显示或隐藏预览，与 **预览…**（**Preview…**）按钮相同。 |
| **预览 G-code**（**Preview G-code**） | | 在标签页中显示作业的 G-code，与 **预览 G-code** 按钮相同。 |

## 机器（Machine）

| 菜单项 | 快捷键 | 作用 |
|---|---|---|
| **切换设备…**（**Switch device…**） | | 返回启动窗口以选择其他设备。 |
| **设备设置…**（**Device settings…**） | | 包含所有页面的设备窗口。参见 [设备与控制器](Devices-and-Controllers-ZH)。 |
| **固件设置 ($$)…**（**Firmware settings ($$)…**） | | 设备窗口的固件页面（GRBL 和 Marlin）。 |
| **校准…**（**Calibration…**） | | 根据测量的移动距离计算某一轴的每毫米步数，并根据两条测量的对角线计算各轴的垂直度；可以将新值写入控制器。需要连接。参见 [设备与控制器](Devices-and-Controllers-ZH)。 |
| **禁入区…**（**No-go zones…**） | | 打开设备窗口的 **禁入区**（**No-go zones**）页面。 |
| **控制设备 ▸**（**Control devices ▸**） | | 控制设备设置…（Control device settings…）打开键盘和手柄及其按键的窗口；本机的控制设备…（Control device of this machine…）为当前打开的机器选择控制设备。参见 [控制设备](Control-Devices-ZH)。 |
| **诊断与统计…**（**Diagnostics and statistics…**） | | 控制器的输入、机器的统计数据、维护计数器，以及可保存的 zip 文件。 |
| **通过网络共享 (ser2net)…**（**Share over network (ser2net)…**） | | 仅限 Linux：通过 ser2net 以 TCP 方式共享本地串口。参见 [设备与控制器](Devices-and-Controllers-ZH)。 |
| **框选**（**Frame**） | | 激光以框选功率沿包围所选形状（未选中任何内容时为全部形状）的矩形框移动；与 **控制**（**Control**）标签页中的 **轮廓框**（**Outline frame**）相同。需要连接且没有正在运行的作业。 |
| **开始作业**（**Start job**） | | 与 **START** 相同：检查作业并将其发送到激光器（对于 Ruida 和 Trocen，则改为导出 PLT / DXF 文件）。 |
| **从某行开始…**（**Start from line…**） | | 从你选择的程序行开始作业；机器会先被置于程序在该行时所处的状态。需要连接。 |
| **检查程序**（**Check the program**） | | GRBL 以检查模式（`$C`）运行程序：不会有任何移动，会报告它不接受的行。需要已连接的机器。 |

## 激光（Laser）

| 菜单项 | 快捷键 | 作用 |
|---|---|---|
| **材料…**（**Materials…**） | | 设备的材料窗口：材料、材料的图层设置、导出和导入。参见 [切割与图层](Cuts-and-Layers-ZH)。 |
| **图层预设…**（**Layer presets…**） | | 图层预设窗口：以某个名称保存当前图层的设置，或将预设应用到当前图层。 |
| **预览 G-code**（**Preview G-code**） | | 在标签页中以文本形式显示作业的 G-code，不保存（与 **预览 G-code** 按钮相同）。 |
| **预览窗口…**（**Preview window…**） | Ctrl+Shift+P | 以工作区标签页的形式打开作业预览。参见 [切割与图层](Cuts-and-Layers-ZH)。 |
| **导出 G-code…**（**Export G-code…**） | Ctrl+E | 与 **文件 → 导出 → 导出 G-code…**（**File → Export → Export G-code…**）以及 **保存 G-code** 按钮相同。 |

## 设置（Settings）

| 菜单项 | 快捷键 | 作用 |
|---|---|---|
| **视图选项…**（**View options…**） | | 网格、工作区及其周围的颜色、线条粗细、图层名称、变量名称、保存后预览 G-code。参见 [设计工具](Design-Tools-ZH)。还有 **G-code 控制台**（**G-code console**）的字号和颜色。 |
| **程序选项…**（**Program options…**） | | 自动保存、启动时检查更新、单位、预览的 **渲染质量**（**Rendering quality**）（1 快速 … 5 清晰）、Web 服务器和 API。参见 [设置与文件](Settings-and-Files-ZH) 和 [Web 服务器与 API](Web-Server-and-API-ZH)。 |
| **插件…**（**Plugins…**） | | 安装、开关和删除插件；官方插件列表。参见 [插件](Plugins-ZH)。 |
| **语言**（**Language**）▸ | | 界面的 38 种语言之一。 |
| **配色方案**（**Colour scheme**）▸ | | 配色方案之一。 |

## 帮助（Help）

| 菜单项 | 快捷键 | 作用 |
|---|---|---|
| **快速指南**（**Quick guide**） | F1 | 程序内置的指南，带有搜索功能。 |
| **GitHub 上的项目页面**（**Project page on GitHub**） | | 在浏览器中打开项目页面。 |
| **下载备份…**（**Download backup…**） | | 包含你的设置和剪贴画的 zip 文件。参见 [设置与文件](Settings-and-Files-ZH)。 |
| **从备份恢复…**（**Restore from backup…**） | | 读回这样的 zip 文件。 |
| **导出诊断信息 (zip)…**（**Export diagnostics (zip)…**） | | 一个可附加到错误报告中的 zip 文件。 |
| **检查更新…**（**Check for updates…**） | | 立即查找更新的版本。 |
| **关于**（**About**） | | 程序和 baseCreator 的版本、链接以及许可证。仅限 Windows 和 Linux（在 macOS 上：**LightCreator → 关于 LightCreator**（**LightCreator → About LightCreator**））。 |
