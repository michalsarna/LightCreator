🌐 **English** · [Polski](Ciecia-i-warstwy)

# Cuts and Layers

![Layers](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/19-layers-lock.png)

![Materials](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/34-material-layers.png)

## Layers
* 64 colour layers. Each has a **mode** (Line, Fill, Fill + Line, Offset fill), speed, power, passes, interval, scan angle, overscan, bidirectional scanning, and output / visibility switches. A layer can also have a **description** (up to 500 characters, in the layer settings): what the layer is for, for example "cut through, two passes". It is shown in the tip of the layer in the list and in the layer bar, it is saved with the project and it travels with the layer settings of a material.
* New layers start at 100 % power and 1000 mm/min. The settings of the layers belong to the **project**: the project file carries them together with the device profile; the device profile itself does not keep layer speeds (older profiles still contain the old list, which is ignored). Speeds are shown per second or per minute (device option *Speed units*), in mm or inches.
* **Air assist:** each layer has an *Air pump on* switch. The G-code turns the pump on (`M8`; on Marlin the fan) when a layer that wants it starts, off when a layer without it starts, and off at the end.
* The row of colours under the work area is the **layer bar**: a click on a colour puts the selected shapes on that layer (with nothing selected it chooses the layer new shapes go to), a double-click opens the **Layer options** window (so does a double-click on a layer in the **Layers** tab). With **Settings → View options… → Show layer names** a layer that has shapes shows its name beside its colour. In the list of layers each row shows the **mode** as an icon before the speed (the name is in the tip); a click on it opens the choice of another mode, except for a layer of pictures, which is always in the Image mode.
* The three switches at the end of each row, headed by icons, are output (burn), visible and air pump. *Image settings* (dithering, minimum power) show only for layers that hold bitmaps.
* The **Layers** tab lists the layers that are used (and the active one; **Show all 64 layers** shows the rest) in **burn order**; move them up or down to change it. The Properties tab shows the layer colour of the selection.

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

## Preview
The preview is a **tab of the work area**, next to the drawings: **Preview…** on the bar over the work area or **Laser → Preview window…** (Ctrl+Shift+P) opens it (the button is lit while it is open and closes it when pressed again), and the cross of the tab closes it. Like any tab it can be dragged to another pane, and its right-click menu has **Open in a separate window** (**Back to the main window** puts it back). It opens zoomed to the objects to burn; the two icons at the right of its title line fit the view to all objects or to the work area. It shows the whole job with each operation (layer, fill, line, offset, image, pass) in its own colour or in the colour of its layer (**Colour by**: **Operation** or **Layer**), a switch for each operation in the list **Operations**, **Show travel moves** (off by default), and a playback: **Play** / **Pause**, a slider and a speed (**Fit 15 s**: the whole job in 15 seconds, **Real time**, ×10 (the default), ×50, ×200). Under the picture are the totals: time (where the slider stands, out of the whole), cut length, passes and moves. **Toolpaths** on the bar draws the same paths over the drawing itself.
![Preview](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/12-preview.png)

## Materials
Every device has a list of **materials** (at least one: a new device starts with an empty *Default material* and a few starting points for its laser type: Plywood 3 mm, Cardboard, Leather 2 mm, Paper and, for CO2, Acrylic 3 mm. Their names are translated into the language of the interface; names you give yourself are kept as typed. The numbers are generic values for a typical diode (about 10 W) or CO2 (about 40 W) laser: always test on scrap). A material has a name and the settings of the layers it uses (mode, speed, power, passes, interval, scan angle, overscan, kerf, perforation, tabs, head, Z, rotary...). A material can have a **colour**: set it with the colour button in the row that opens when a material is added (**+**) or renamed (the pencil), with a cross to clear it. It is only shown, to tell the materials apart (it tints the icon in front of the list and is a small square before each name in the list), and it changes nothing in the output. The list is part of the device, so it travels with the device profile (also in exports and backups) and with every saved project, together with the layers.

The **Layers tab** edits the layers of the selected material, and its heading says which one it is. In the heading you can choose another material (the layers take its settings; layers it does not define go back to the defaults, and the names, colours and switches of the layers stay), add a new, empty one (under the name you type; the layers go back to the defaults and the ones the design uses join it), rename or delete it (the last one cannot be deleted). A layer that the design uses but the selected material lacks is added to it at once, with the default settings; a layer you change is changed in the material. A project that is opened brings its material into the device's list (or finds it there).

## Kerf, perforation and tabs (line layers)
* **Kerf offset:** moves closed outlines out by the given distance (in, when negative) to make up for the width of the cut; holes move the other way. 0 = off. Only the G-code uses it; the HPGL / DXF export and the toolpath preview show the outlines as drawn.
* **Perforation:** the outline is cut for *Cut length*, skipped for *Skip length*, and so on. The skipped stretches are travel moves with the laser off.
* **Tabs / bridges:** short stretches of the outline are left uncut, or cut with a percentage of the layer's power (*Tab cut power*), so the piece stays in place. *Tab size* is the length along the outline. *Even spacing* puts a tab every *Spacing* millimetres (at least *Min tabs*), *Tabs per shape* a fixed number; *Limit max tabs* caps the number, and *Skip inner shapes* leaves shapes lying inside another one without tabs. Tabs are placed automatically, evenly along the outline; placing them by hand is not possible yet.
The options are switched off by default and then change nothing in the G-code. They are saved with the layer and in the project.

## Images and the Image mode
A layer that holds only bitmap images is always in the **Image** mode; the mode cannot be changed, but speed, power, passes, interval, scan angle, overscan, dithering and the rest can. Vector shapes put on such a layer are not burnt (a layer with both images and outlines keeps the mode you chose). The **scan angle** turns the engraving lines of an image by that angle.

Not implemented from other programs' cut settings: sub-layers (several settings on one colour) and placing tabs by hand.

**Materials window.** **Laser → Materials…** opens a window that manages the materials of the device: the list on the left (a dot marks the one the design uses; a colour square shows its colour), with buttons to add a material (**+**), copy it, move it up or down and delete it, and on the right the name and the colour of the selected one (**Save** applies them) and its layers with all their settings, which you can change there, add (**Add layer**) or remove. **Export…** writes the selected material to a file (`.json`), **Export all…** all of them, and **Import…** reads such a file; a material with a name that is already in the list is kept under another name (*Name (2)*), or, with **Replace materials that have the same name**, replaces it. The file is checked strictly on import (size, number of materials, every number), and a file made by a newer version is refused.

![The materials window](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/41-materials-window.png)
