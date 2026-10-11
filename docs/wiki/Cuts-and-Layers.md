🌐 **English** · [Polski](Ciecia-i-warstwy)

# Cuts and Layers

![Layers](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/19-layers-lock.png)

## Layers
* 30 colour layers. Each has a **mode** (Line, Fill, Fill + Line, Offset fill), speed, power, passes, interval, scan angle, overscan, bidirectional scanning, and output / visibility switches.
* New layers start at 100 % power and 1000 mm/min. The settings of the layers belong to the **project**: the project file carries them together with the device profile and the material it was made with. What you want to reuse is saved as a **material** in the library of the device; the device profile itself no longer keeps layer speeds (older profiles still contain the old list, which is ignored). Speeds are shown per second or per minute (device option *Speed units*), in mm or inches.
* **Air assist:** each layer has an *Air pump on* switch. The G-code turns the pump on (`M8`; on Marlin the fan) when a layer that wants it starts, off when a layer without it starts, and off at the end.
* The colour strip at the bottom shows layer names. Double-click a name (there or in the **Layers** tab) to open the layer options window.
* The three switches at the end of each row, headed by icons, are output (burn), visible and air pump. *Image settings* (dithering, minimum power) show only for layers that hold bitmaps.
* The **Layers** tab lists layers in **burn order**; move them up or down to change it. The Properties tab shows the layer colour of the selection.

## Modes
| Mode | Result |
|---|---|
| Line | follows outlines |
| Fill | scan lines at the layer interval |
| Fill + Line | fill then outline |
| Offset fill | concentric inward rings at the layer interval |

Inner shapes are cut first; ordering is nearest-neighbour.

## Raster images
Images engrave line by line at the layer interval. Choose **threshold**, **Floyd-Steinberg**, **Jarvis**, **Stucki**, **Atkinson**, **ordered** dithering or **grayscale power** (min / max power). Each image has a negative option.
![Image engraving](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/09-image.png)

## Material library
A **material** holds the settings of every layer it needs: mode, speed, power, passes, interval, scan angle, overscan, dithering, minimum power and air assist. Three levels work together:

* **Built-in library:** starting values for typical 10 W diode and 40 W CO2 lasers. It is the source for the other libraries. Values are generic: **always test on scrap**.
* **Global materials:** the *Material library…* window (Laser menu) is the only place that extends the built-in library. *Save the layers of the design as a material for every device* adds a material that every device can use.
* **Device library:** every device has its own library in the **Materials** tab of the side panel. It is filled once from the built-in and global materials for its laser type. In the tab you edit all the settings of every layer, add or remove layers, rename, delete, **apply a material to the layers** of the design and **save the layers of the design as a material**; this goes to the device library only.

The device library is stored in the device profile (so also in backups) and inside every saved project. When you open a project, its library is merged into the library of the active device: new materials are added, and where a material differs a window lets you choose, value by value, to keep yours or take the one from the file.

A material is **one entry for both laser types**: diode and CO2 keep their own settings under the same id. Names such as *Plywood* are translated, the thickness is a separate value, and every material has a one-line note and tags (*Cut*, *Engrave*...). The built-in values can be changed on your computer: **double-click** a material in the Material library window (or press *Edit material*), set every layer and Save; the copy overrides the built-in data and *Restore built-in values* brings the original back. A device's library receives the variant for its own laser type. Lists are grouped by category and each group can be folded up by its title. The Materials tab has **Delete all materials**: it empties the device's library and shows the main library instead, and a material you change there is kept as a copy for the device. Tags are suggested from those in use, and hovering over *Dithering* explains the method. A saved project carries the device profile, the device library and the material it was made with.

![Editing a material](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/36-material-edit.png)

Each layer of a material also keeps its **colour** and a **label** (its purpose, e.g. *Cut*); choosing a material (double-click in the Materials tab; a green mark shows the one in use) puts the settings and colours of its layers into the layers of the design, but not their names, and keeps the two in step: a change in the Layers tab is copied into the material at once and saved into the device's library every 60 seconds; a change of the material in its editor goes into the layers. The list shows only the names. Materials have a **category** (Wood, Metal, Plastic, Paper and leather, Stone and glass, Other, or one you add) and the list in the Materials tab can be filtered by it. Lists show each material's name and tags in rows with alternating backgrounds, and small arrows at the edge when the list scrolls. A layer's label is chosen from a list; the **Layer labels…** button of the Material library window adds your own.

**Export library… / Import library…** (in the Material library window and the Materials tab) write and read one `.lcb` file with your global materials and your devices with their libraries. A window lets you tick exactly what goes into the file or is taken from it; materials that differ open the window of differences. The tabs of the side panel can be dragged to change their order.

![Library file](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/35-library-export.png)
![Materials tab](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/33-materials-tab.png)
![Differences](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/34-material-merge.png)
![Materials](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/11-materials.png)

## Preview window
Opens zoomed to the objects to burn; two icons in its title line fit the view to the work area or to all objects. The *Preview…* button on the control bar turns light blue while the window is open and closes it when pressed again. The whole job with each operation (layer, fill, line, offset, image, pass) in its own or the layer colour, per-operation switches, optional travel moves, machine zero at its real corner, time-based playback (15 s fit, real time, ×10 to ×200), scrub slider, and totals for time, length, passes and moves.
![Preview](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/12-preview.png)

## Kerf, perforation and tabs (line layers)
* **Kerf offset:** moves closed outlines out by the given distance (in, when negative) to make up for the width of the cut; holes move the other way. 0 = off. Only the G-code uses it; the HPGL / DXF export and the toolpath preview show the outlines as drawn.
* **Perforation:** the outline is cut for *Cut length*, skipped for *Skip length*, and so on. The skipped stretches are travel moves with the laser off.
* **Tabs / bridges:** short stretches of the outline are left uncut, or cut with a percentage of the layer's power (*Tab cut power*), so the piece stays in place. *Tab size* is the length along the outline. *Even spacing* puts a tab every *Spacing* millimetres (at least *Min tabs*), *Tabs per shape* a fixed number; *Limit max tabs* caps the number, and *Skip inner shapes* leaves shapes lying inside another one without tabs. Tabs are placed automatically, evenly along the outline; placing them by hand is not possible yet.
The options are switched off by default and then change nothing in the G-code. They are saved with the layer, with the material and in the project.

## Images and the Image mode
A layer that holds only bitmap images is always in the **Image** mode; the mode cannot be changed, but speed, power, passes, interval, scan angle, overscan, dithering and the rest can. Vector shapes put on such a layer are not burnt (a layer with both images and outlines keeps the mode you chose). The **scan angle** turns the engraving lines of an image by that angle.

Not implemented from other programs' cut settings: sub-layers (several settings on one colour) and placing tabs by hand.
