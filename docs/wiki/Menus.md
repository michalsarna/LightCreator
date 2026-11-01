🌐 **English** · [Polski](Menu)

# Menus

Every entry of every menu, in the order the program shows them, with its key and a short note on what it does. The links lead to the page that explains it in detail. LightCreator, MillCreator and PenCreator share one layout of the menus:

**File · Edit · Insert · Arrange · View · Machine · Laser · Settings · Help**

Read **Ctrl** as **Cmd** on macOS. The keys are listed in one place on [Keyboard Shortcuts](Keyboard-Shortcuts).

**Platform differences**

* **Windows and Linux:** the menu bar is drawn inside the window. **Quit** is the last entry of **File** and **About** the last entry of **Help**.
* **macOS:** the menus are in the system menu bar. Left of **File** is the **LightCreator** menu with **About LightCreator**, Services, Hide, Hide Others, Show All and **Quit** (Cmd+Q); **File** has no Quit and **Help** has no About. After **Help** comes a **Window** menu with Minimize and Maximize. The system menu shows a key only beside **New**, **Open…**, **Save**, **Save as…** and **Import SVG…**; the other editing keys (Cmd+Z, Cmd+C, Cmd+G…) still work in the window. **Ctrl+P**, **Ctrl+E** and **Ctrl+Shift+P** have no key on macOS: use the menu entries.
* **Linux only:** **Machine → Share over network (ser2net)…**.
* **On the start window** only these entries do anything: **New**, **Open…**, **Open recent**, **View G-code file…**, **Quit**, **About**, **Language**, **Colour scheme**, **Restore from backup…** and **Check for updates…**. The rest wait for a project to be open.

An entry that cannot be used at the moment is greyed out (for example **Undo** with nothing to undo, **Frame** without a connection). Switches (**Grid**, **Toolpath preview**…) show a tick while they are on.

## File

