🌐 [English](Plugins) · [Polski](Wtyczki) · **简体中文** · [한국어](Plugins-KO)

> ⚠️ 本页由英文版机器翻译而成，可能有不准确之处。如有疑问，请以[英文版](Plugins)为准。欢迎提出修改建议。

# 插件

插件可以在不修改程序的情况下为 LightCreator 添加内容：**剪贴画、机器预设、翻译、其他文件格式的导入器、形状生成器以及 G-code 后处理器。**它们由 LightCreator、MillCreator 和 PenCreator 共用：插件位于共享数据文件夹（`baseCreator`）中的 `plugins` 文件夹里，每个程序使用其中适合自己的部分。

**设置 → 插件…**（**Settings → Plugins…**）会打开插件窗口：从 `.zip` 安装插件、开启和关闭插件、删除插件、打开插件文件夹，以及设置有设置项的插件：每个有可选内容的插件（**后处理器**（**postprocessor**））在右侧面板中都有自己的标签页，在那里为下一次 G-code 导出选择它。

* **导入器：****文件 → 导入 → 使用插件导入…**（**File → Import → Import with a plugin…**）、拖放，或对插件能读取其扩展名的文件使用打开（Open）。形状会被添加到绘图中（处于选中状态，一次撤销即可撤回）。
* **生成器：****插入 → 生成形状…**（**Insert → Generate shape…**）（也可用 **文件 → 导入 → 生成形状…**（**File → Import → Generate shape…**））显示生成器的表单（数字、开关、文本、列表），并添加它生成的形状。
* **后处理器：**在其插件的标签页中选择某个后处理器后，**导出 G-code…**（**Export G-code…**）（**保存 G-code**（**Save G-code**）按钮、**文件 → 导出 → 导出 G-code…**（**File → Export → Export G-code…**）、**激光 → 导出 G-code…**（**Laser → Export G-code…**））和 **预览 G-code**（**Preview G-code**）会让程序经过它处理；切换后处理器时，G-code 文本标签页会重新生成。后处理器只能修改 G-code 行；包含控制器命令（`$…`、`~`、`!`、`?`）或控制字符的行会被拒绝，并且不会保存任何内容。直接将作业发送到机器（**START**、**机器 → 开始作业**（**Machine → Start job**））不会经过后处理器。
* LightCreator 不使用插件的预设和数据文件（`mill-tools`）；它使用剪贴画、导入器、生成器和后处理器。

**分步使用 G-code tools（以及任何后处理器）。**1. **设置 → 插件…** → **官方插件**（**Official plugins**）→ 安装 **G-code tools**（或使用 **从 zip 安装插件…**（**Install plugin from zip…**））。2. 右侧面板中会在其他标签页之后出现一个名为 **G-code tools** 的标签页（面板折叠时，出现在窄条上）。3. 打开它并选择 **为行编号 (N10, N20, …)**（**Number the lines (N10, N20, …)**）或 **删除注释**（**Remove comments**）；**无**（**None**）会再次关闭它。4. 现在 **导出 G-code…**（LightCreator 中的 **激光**（**Laser**）菜单，其他程序中的铣削（Milling）/ 绘图（Plot）菜单，或 **保存 G-code** 按钮）会写入处理后的文件；状态栏会显示使用了哪个后处理器。用 **开始**（**Start**）启动的作业会原样发送，G-code 查看器会按写入时的样子显示文件。

**安全。**插件脚本（Rhai）在沙箱中运行，无法访问文件、网络、系统或机器，并有时间和内存限制。即便如此，插件仍然是由他人编写、决定你的机器要执行什么的代码：只安装你信任的插件，并在机器上运行新的后处理器之前查看其生成的 G-code。

## 官方插件

**设置 → 插件… → 官方插件**（**Settings → Plugins… → Official plugins**）显示官方插件列表，该列表保存在 [baseCreator-plugins](https://github.com/michalsarna/baseCreator-plugins) 仓库中。按 **检查更新**（**Check for updates**）加载列表：它会列出 *所有* 插件，包括用于其他程序的插件（标记为 *适用于：…*（*For: …*））。**安装**（**Install**）会从官方站点下载插件（会校验其校验和）到所有程序共用的插件文件夹中；当列表中有更新的版本时会出现 **更新**（**Update**），鼠标悬停可查看更新日志。需要比你的程序更新版本的插件会被显示，但不提供安装。程序启动时检查更新默认 **开启**（取消勾选 *启动时检查插件更新*（*Check for plugin updates at start*）可将其关闭）：当有插件过期时，会弹出一个窗口列出它们，带有 **全部更新**（**Update all**）、**插件**（**Plugins**）和 **稍后**（**Later**）。未经你点击，不会安装任何内容。

编写你自己的插件并将其提交到列表：参见插件仓库中的 [作者指南](https://github.com/michalsarna/baseCreator-plugins/blob/main/docs/GUIDE.md)，其中附有模板；问题和想法请提交到该仓库的 issues。

如何编写插件（清单、表单、脚本函数、示例）：[baseCreator 中的 PLUGINS.md](https://github.com/michalsarna/baseCreator/blob/main/docs/PLUGINS.md)。

可以同时开启多个后处理器：在各插件的标签页中勾选它们；它们会 **按勾选的顺序** 运行（一个的输出行交给下一个），状态栏会以 `+` 连接显示它们的名称。**无**（**None**）会将它们全部关闭。
