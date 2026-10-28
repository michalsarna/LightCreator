🌐 **English** · [Polski](Symulator-GRBL)

# GRBL Simulator

[grbl-sim](https://github.com/grbl/grbl-sim) compiles the real GRBL 1.1h firmware into a PC program speaking the GRBL serial protocol, so streaming, status, feed hold and reset can be tested without a laser.

**Not covered:** the USB-serial link, laser power output, limit switches and homing, ATmega328P timing. A pass here is a good sign, not a replacement for a low-power test on the machine.

## Build
The simulator lives in [baseCreator](https://github.com/michalsarna/baseCreator) (`tools/grbl-sim/`, shared by LightCreator, MillCreator and PenCreator). Linux or macOS with `git`, `make` and a C compiler:
```sh
tools/grbl-sim/build.sh
```
It fetches GRBL `v1.1h.20190825` and grbl-sim into `target/grbl-sim/` and applies patches (`grbl-sim-fixes.patch`, `grbl-fixes.patch`, and `grbl-sim-macos.patch` on macOS) that fix PC-only problems: delayed `ok` replies, per-poll syscalls, stalled steppers from a shared interrupt flag, orphaned simulators, lost reset requests, a receive ring that dropped bytes (garbled or joined lines), and a clock that made the simulator far too slow on macOS. They do not change GRBL's behaviour on a real controller. `GRBL_REF` and `SIM_REF` select other versions.

## Use from the app
```sh
python3 tools/grbl-sim/serve.py        # listens on 127.0.0.1:3333
```
In *Laser → Device settings* pick controller **GRBL**, connection **TCP**, host `127.0.0.1`, port `3333`, and connect. Each connection starts a fresh simulator with default settings. For faster runs send in the console: `$110=30000`, `$111=30000`, `$120=5000`, `$121=5000`, `$32=1`. `-v` keeps step / block traces.

A virtual serial port on Linux:
```sh
socat PTY,raw,link=/tmp/ttyGRBL,echo=0 "EXEC:'target/grbl-sim/grbl/grbl/sim/grbl_sim.exe -n -s /dev/null -b /dev/null',pty,raw,echo=0"
```

## Automated tests
The serial link is tested in baseCreator (`base_ui::sim_test`: real-time check, long job with character counting, rejected line, feed hold, abort, connect race, `M0` pause, settings backup/restore, spindle and arcs). LightCreator's own tests (`crates/lightcreator/src/grbl_sim_tests.rs`) run what it generates:

| Test | Checks |
|---|---|
| `grbl_sim_streams_a_generated_job` | generated job fully acknowledged, ends idle |
| `grbl_sim_runs_a_job_with_arcs_and_ends_where_expected` | arcs, ends at zero |
| `grbl_sim_starts_with_laser_mode_off_and_accepts_enabling_it` | `$32` read and written |
| `grbl_sim_focus_beam_switches_on_with_m3_and_off_with_m5` | focus beam |
| `grbl_sim_pass_by_pass_and_object_by_object_both_run` | both orders of the passes |

```sh
# in baseCreator: tools/grbl-sim/build.sh (into target/grbl-sim there), or point GRBL_SIM_EXE at a build
cargo test -p lightcreator grbl_sim -- --ignored --test-threads=1
```
`GRBL_SIM_EXE` points at a simulator built elsewhere. The *Tests* workflow runs them on every pull request. For grblHAL, its [Simulator](https://github.com/grblHAL/Simulator) should work with `serve.py --sim` (untried). The simulator keeps about two CPU cores busy.
