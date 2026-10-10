🌐 **English** · [Polski](Ciecia-i-warstwy)

# Cuts and Layers

![Layers](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/19-layers-lock.png)

## Layers
* 30 colour layers. Each has a **mode** (Line, Fill, Fill + Line, Offset fill), speed, power, passes, interval, scan angle, overscan, bidirectional scanning, and output / visibility switches.
* New layers start at 100 % power and 1000 mm/s.
* The colour strip at the bottom shows layer names. Double-click a name (there or in the **Layers** tab) to open the layer options window.
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
Starting settings for common materials and operations (diode and CO2). Apply them to the active layer or save your own presets. Values are generic: **always test on scrap**.
![Materials](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/11-materials.png)

## Preview window
The whole job with each operation (layer, fill, line, offset, image, pass) in its own or the layer colour, per-operation switches, optional travel moves, machine zero at its real corner, time-based playback (15 s fit, real time, ×10 to ×200), scrub slider, and totals for time, length, passes and moves.
![Preview](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/12-preview.png)
