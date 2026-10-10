🌐 **English** · [Polski](Szybki-start)

# Quick Start

1. **Start the program** ([Installation](Installation)). The **start window** asks which device to work with. Create a profile first: name, units, work area, machine zero and controller settings. The editor opens only after a profile is selected.
   ![Start window](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/01-start.png)
2. **Check the device settings** under *Laser → Device settings…*: controller (GRBL / Marlin / Ruida / Trocen), laser type, work area, S-value max, serial port or TCP address. For GRBL use *Read from device* to fill them in. See [Devices and Controllers](Devices-and-Controllers).
3. **Draw or import** a design: tools on the left, *File → Import* for SVG, bitmaps or Illustrator / PDF. See [Design Tools](Design-Tools).
4. **Assign cut layers.** Click a colour in the strip at the bottom to put the selection on that layer; double-click a layer name to set mode, speed, power and passes. See [Cuts and Layers](Cuts-and-Layers).
   ![Layers](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/03-layers.png)
5. **Preview** the job: the preview window shows every operation in its own colour, optional travel moves, and a playback slider with the time estimate.
   ![Preview](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/12-preview.png)
6. **Frame** the design (Device tab) to check the position without firing the laser (frame power is a device setting).
7. **Start** the job (the button is enabled when the laser is connected and becomes **STOP** while it runs). Use pause / resume as needed. The **Console** tab shows everything sent and received.
8. **No laser?** Export G-code (*File → Export*) or use the [GRBL Simulator](GRBL-Simulator).

> ⚠️ Always run a first test at low power on scrap material. See [Safety and Limitations](Safety-and-Limitations).

The built-in **Quick guide (F1)** documents every option and is searchable (English and Polish).
