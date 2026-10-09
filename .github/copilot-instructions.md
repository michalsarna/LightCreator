# Instructions for GitHub Copilot (code review & autofix)

LightCreator is a Rust workspace: `lc-core` (geometry, layers, SVG I/O, G-code generation) and
`lightcreator` (egui desktop app + GRBL serial worker). It drives real lasers, so review with safety in mind.

Focus on:
- **Machine safety** – G-code must never leave the laser on after a job (`M5`), must clamp to the work area,
  and must not send power above the configured `S` max. Flag any change that could fire the laser unexpectedly.
- **Serial / G-code input** – validate anything sent to the controller; avoid unbounded buffers and panics
  in the serial worker thread.
- **File parsing** – SVG and `.lcr` project files are untrusted input: no panics, no unbounded allocation.
- **Rust quality** – avoid `unwrap()` on user-controlled data, keep `lc-core` free of UI dependencies,
  and keep `cargo test --workspace` and `cargo clippy` clean.
