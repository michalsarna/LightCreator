🌐 [English](GRBL-Simulator) · [Polski](Symulator-GRBL) · **简体中文** · [한국어](GRBL-Simulator-KO)

> ⚠️ 本页由英文版机器翻译而成，可能有不准确之处。如有疑问，请以[英文版](GRBL-Simulator)为准。欢迎提出修改建议。

# GRBL 模拟器

[grbl-sim](https://github.com/grbl/grbl-sim) 将真实的 GRBL 1.1h 固件编译为一个使用 GRBL 串口协议通信的 PC 程序，因此可以在没有激光器的情况下测试流式传输、状态、进给保持和复位。

**不涵盖：**USB 串口连接、激光功率输出、限位开关和回零、ATmega328P 时序。在这里通过测试是一个好兆头，但不能代替在机器上的低功率测试。

## 构建
模拟器位于 [baseCreator](https://github.com/michalsarna/baseCreator) 中（`tools/grbl-sim/`，由 LightCreator、MillCreator 和 PenCreator 共用）。需要带有 `git`、`make` 和 C 编译器的 Linux 或 macOS：
```sh
tools/grbl-sim/build.sh
```
它会将 GRBL `v1.1h.20190825` 和 grbl-sim 获取到 `target/grbl-sim/` 中，并应用补丁（`grbl-sim-fixes.patch`、`grbl-fixes.patch`，在 macOS 上还有 `grbl-sim-macos.patch`），修复仅在 PC 上出现的问题：延迟的 `ok` 应答、每次轮询的系统调用、共享中断标志导致的步进电机停滞、孤立的模拟器进程、丢失的复位请求、会丢字节的接收环形缓冲区（导致行乱码或行被合并），以及在 macOS 上导致模拟器运行过慢的时钟。它们不会改变 GRBL 在真实控制器上的行为。`GRBL_REF` 和 `SIM_REF` 用于选择其他版本。

## 在应用程序中使用
```sh
python3 tools/grbl-sim/serve.py        # listens on 127.0.0.1:3333
```
在 **机器 → 设备设置…**（**Machine → Device settings…**）中选择控制器 **GRBL**，并在 **连接**（**Connection**）页面中选择连接类型 **网络（TCP，例如 ser2net）**（**Network (TCP, e.g. ser2net)**），主机为 `127.0.0.1`，TCP 端口为 `3333`，然后在 **设备**（**Device**）标签页中按 **连接**（**Connect**）。每次连接都会启动一个使用默认设置的新模拟器。若要运行得更快，请在控制台中发送：`$110=30000`、`$111=30000`、`$120=5000`、`$121=5000`、`$32=1`。`-v` 会保留步进 / 程序段的跟踪信息。

Linux 上的虚拟串口：
```sh
socat PTY,raw,link=/tmp/ttyGRBL,echo=0 "EXEC:'target/grbl-sim/grbl/grbl/sim/grbl_sim.exe -n -s /dev/null -b /dev/null',pty,raw,echo=0"
```

## 自动化测试
串口连接在 baseCreator 中进行测试（`base_ui::sim_test`：实时检查、使用字符计数的长作业、被拒绝的行、进给保持、中止、连接竞争、`M0` 暂停、设置备份/恢复、主轴和圆弧）。LightCreator 自己的测试（`crates/lightcreator/src/grbl_sim_tests.rs`）运行它所生成的内容：

| 测试 | 检查内容 |
|---|---|
| `grbl_sim_streams_a_generated_job` | 生成的作业被完全确认，最后处于空闲状态 |
| `grbl_sim_runs_a_job_with_arcs_and_ends_where_expected` | 圆弧，结束于零点 |
| `grbl_sim_starts_with_laser_mode_off_and_accepts_enabling_it` | 读取和写入 `$32` |
| `grbl_sim_focus_beam_switches_on_with_m3_and_off_with_m5` | 对焦光束 |
| `grbl_sim_pass_by_pass_and_object_by_object_both_run` | 两种遍数顺序都能运行 |

```sh
# in baseCreator: tools/grbl-sim/build.sh (into target/grbl-sim there), or point GRBL_SIM_EXE at a build
cargo test -p lightcreator grbl_sim -- --ignored --test-threads=1
```
`GRBL_SIM_EXE` 指向在其他位置构建的模拟器。*Tests* 工作流会在每个 pull request 上运行这些测试。对于 grblHAL，其 [Simulator](https://github.com/grblHAL/Simulator) 应该可以配合 `serve.py --sim` 使用（未试验）。模拟器会占用大约两个 CPU 核心。
