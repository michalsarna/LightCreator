🌐 **English** · [Polski](Narzedzia-projektowe)

# Design Tools

![Node editing](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/07-nodes.png)

## Drawing
* Rectangle, ellipse, line, polyline and text tools; select, move and resize with handles.
* **Automatic shapes:** triangle (Y), five-pointed star (S), regular polygon (G, 3 to 360 sides chosen in a popup), heart (K) and spiral (I). Drag a box (Shift keeps it square); the result is an ordinary editable Bézier shape. A small window in the corner of the work area sets the number of sides or of turns (3 by default); it has no OK button and stays while the tool is active; the polygon has a slider from 5 to 36 sides and the spiral one from 1 to 24 turns next to the number box (3 to 360 and 1 to 30), and a change also reshapes the shape drawn last.
  ![Shapes](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/16-shapes.png)
* **Tool bar groups:** selecting / node editing, then insert tools, then the shape operations (union, intersection, subtract, exclusive or), then navigation, separated by lines. A double arrow at the edge of a bar means more icons are out of sight: scroll to reach them.

## Node editing (N)
Drag nodes and handles, switch corner / smooth, insert (double-click a segment) and delete nodes, turn segments into lines or curves, open and close paths. Open it from the second tool, *Arrange → Edit nodes*, the right-click menu, or by double-clicking a shape. Clipart is editable the same way. Text is not converted automatically: use *Convert to curves* first.

## Text (T)
Live text from system fonts (family search, bold, italic, size, letter and line spacing, alignment), convertible to curves. Devanagari and Arabic are not shaped.
![Text](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/08-text.png)

## Shape operations
* **Boolean:** union, intersection, subtract, exclusive or.
* **Offset:** grow or shrink outlines, keeping or replacing the original.
* **Round corners:** *Arrange → Round corners…* rounds sharp straight corners with a given radius; with two straight lines selected it joins them with an arc. In the node tool the bar rounds only the selected corners.
  ![Round corners](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/23-rounded-shapes.png)
* Numeric position and size, rotate, flip, align, centre on bed, **centre on each other** (the centres of the selected objects, groups counting as one, go to one point), grid array, copy / paste / duplicate.
* **Arrange bar:** a narrow strip of icons between the work area and the side panel with align, centre, turn, mirror, group, lock and order commands for the selection (hover for names). The *Arrange* menu holds the same commands, with curve editing (edit nodes, convert, round corners) together and *Adjust image* / *Trace image* apart.
* **Group** (Ctrl+G) and **lock** (Ctrl+L). Locked objects cannot be moved, edited or deleted but can be selected and copied (the copy is unlocked).

## Selection
The square handles around a selection resize it. **Click a handle** (without dragging) and they become round corner handles that rotate the selection about its middle (Shift snaps to 15°) and diamond edge handles that slant it along that edge; click a handle again to go back to resizing.

**Snap to objects** (the target icon next to the magnet in the control bar, and View > Snap to objects; saved with the settings): while you move a selection, its left, middle and right line (top, middle, bottom) jump to the same line of a nearby visible object when it comes within about 8 screen pixels, and a magenta guide line shows where it joined. It works independently of *Snap to grid*; objects on hidden layers are not snap targets.

Drag left → right: only objects wholly inside. Right → left: everything touched. Shift+click adds. Escape twice returns to the select tool. The right-click menu holds the common commands.

## View and navigation
Pan (H), zoom in (Z), zoom out (X), *Fit work area* (Ctrl+Shift+0), *Fit all objects* (Ctrl+Shift+9). Hold **Space** to pan with any tool. The work area auto-fits the window until you zoom by hand. *Settings → View options*: main and secondary grid, background colour, line thickness.
![Grid](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/15-grid-layers.png)

## Importing
* **SVG:** curves stay Béziers; colours map to layers.
* **Bitmaps:** PNG, JPEG, BMP, GIF, WebP, with rotate, flip, brightness, contrast, gamma, auto levels, negative.
  ![Image import](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/13-image-import.png)
* **Illustrator / PDF:** first page, vector paths (Illustrator files need PDF compatibility).
* **DXF (`.dxf`):** *File → Import → Import DXF…* (also *Open…* and drag and drop) reads an ASCII DXF (R12 and newer): lines, polylines with bulges, arcs, circles, ellipses, splines and blocks (with their scale and rotation) as outlines, in millimetres (the unit of the file is read), arcs and circles as curves. Lines that meet (within 0.01 mm) become one path. The layers of the DXF go to the layers of the program one after the other (the first on layer 1, and so on; the rest on the last). What is not a line (text, hatches, dimensions) is left out and named in the status bar. *Arrange → Auto-join paths…* / *Close path with tolerance…* join and close paths by a tolerance of your own, and *File → Export → Export selection as SVG… / DXF…* write the selected shapes.
* **LightBurn projects (`.lbrn2`, `.lbrn`):** *File → Import → Import LightBurn project…* (or drag and drop) reads rectangles, ellipses, polygons, stars, paths with curves, groups, text (as outlines, so it looks the same without the font) and bitmaps with their rotation and mirroring. The layers come with their kind (line, fill, offset fill, image), speed, power, passes and interval, and the drawing is turned over to this program's Y axis using the height of the selected device's work area. The shapes are added to the open drawing (one undo takes them back); the layers of the file are used for layers that have no shapes yet. Things LightBurn has that are not read yet (for example tool layers T1/T2, variable text, QR codes) are left out and named in the message. Tested against projects saved by LightBurn 2.1.04.
* **UGS Designer files (`.ugsd`):** *File → Import → Import UGS design…* (or drag and drop) reads paths with curves, rectangles (also with rounded corners), ellipses, points, text, pictures and groups; hidden objects are drawn too. Every way of cutting becomes a layer: pockets and fills as *Fill*, the rest as *Line*. UGS is a milling program, so its feeds and depths are not taken for the laser. V-carving, surfacing, height maps, tabs and start depths are not supported and are named in the message.
* **Trace image:** turns dark or light areas into Béziers with threshold, speckle filter, simplification and smoothing.
  ![Trace](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/14-trace.png)

## Saving and export
Native `.cre` projects (images embedded), SVG, G-code, HPGL (`.plt`) and DXF.