| Entry | Key | What it does |
|---|---|---|
| **New** | Ctrl+N | A new, empty drawing in a new tab (up to 10 tabs). On the start window it starts a project with the selected device (or the Dummy when there is none). See [Settings and Files](Settings-and-Files). |
| **Open…** | Ctrl+O | Opens a project (`.cre`) in a tab of its own (an empty tab is used as it is; a project that is open already is only shown). An SVG or DXF file chosen here is added to the drawing that is open. |
| **Open recent** ▸ | | The last 10 projects and files you opened or saved; **Clear list** empties it. A file that no longer exists is taken off the list. |
| **Save** | Ctrl+S | Saves the project; a project without a file name asks for one. |
| **Save as…** | Ctrl+Shift+S | Saves the project under a new name. |
| **Import** ▸ **Import image…** | | Puts a bitmap (PNG, JPEG, BMP, GIF, WebP) on the page, with the adjust dialog. See [Design Tools](Design-Tools#importing). |
| **Import** ▸ **Import SVG…** | Ctrl+I | Adds the shapes of an SVG file to the drawing; colours map to layers. |
| **Import** ▸ **Import Adobe Illustrator / PDF…** | | The vector paths of the first page of a PDF or a PDF-compatible `.ai` file. |
| **Import** ▸ **Import DXF…** | | Lines, arcs, circles, polylines, splines and blocks of an ASCII DXF. |
| **Import** ▸ **Import LightBurn project…** | | Shapes and layer settings of a `.lbrn2` / `.lbrn` file. |
| **Import** ▸ **Import UGS design…** | | Shapes of a UGS Designer file (`.ugsd`). |
| **Import** ▸ **Import with a plugin…** | | Reads a file with an importer of a plugin. See [Plugins](Plugins). |
| **Import** ▸ **Generate shape…** | | The form of a shape generator of a plugin (the same entry is in **Insert**). |
| **Export** ▸ **Export G-code…** | | Checks the job and writes the G-code to a file, through the postprocessors you switched on. The same as the **Save G-code** button and **Laser → Export G-code…**. |
| **Export** ▸ **Export SVG…** | | Writes the drawing as SVG. |
| **Export** ▸ **Export PLT / DXF…** | | Checks the job and writes HPGL (`.plt`) or DXF for the software of Ruida and Trocen controllers. See [Devices and Controllers](Devices-and-Controllers#controllers). |
| **Export** ▸ **Export selection as SVG…** / **Export selection as DXF…** | | Writes the selected shapes (all of them when nothing is selected). |
| **View G-code file…** | | Opens a G-code file in the analysis of G-code (a tab with its toolpaths). |
| **Print…** | Ctrl+P | Prints the design on a paper printer, or saves it as a PDF. See [Settings and Files](Settings-and-Files#printing). |
| **Quit** | Ctrl+Q | Closes the program; unsaved work is asked about tab by tab. Windows and Linux only (on macOS it is in the LightCreator menu). |

## Edit

| Entry | Key | What it does |
|---|---|---|
| **Undo** | Ctrl+Z | Takes back the last change. |
| **Redo** | Ctrl+Y (also Ctrl+Shift+Z) | Does again what was undone. |
| **Copy** | Ctrl+C | Copies the selection. |
| **Paste** | Ctrl+V | Pastes what was copied. |
| **Duplicate** | Ctrl+D | A copy of the selection, at once. |
| **Delete** | Delete or Backspace | Deletes the selection (in node editing: the selected nodes). Locked objects are not deleted. |
| **Select all** | Ctrl+A | Selects every object. |

## Insert

The tools of the left palette that put new objects on the page; the tool in use is ticked. Choosing one is the same as clicking its button or pressing its key. See [Design Tools](Design-Tools#drawing).

| Entry | Key | What it does |
|---|---|---|
| **Rectangle (R)** | R | Drag a rectangle (Shift: a square). |
| **Ellipse (E)** | E | Drag an ellipse (Shift: a circle). |
| **Triangle (Y)** | Y | Drag a triangle. |
| **Star (S)** | S | Drag a five-pointed star. |
| **Polygon (G)** | G | Drag a regular polygon; a small window sets the number of sides. |
| **Heart (K)** | K | Drag a heart. |
| **Spiral (I)** | I | Drag a spiral; a small window sets the number of turns. |
| **Line (L)** | L | Drag a straight line. |
| **Polyline / pen (P)** | P | Click point after point; a right-click, **Enter** or a double-click finishes the line. |
| **Text (T)** | T | Click to place a text. |
| **Generate shape…** | | The form of a shape generator of a plugin. See [Plugins](Plugins). |
| **Online clipart…** | | Opens the **Clipart** tab with the search of online icon collections. See [Clipart Gallery](Clipart-Gallery). |

## Arrange

The commands work on the selection. Most of them are also in the right-click menu of the work area and in the arrange bar beside it. See [Design Tools](Design-Tools).

| Entry | Key | What it does |
|---|---|---|
| **Align left**, **Align centre (H)**, **Align right**, **Align top**, **Align centre (V)**, **Align bottom** | | Lines the selected objects up on one edge or on their middle. |
| **Centre on bed** | | Moves the selection to the middle of the work area. |
| **Centre on each other** | | Puts the centres of the selected objects on one point (a group counts as one object); needs two or more. |
| **Flip horizontal** / **Flip vertical** | | Mirrors the selection. |
| **Rotate 90° CW** / **Rotate 90° CCW** | | Turns the selection a quarter turn. |
| **Group** | Ctrl+G | Joins the selection into a group. |
| **Ungroup** | Ctrl+Shift+G | Takes the group apart. |
| **Lock** | Ctrl+L | Locked objects cannot be moved, edited or deleted, but can be selected and copied. |
| **Unlock** | Ctrl+Shift+L | Unlocks the selection. |
| **Bring to front** / **Send to back** | | Changes the order of the objects. |
| **Edit nodes** | N | Switches to the node tool. See [Design Tools](Design-Tools#node-editing-n). |
| **Convert to path** | | Turns the selected shapes into plain paths of straight segments (their turn and size become part of the outline). |
| **Union**, **Intersection**, **Subtract**, **Exclusive or** | | Boolean operations on the selected shapes. |
| **Offset shape…** | | Grows or shrinks the outlines. |
| **Round corners…** | | Rounds sharp corners with a radius. |
| **Convert to curves** | | Turns the selected shapes (text too, not pictures) into editable Bézier curves. |
| **Auto-join paths…** | | Joins open paths whose ends are nearer than a tolerance. |
| **Close path with tolerance…** | | Closes paths whose two ends are that near. |
| **Dogbone / T-bone corners…** | | A notch in every sharp corner (for milling). |
| **Grid array…** | | Copies of the selection in rows and columns. |
| **Circular array…** | | Copies round the middle of the selection. |
| **Copy along path…** | | Copies along a path (select the path last). |
| **Text on path…** | | Bends the outlines of a text along a path (select the path last). |
| **Trim by shape…** | | Keeps the parts of lines inside (or outside) a closed shape selected last. |
| **Remove duplicates** | | Deletes shapes with the same outline on the same layer. |
| **Break apart** | | One shape of every outline. |
| **Simplify…** | | Takes away points within a tolerance of the line. |
| **Serial text…** | | Copies of a text with a number counted up, or one copy per row of a CSV file. |
| **Barcode / QR code…** | | A QR code or a Code 39 barcode as outlines. |
| **Nest shapes…** | | Packs the selected shapes on an area. |
| **Measure…** | | Size, position, area and length of the selection, and distances between two shapes. |
| **Adjust image…** | | Brightness, contrast, gamma and the rest of a selected picture. |
| **Trace image…** | | Turns a selected picture into outlines. |

## View

| Entry | Key | What it does |
|---|---|---|
| **Grid** | | Shows or hides the grid. |
| **Snap to grid** | | Moves and drawn points jump to the grid. |
| **Snap to objects** | | A moved selection lines up with the edges and middles of nearby objects. |
| **Zoom in** / **Zoom out** | Ctrl++ / Ctrl+- | Zooms around the middle of the view. |
| **Fit bed to window** | Ctrl+Shift+0 | Fits the work area to the window and keeps it fitted until you zoom by hand. |
| **Fit all objects** | Ctrl+Shift+9 | Zooms to all objects. |
| **Auto-fit work area to window** | | Switch: keep the work area fitted while the window changes size. |
| **Split view: side by side** | | Divides the work area into panes next to each other (up to three). A second pane is made when there is only one and it holds two tabs or more. See [Design Tools](Design-Tools#view-and-navigation). |
| **Split view: one above the other** | | The same, with the panes stacked. |
| **Single view** | | Puts all the tabs back into one pane. |
| **Toolpath preview** | | Draws the toolpaths over the drawing; the same as the **Toolpaths** button. |
| **Camera view** | | Shows the live picture of the device's camera; needs a camera address in the device settings. See [Devices and Controllers](Devices-and-Controllers#camera). |
| **Show camera overlay** | | Shows or hides the overlay picture under the design; greyed out while there is no overlay. |
| **Overlay from…** | | Opens the **Camera overlay** window: load a picture or start a camera, then align it. |
| **Show no-go zones** | | Draws the no-go zones of the device in red; greyed out when the device has none. |

## Machine

| Entry | Key | What it does |
|---|---|---|
| **Switch device…** | | Goes back to the start window to choose another device. |
| **Device settings…** | | The device window with all its pages. See [Devices and Controllers](Devices-and-Controllers). |
| **Calibration…** | | Works out the steps per millimetre of an axis from a measured move, and the squareness of the axes from two measured diagonals; can write the new value to the controller. Needs a connection. See [Devices and Controllers](Devices-and-Controllers#calibration). |
| **Share over network (ser2net)…** | | Linux only: shares a local serial port over TCP with ser2net. See [Devices and Controllers](Devices-and-Controllers#tcp-and-ser2net). |
| **Frame** | | The laser traces the box round the selected shapes (all when nothing is selected) at the frame power; the same as **Outline frame** in the **Control** tab. Needs a connection and no running job. |
| **Start job** | | The same as **START**: checks the job and sends it to the laser (for Ruida and Trocen it exports a PLT / DXF file instead). |
| **Start from line…** | | Starts the job at a line of the program you choose; the machine is first brought to the state the program has at that line. Needs a connection. |

## Laser

| Entry | Key | What it does |
|---|---|---|
| **Materials…** | | The materials window of the device: materials, their layer settings, export and import. See [Cuts and Layers](Cuts-and-Layers#materials). |
| **Preview window…** | Ctrl+Shift+P | Opens the preview of the job as a tab of the work area. See [Cuts and Layers](Cuts-and-Layers#preview). |
| **Export G-code…** | Ctrl+E | The same as **File → Export → Export G-code…** and the **Save G-code** button. |

## Settings

| Entry | Key | What it does |
|---|---|---|
| **View options…** | | Grids, colours of the work area and around it, line thickness, layer names, variable names, preview of the G-code after saving. See [Design Tools](Design-Tools#view-and-navigation). |
| **Program options…** | | Auto-save, the update check at start, the web server and API. See [Settings and Files](Settings-and-Files) and [Web Server and API](Web-Server-and-API). |
| **Plugins…** | | Installs, switches and removes plugins; the official list. See [Plugins](Plugins). |
| **Language** ▸ | | One of the 30 languages of the interface. |
| **Colour scheme** ▸ | | One of the colour schemes. |

## Help

| Entry | Key | What it does |
|---|---|---|
| **Quick guide** | F1 | The guide inside the program, with a search. |
| **Project page on GitHub** | | Opens the page of the project in the browser. |
| **Download backup…** | | A zip with your settings and your clipart. See [Settings and Files](Settings-and-Files). |
| **Restore from backup…** | | Reads such a zip back. |
| **Export diagnostics (zip)…** | | A zip to attach to a bug report. |
| **Check for updates…** | | Looks for a newer release now. |
| **About** | | The version of the program and of baseCreator, the links and the licence. Windows and Linux only (on macOS: **LightCreator → About LightCreator**). |
