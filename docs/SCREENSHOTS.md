# Screenshots

All pictures are rendered headlessly from the real UI. Back to the [README](../README.md).

| | |
|---|---|
| ![Start window](screenshots/01-start.png) | ![Properties tab](screenshots/02-editor-properties.png) |
| Start window: pick the device to work with | Editor, Properties tab, toolpath preview on |
| ![Cuts / Layers tab](screenshots/03-layers.png) | ![Console tab](screenshots/04-console.png) |
| Cuts / Layers tab | Console tab: live view of the serial traffic |
| ![Device configuration](screenshots/05-device-config.png) | ![Light scheme, Polish](screenshots/06-light-polish.png) |
| Device configuration (controller, units, work area, port, jog) | Light colour scheme with the Polish interface |
| ![Node editing](screenshots/07-nodes.png) | ![Text tool](screenshots/08-text.png) |
| Bézier node editing with its floating toolbar | Text tool and text properties |
| ![Image engraving](screenshots/09-image.png) | ![Camera overlay](screenshots/10-camera-overlay.png) |
| Raster image engraving, dithered toolpath preview | Camera overlay aligned with four corners |
| ![Preview window](screenshots/12-preview.png) | |
| Preview window: every burn operation in its own colour, with travel moves | |
| ![Import image](screenshots/13-image-import.png) | ![Trace image](screenshots/14-trace.png) |
| Image import with rotate, flip, brightness, contrast and gamma | Trace an image into curves |
| ![Grid options](screenshots/15-grid-layers.png) | ![Automatic shapes](screenshots/16-shapes.png) |
| Main and secondary grid options, layers in burn order | Triangle, star and polygon tools with the sides popup |
| ![Camera view as a tab](screenshots/17-camera-tab.png) | ![Share over network](screenshots/18-ser2net.png) |
| Camera view docked as a side tab, rotated 90° | Sharing the laser's serial port with ser2net (Linux) |
| ![Layer options and locking](screenshots/19-layers-lock.png) | ![Chinese interface](screenshots/20-chinese.png) |
| Layer names in the strip, layer options window, locked object, light work area in a dark scheme | Chinese interface, fonts and icons from the assets folder |
| ![Hindi interface](screenshots/21-hindi.png) | ![Quick guide and round corners](screenshots/22-round-help.png) |
| Hindi interface | Built-in quick guide (F1) and the round corners window |
| ![Rounded shapes](screenshots/23-rounded-shapes.png) | ![Clipart gallery](screenshots/24-clipart.png) |
| Rounded rectangle and triangle | Clipart gallery: blue dots ship with the program, green dots are yours |
| ![Online clipart](screenshots/25-clipart-online.png) | ![Node editing of a gallery picture](screenshots/26-clipart-nodes.png) |
| Searching open icon collections and saving into a category | A gallery picture opened in the node tool |
| ![Device tab](screenshots/27-device-tab.png) | ![Rotate and slant mode](screenshots/28-rotate-mode.png) |
| Device tab: settings in full width, jog arrows and controls as large equal buttons, STOP across the panel | Click a handle: the red resize selection turns blue, with round corner handles (rotate) and diamond edge handles (slant) |
| ![Close dialog](screenshots/29-close-dialog.png) | |
| Closing with unsaved work: Save, Discard or Cancel | |
| ![About](screenshots/30-about.png) | ![Greek interface](screenshots/31-greek.png) |
| The About window, in the style of the preview and camera windows | Greek interface |
| ![Ukrainian interface](screenshots/32-ukrainian.png) | |
| Ukrainian interface (one of the 23 added European languages) | |

Regenerate them with
`cargo test -p lightcreator --release render_screenshots -- --ignored`.
| ![Add a device](screenshots/33-add-device.png) | ![Materials and layers](screenshots/34-material-layers.png) |
| Adding a device: one window with the basic settings and a first material | Layers tab: the material header above the layers of that material |
| ![Description tab](screenshots/35-description.png) | ![Program options](screenshots/36-options-web-server.png) |
| Description tab, saved in the project | Program options with the web server; its state is in the status bar |
