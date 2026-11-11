🌐 **English** · [Polski](Panele-i-karty)

# Panes and Tabs

How to arrange the work area of LightCreator: a drawing next to its preview, two drawings at once, the preview on a second monitor. Everything here is the same in LightCreator, MillCreator and PenCreator.

```
+--------------------------------------------+------------------+
| drawing A | drawing B | + || Preview | +    | Properties | ... |
|---------------------------||----------------|                  |
|                           ||                |   right panel    |
|          pane 1           ||     pane 2     |  (its own tabs)  |
|                           ||                |                  |
+--------------------------------------------+------------------+
```

## What is a tab

The work area shows **tabs**, in a bar above it:

* every open **drawing** (a dot on the tab means unsaved changes);
* the **Preview**;
* **Preview G-code**: the G-code of a project as text;
* the **G-code analysis** of a file;
* the **Camera view**, when it is shown as a tab in the work area.

The **+** at the end of a bar makes a new drawing. A tab exists once: opening a project that is open already shows its tab, and a project has one preview, one G-code text and one camera view.

A **pane** is a part of the work area with its own bar of tabs. There are **three panes at most**. With one pane (the start) you see just the one bar.

## Make a second pane

Open at least **two tabs** first: a pane needs a tab of its own, so with one tab nothing happens. Then either:

* **View → Split view: side by side** or **View → Split view: one above the other**: the last tab of the bar goes to a new pane; or
* right-click the tab you want to move and choose **Move to pane 2**.

## Make a third pane

Right-click a tab in a pane with two tabs or more and choose **Move to pane 3**. The **Split view** entries make only the second pane.

## Move a tab

* **Drag the tab** onto the bar of another pane: a line shows where it lands. Or drop it on the body of that pane (the pane gets a frame): the tab goes to the end of its bar.
* Drag it along its own bar to change the order.
* Or use its menu (right click): **Move to pane N**.

Dragging moves a tab only between the panes there are: a new pane comes from the menu.

## Change the sizes and the direction

* **Drag the line between two panes** to change their sizes. A pane does not get smaller than 160 points.
* **Double-click the line** to make the panes equal.
* With two or three panes, **View → Split view: …** changes the direction: side by side or one above the other. The tabs stay where they are.

## Back to one pane

* **View → Single view** puts all the tabs in one pane again.
* Closing the last tab of a pane closes the pane, and the others close up. So does moving its last tab away.
* Closing the last open drawing (after the question about saving it, when it has unsaved work) asks **Close the program?**: **Close the program** closes it, **Back to the start screen** (or Escape) shows the start screen.

## Close a tab

Click its cross, click it with the **middle button**, or choose **Close tab** in its menu (right click). A drawing with unsaved changes asks first.

## Which drawing the tools work on

One drawing is the **active** one: the tools, the menus and the keys work on it. A drawing in another pane is shown for looking only, with a zoom of its own. **Click it, or its tab, to make it the active one.**

## A tab in a window of its own

The **Preview**, the **G-code text** and the **Camera view** can go to a separate window, for example on a second monitor. A drawing cannot.

1. Right-click the tab and choose **Open in a separate window**.
2. Move the window where you want it. It remembers its place and its size.
3. **Back to the main window**, at the top of the window, or closing the window puts the tab back, in the pane you used last.

**View → Single view** does not touch these windows.

## Three ways to work

* **Drawing and preview side by side:** open the preview (**Laser → Preview window…** or the **Preview…** button; Ctrl+Shift+P), then **View → Split view: side by side**. Check that the preview tab is the one that went to pane 2; if not, drag it there.
* **Two drawings at once:** open both, then **View → Split view: side by side**. Click the one you want to edit: the other stays in view with its own zoom.
* **Preview on a second monitor:** right-click the preview tab, **Open in a separate window**, and drag the window to that monitor.

## The right panel

The panel at the right is separate from the panes. It has its own tabs: **Properties**, **Layers**, **Device**, **Control**, **Console**, **Clipart**, **Description** and **Recording** (a panel for each camera, the photos aside, the long lines), and the camera or a plugin when they ask for one (these two stay at the end of the row).

* **Drag a tab** in its row to change the order. The order is remembered.
* **Drag the left edge** of the panel to change its width.
* The arrow at the start of the row **hides the panel**: the names of the tabs stay on a narrow strip, written vertically. Click a name to open the panel on that tab; the button on the strip (**Show the panel**) opens it again.

Tabs of the right panel and tabs of the work area do not mix: you cannot drag one to the other.

## Remembered

The panes, the tabs in them, the direction and the sizes are saved with the settings of the program and come back when you start it again, without the tabs of what is no longer open. So do the order of the right panel's tabs and whether it is hidden.

See also: [Design Tools](Design-Tools), [Keyboard Shortcuts](Keyboard-Shortcuts), [Menus](Menus).
