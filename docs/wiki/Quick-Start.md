🌐 **English** · [Polski](Szybki-start)

# Quick Start

Your first job, step by step: from the start window to a burned piece. Every label below is the one the program shows; the menus are described entry by entry on [Menus](Menus).

> ⚠️ Run the first test at low power on scrap material, wear eye protection and stay at the machine. See [Safety and Limitations](Safety-and-Limitations).

## 1. Start the program and choose a device

Install and start LightCreator ([Installation](Installation)). The **start window** has two parts.

![Start window](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/01-start.png)

* **Device** ("Choose the device you will work with."): the list of your devices with **Add device…**, **Edit…**, **Duplicate**, **Import…** and **I have no device**.
* **Start** ("Open a project or start an empty one."): **New**, **Open…**, **View G-code file…** and the files you opened last. Under it you choose the **Language** and the **Colour scheme**.

The first time the list is empty. Press **Add device…**. One window asks for the basics:

* **Name**, **Controller** (GRBL, Marlin, Smoothieware, Ruida, Trocen) and the **Connection type**: a serial **Port** (with **Refresh** to look for ports again) or a network **Host** and **TCP port**.
* **Read from device** (blue): with the machine plugged in, it reads the work area and the speed from the controller.
* **Units**, **Speed units**, **Work area** (X and Y), **Machine zero (0,0)** (the corner where the machine has its zero), **S-value max ($30)**, **Laser type** and, if you have one, a **Camera**.
* **Worth checking**: a list of what the window could not know (for example laser mode and `$30` not read yet).

**Add device** puts the device in the list and selects it. **More settings…** opens the whole device window with what you typed. **I have no device** makes a plain device called **Dummy**, for preparing a project without a machine.

Select the device and press **New**. (With no device at all, **New** and **Open…** make and use the Dummy.) The editor opens with an empty drawing in a tab. You can check or change the device later under **Machine → Device settings…** and choose another one with **Machine → Switch device…**. All the settings are on [Devices and Controllers](Devices-and-Controllers).

## 2. Draw a shape or import a design

* **Draw:** pick a tool on the left or in the **Insert** menu, for example **Rectangle (R)**, and drag it on the work area (Shift keeps it square). **Text (T)**: click and type. The size and the position can be typed in the **Properties** tab on the right.
* **Import:** **File → Import → Import SVG…** (Ctrl+I) adds the shapes of an SVG file; each colour of the file goes to a layer. The same submenu reads pictures, Illustrator / PDF, DXF, LightBurn and UGS files. A file can also be dragged onto the window.
* **Ready-made pictures:** the **Clipart** tab, see [Clipart Gallery](Clipart-Gallery).

Select with a click or a frame, move by dragging or with the arrow keys (1 mm, Shift: 10 mm). The **Arrange** menu and the narrow bar next to the work area align, centre, turn and group. See [Design Tools](Design-Tools).

![Editor](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/02-editor-properties.png)

## 3. Put the shapes on layers and set the layers

The colour of a shape is its **layer**, and the layer says how it is burned.

* The row of colours under the work area is the **layer bar**. Select shapes and click a colour: they move to that layer. With nothing selected the click chooses the layer new shapes go to.
* **Double-click** a colour (or a layer in the **Layers** tab) to open **Layer options**: **Name**, **Colour**, **Output (burn this layer)**, **Visible**, then **Mode** (**Line** follows the outlines, **Fill** scans the inside, **Fill + Line**, **Offset fill**), **Speed**, **Power (%)**, **Passes**, the **Interval** of the fill lines and more. A new layer starts at 100 % power and 1000 mm/min.
* The **Layers** tab lists the layers that are used (**Show all 64 layers** shows the others), in the order they are burned; move them up or down to change the order. Its heading shows the **material** whose settings the layers carry; choose another one there, or manage them in **Laser → Materials…**.

![Layers](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/03-layers.png)

All the settings are on [Cuts and Layers](Cuts-and-Layers).

## 4. Look at the job before burning

* **Toolpaths** (the bar over the work area; green while on) draws the paths on the drawing itself.
* **Preview…** on the same bar, or **Laser → Preview window…** (Ctrl+Shift+P), opens the **Preview** tab: every operation in its own colour (**Colour by** **Operation** or **Layer**), **Show travel moves**, **Play** with a slider and a speed (**Fit 15 s**, **Real time**, ×10, ×50, ×200), the list of **Operations** with a switch each, and the totals: time, cut length, passes and moves.

![Preview](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/12-preview.png)

* **Preview G-code** (next to **Save G-code** on the bar) makes the program and shows it as text in a tab, **Preview G-code: name.gcode**, without saving anything. The words are coloured and the text can be edited. **Save G-code** in the tab writes it to a file, **Make again** makes it from the project again (with edits it asks first: **Lose the edits?**), **Analyse** shows its toolpaths. While you have not edited it, the text is made again by itself when you save the project, change its notes (the **Description** tab, which go into the file as `; Note:` comments), change the device or switch postprocessors.

Tabs can be dragged into up to three panes (**View → Split view: side by side**) so that the drawing and the preview stand next to each other. See [Design Tools](Design-Tools#view-and-navigation).

## 5. Save the G-code, or send the job to the machine

**Without a machine (or for a card or another program):** **Save G-code** on the bar, **File → Export → Export G-code…** or **Laser → Export G-code…** (Ctrl+E). The job is checked first (objects outside the work area are an error; layers at 0 % power are a warning), then the file is written, through the postprocessors you switched on in [Plugins](Plugins). For Ruida and Trocen use **File → Export → Export PLT / DXF…** and open the file in the controller's own software. The [GRBL Simulator](GRBL-Simulator) runs a job without a laser.

**With a connected laser (GRBL, Marlin, Smoothieware):**

1. In the **Device** tab press **Connect** (green). After connecting, GRBL is read once: if laser mode (`$32`) is off, a window warns you and offers to switch it on.
2. In the **Control** tab set **Frame power** (0 % keeps the laser off while framing; 1–2 % shows a dim dot on a diode laser).
3. **Outline** (the laser follows the line that wraps all the shapes) or **Outline frame** (the box round them; also **Machine → Frame**) shows where the job will burn. Put the material there.
4. **START** checks the job and sends it. While it runs, **Pause** and **Resume** hold and continue it, and the wide red **STOP** ends it. **Machine → Start job** does the same as START; **Machine → Start from line…** continues a job from a line of the program.
5. The **Console** tab shows everything sent to and received from the controller.

The job options under the buttons (**One element at a time**, **Selection only**) and where the job starts (**Start from**) are explained in [Devices and Controllers](Devices-and-Controllers#device-tab-and-the-control-bar).

## 6. Camera (if the device has one)

With a camera address in the device settings (page **Camera**), the bar shows **Camera view**: the live picture, with **Take a photo**, **Record video** and a time-lapse. **Overlay** (on the bar) puts a photo or the camera picture under the design so that you can place the work on the material: with no overlay shown it opens the **Camera overlay** window (the same as **View → Overlay from…**), where you load a picture or start the camera and drag its four corners into place. The overlay is saved in the project file. See [Devices and Controllers](Devices-and-Controllers#camera).

## 7. Save the project

**File → Save** (Ctrl+S) writes a `.cre` project with the drawing, the layers, the material, the notes and the overlay picture. Auto-save (**Settings → Program options…**) keeps your work safe in between. See [Settings and Files](Settings-and-Files).

The built-in **Quick guide** (F1) describes the options inside the program and has a search.
