🌐 **English** · [Polski](Bezpieczenstwo-i-ograniczenia)

# Safety and Limitations

> ⚠️ **Lasers are dangerous.** Test G-code at low power, wear eye protection, never leave a running machine unattended, and verify your machine's `$30` (S-max) and `$32` (laser mode) settings.

## What has been verified
All geometry, boolean, offset, raster, G-code, export and translation logic is covered by unit tests, and the screenshots are rendered from the real UI. GRBL streaming is exercised end to end against real GRBL 1.1h firmware in a [simulator](GRBL-Simulator).

## What has not
**Nothing has been run against real hardware yet.** Still untested on actual machines: GRBL and Marlin streaming, the Ruida / Trocen path, the live camera, reading settings from a device, and the Illustrator import (PDF-compatible files only). Material values are generic starting points; test on scrap.

## Not yet implemented
Direct Ruida / Trocen binary protocols, scan angles for images, kerning and complex-script shaping (Devanagari, Arabic) in text, lens-distortion correction for the camera. RTSP cameras are not supported (use MJPEG).

## Security
Report vulnerabilities privately via [GitHub security advisories](https://github.com/michalsarna/LightCreator/security/advisories/new). Only the newest release is supported.
