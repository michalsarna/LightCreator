🌐 **English** · [Polski](Symulator-GRBL)

# GRBL Simulator

[grbl-sim](https://github.com/grbl/grbl-sim) compiles the real GRBL 1.1h firmware into a PC program speaking the GRBL serial protocol, so streaming, status, feed hold and reset can be tested without a laser.

**Not covered:** the USB-serial link, laser power output, limit switches and homing, ATmega328P timing. A pass here is a good sign, not a replacement for a low-power test on the machine.

## Build
Linux or macOS with `git`, `make` and a C compiler:
```sh
tools/grbl-sim/build.sh
```
It fetches GRBL `v1.1h.20190825` and grbl-sim into `target/grbl-sim/` and applies two patches (`grbl-sim-fixes.patch`, `grbl-fixes.patch`) that fix PC-only problems: delayed `ok` replies, per-poll syscalls, stalled steppers from a shared interrupt flag, orphaned simulators, and lost reset requests. They do not change GRBL's behaviour on a real controller. `GRBL_REF` and `SIM_REF` select other versions.

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
`crates/lightcreator/src/grbl_sim_tests.rs` drives the real serial worker against the simulator:

| Test | Checks |
|---|---|
| `grbl_sim_keeps_up_with_real_time` | simulator runs at real speed |
| `grbl_sim_streams_a_generated_job` | generated job fully acknowledged, ends idle at zero |
| `grbl_sim_long_job_keeps_character_counting_in_sync` | dense fill with the 120-byte window always full |
| `grbl_sim_rejected_line_does_not_desync_the_stream` | `error:20` does not break the rest of the job |
| `grbl_sim_feed_hold_and_resume` | `!` holds, `~` resumes |
| `grbl_sim_abort_then_new_job_runs` | soft reset mid-job, then a new job runs |
| `grbl_sim_job_sent_right_after_connect_is_not_lost` | job sent while GRBL restarts after connect completes |

```sh
tools/grbl-sim/build.sh
cargo test -p lightcreator grbl_sim -- --ignored --test-threads=1
```
`LC_GRBL_SIM_EXE` points at a simulator built elsewhere. The *Tests* workflow runs them on every pull request. For grblHAL, its [Simulator](https://github.com/grblHAL/Simulator) should work with `serve.py --sim` (untried). The simulator keeps about two CPU cores busy.
