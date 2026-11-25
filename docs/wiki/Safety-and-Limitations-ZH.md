🌐 [English](Safety-and-Limitations) · [Polski](Bezpieczenstwo-i-ograniczenia) · **简体中文** · [한국어](Safety-and-Limitations-KO)

> ⚠️ 本页由英文版机器翻译而成，可能有不准确之处。如有疑问，请以[英文版](Safety-and-Limitations)为准。欢迎提出修改建议。

# 安全与限制

> 已确认 LightCreator 能正常工作的机器，以及如何添加你的机器：[TESTS_ON_DEVICES.md](https://github.com/michalsarna/LightCreator/blob/main/TESTS_ON_DEVICES.md)。

> ⚠️ **激光是危险的。**请以低功率测试 G-code，佩戴护目镜，切勿让运行中的机器无人看管，并核实机器的 `$30`（S 最大值）和 `$32`（激光模式）设置。

## 已经验证的内容
所有几何、布尔运算、偏移、栅格、G-code、导出和翻译逻辑都有单元测试覆盖，截图从真实界面渲染。GRBL 流式传输在 [模拟器](GRBL-Simulator-ZH) 中针对真实的 GRBL 1.1h 固件进行了端到端测试。

## 尚未验证的内容
**LightCreator 已在 Sculpfun S9（一台使用 GRBL 的二极管激光器）上测试并能正常工作。**以下内容仍未在实际机器上测试：Marlin 流式传输、Z 轴和旋转附件（卡盘和滚轮的模型是一种假设）、钩子和宏、禁入区、多个激光头（`T<n>` 和 S 缩放）、对焦光束、指示光对齐和空运行、简单流式传输模式和停滞报告、USB 绑定和重新连接、连接后回零 / 解锁、固件 ($$) 页面（写入、备份、恢复、恢复出厂设置；单轴回零选项通过 `$I` 检测）、Smoothieware 方言、圆弧（G2 / G3）、自定义开始 / 结束 G-code、每个图层的功率模式、四角零点、反转轴和旋转面板、软限位和作业检查、连接后的警告（$32 / $30）、Ruida / Trocen 路径、在纸张打印机上打印（`lp`、Windows 打印动作）、实时摄像头（多个摄像头、画面设置、镜头校正、通过参考点对齐）、维护计数器、从设备读取设置，以及 Illustrator 导入（仅限兼容 PDF 的文件）。请先在废料上测试。

## Web 服务器与 API
开启后（程序选项（Program options）），Web 服务器和 API 需要访问口令（默认为 `LightCreator`，请修改它），但 **没有加密**。默认情况下，它们只在本机上监听；开启 *可从网络中的其他计算机访问*（*Reachable from other computers on the network*）后，网络中任何知道口令的人都可以查看机器，并停止或移动它。只在你信任的网络中使用。

## 尚未实现
直接的 Ruida / Trocen 二进制协议、文字的字距调整和复杂文字塑形（天城文、阿拉伯文）、摄像头的自动镜头校准（校正值需手动输入）。

## 安全漏洞
请通过 [GitHub security advisories](https://github.com/michalsarna/LightCreator/security/advisories/new) 私下报告漏洞。只有最新版本受支持。
