🌐 **English** · [Polski](Bezpieczenstwo-i-ograniczenia)

# Safety and Limitations

> ⚠️ **Lasers are dangerous.** Test G-code at low power, wear eye protection, never leave a running machine unattended, and verify your machine's `$30` (S-max) and `$32` (laser mode) settings.

## What has been verified
All geometry, boolean, offset, raster, G-code, export and translation logic is covered by unit tests, and the screenshots are rendered from the real UI. GRBL streaming is exercised end to end against real GRBL 1.1h firmware in a [simulator](GRBL-Simulator).

## What has not
**LightCreator has been tested on a Sculpfun S9 (a diode laser on GRBL) and works there.** Still untested on actual machines: Marlin streaming, the Z axis and the rotary attachment (the model of chuck and rollers is an assumption), hooks and macros, no-go zones, several laser heads (`T<n>` and S scaling), the focus beam, the pointer alignment and dry run, the simple streaming mode and the stall report, USB binding and reconnecting, homing / unlocking after connecting, the Firmware ($$) page (writes, backup, restore, factory reset; the single-axis homing option is detected from `$I`), the Smoothieware dialect, arcs (G2 / G3), own start / end G-code, per-layer power mode, the four-corner zero, inverted axes and rotated panels, the soft limits and the job check, the warnings after connecting ($32 / $30), the Ruida / Trocen path, printing on a paper printer (`lp`, the Windows print verb), the live camera (several cameras, picture settings, lens correction, alignment by reference points), maintenance counters, reading settings from a device, and the Illustrator import (PDF-compatible files only). Test on scrap material first.

## The web server and API
When switched on (Program options) the web server and API need an access phrase (default `LightCreator`, change it) but have **no encryption**. By default they listen only on this computer; with *Reachable from other computers on the network* anyone on the network who knows the phrase can see the machine and stop or move it. Use it only on a network you trust.

## Not yet implemented
Direct Ruida / Trocen binary protocols, scan angles for images, kerning and complex-script shaping (Devanagari, Arabic) in text, automatic lens calibration for the camera (the correction is entered by hand).

## Security
Report vulnerabilities privately via [GitHub security advisories](https://github.com/michalsarna/LightCreator/security/advisories/new). Only the newest release is supported.
