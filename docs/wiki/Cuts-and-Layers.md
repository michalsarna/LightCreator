🌐 **English** · [Polski](Ciecia-i-warstwy)

# Cuts and Layers

![Layers](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/19-layers-lock.png)

## Layers
* 30 colour layers. Each has a **mode** (Line, Fill, Fill + Line, Offset fill), speed, power, passes, interval, scan angle, overscan, bidirectional scanning, and output / visibility switches.
* New layers start at 100 % power and 1000 mm/min. The settings of the layers belong to the **project**: the project file carries them together with the device profile; the device profile itself does not keep layer speeds (older profiles still contain the old list, which is ignored). Speeds are shown per second or per minute (device option *Speed units*), in mm or inches.
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

## Preview window
Opens zoomed to the objects to burn; two icons in its title line fit the view to the work area or to all objects. The *Preview…* button on the control bar turns light blue while the window is open and closes it when pressed again. The whole job with each operation (layer, fill, line, offset, image, pass) in its own or the layer colour, per-operation switches, optional travel moves, machine zero at its real corner, time-based playback (15 s fit, real time, ×10 to ×200), scrub slider, and totals for time, length, passes and moves.
![Preview](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/12-preview.png)

## Kerf, perforation and tabs (line layers)
* **Kerf offset:** moves closed outlines out by the given distance (in, when negative) to make up for the width of the cut; holes move the other way. 0 = off. Only the G-code uses it; the HPGL / DXF export and the toolpath preview show the outlines as drawn.
* **Perforation:** the outline is cut for *Cut length*, skipped for *Skip length*, and so on. The skipped stretches are travel moves with the laser off.
* **Tabs / bridges:** short stretches of the outline are left uncut, or cut with a percentage of the layer's power (*Tab cut power*), so the piece stays in place. *Tab size* is the length along the outline. *Even spacing* puts a tab every *Spacing* millimetres (at least *Min tabs*), *Tabs per shape* a fixed number; *Limit max tabs* caps the number, and *Skip inner shapes* leaves shapes lying inside another one without tabs. Tabs are placed automatically, evenly along the outline; placing them by hand is not possible yet.
The options are switched off by default and then change nothing in the G-code. They are saved with the layer and in the project.

## Images and the Image mode
A layer that holds only bitmap images is always in the **Image** mode; the mode cannot be changed, but speed, power, passes, interval, scan angle, overscan, dithering and the rest can. Vector shapes put on such a layer are not burnt (a layer with both images and outlines keeps the mode you chose). The **scan angle** turns the engraving lines of an image by that angle.

Not implemented from other programs' cut settings: sub-layers (several settings on one colour) and placing tabs by hand.
