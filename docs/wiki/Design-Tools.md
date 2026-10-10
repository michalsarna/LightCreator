🌐 **English** · [Polski](Narzedzia-projektowe)

# Design Tools

![Node editing](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/07-nodes.png)

## Drawing
* Rectangle, ellipse, line, polyline and text tools; select, move and resize with handles.
* **Automatic shapes:** triangle (Y), five-pointed star (S), regular polygon (G, 3 to 360 sides chosen in a popup), heart (K) and spiral (I). Drag a box (Shift keeps it square); the result is an ordinary editable Bézier shape. A small window in the corner of the work area sets the number of sides or of turns (3 by default); it has no OK button, disappears once the shape is placed, returns with the next one, and a change in it also reshapes the shape drawn last.
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

Drag left → right: only objects wholly inside. Right → left: everything touched. Shift+click adds. Escape twice returns to the select tool. The right-click menu holds the common commands.

## View and navigation
Pan (H), zoom in (Z), zoom out (X), *Fit work area* (Ctrl+0), *Fit all objects* (Ctrl+9). Hold **Space** to pan with any tool. The work area auto-fits the window until you zoom by hand. *Settings → View options*: main and secondary grid, background colour, line thickness.
![Grid](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/15-grid-layers.png)

## Importing
* **SVG:** curves stay Béziers; colours map to layers.
* **Bitmaps:** PNG, JPEG, BMP, GIF, WebP, with rotate, flip, brightness, contrast, gamma, auto levels, negative.
  ![Image import](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/13-image-import.png)
* **Illustrator / PDF:** first page, vector paths (Illustrator files need PDF compatibility).
* **Trace image:** turns dark or light areas into Béziers with threshold, speckle filter, simplification and smoothing.
  ![Trace](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/14-trace.png)

## Saving and export
Native `.lcr` projects (images embedded), SVG, G-code, HPGL (`.plt`) and DXF.
