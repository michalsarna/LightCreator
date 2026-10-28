# Testing with the GRBL simulator

The simulator (grbl-sim: the real GRBL 1.1h firmware compiled for the PC), the harness of the tests and the tests of the
serial link are in baseCreator: see `docs/grbl-sim.md` there.

LightCreator's own tests (`crates/lightcreator/src/grbl_sim_tests.rs`) run what `lc-core` generates against it: a job with
fill and line, arcs, both orders of the passes, laser mode (`$32`) and the focus beam (M3 / M5). They are
ignored in a normal `cargo test`; build the simulator with `tools/grbl-sim/build.sh` of baseCreator into `target/grbl-sim`
(or point `GRBL_SIM_EXE` at a build) and run

```sh
cargo test -p lightcreator grbl_sim -- --ignored --test-threads=1
```

The *Tests* workflow builds it with the `grbl-sim` action of baseCreator (at the tag the program is pinned to) and runs them.
