🌐 **English** · [Polski](Urzadzenia-i-sterowniki)

# Devices and Controllers

![Device configuration](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/05-device-config.png)

## Axes, work area and the check before a job
* **Machine zero** can be at any of the four corners; **Invert axes** reverses X and/or Y, and **Panel orientation** (native, rotated left, rotated right) turns the work area the way the panel sits in front of you (the area is shown turned and the G-code is mapped to match). Everything goes through one design-to-machine transform, so the jog arrows, the zero marker, frame, G-code, HPGL and DXF agree.
* **Margins** are strips along the edges that the machine cannot use (clamps, a frame); they are hatched on the canvas. **Software limits** are an optional own rectangle inside the usable area (outlined in orange). The jog buttons stop at the edge of the allowed area; a head that stands outside may always move back in.
* **Check before the job:** Start, Frame, Save G-code and the file export first check the job. An object or toolpath (overscan included) outside the usable area or the software limits is an **error**: nothing is sent, framed or exported, and the window offers *Select objects*. Layers that burn at 0 % power, or hold objects that give no toolpath, are **warnings** you can confirm. The G-code writer no longer clips coordinates silently.
* **After connecting** a GRBL controller is read once (`$$`). If laser mode (`$32`) is off a window warns that the laser may fire on rapid moves and offers *Enable laser mode ($32=1)* after a confirmation; if the controller's `$30` differs from the S max of the profile you can *Use the value from the controller*.

## G-code settings of a device
* **Speeds:** *Max cutting speed* and *Acceleration* (0 = unknown) next to the travel speed. With an acceleration the time estimate includes the start and stop ramps (a run of moves at one speed ramps once, a travel move each time); the cutting speed is capped at the maximum; a layer faster than the maximum gets a warning before the job, and the layer panel suggests an overscan equal to the braking distance.
* **Dialect:** automatic from the controller, or GRBL (`M3`), GRBL dynamic (`M4`), Marlin, Smoothieware. Smoothieware is a controller too (streamed like GRBL, jog and settings like Marlin) and writes S as a fraction of the full power (set *S-value max* to 1); its details are assumptions from the documentation, not tried on hardware.
* **Advanced page** (with a warning): own start of the job (replaces units, absolute mode, plane and the laser-mode line), own end of the job (replaces the closing `M2`; the laser is always switched off and the head sent home before it) and the air assist commands. Empty = built in; control characters and oversize text are refused; *Restore defaults* clears them.
* **Power mode per layer:** like the device, constant (`M3`) or dynamic (`M4`); the G-code inserts the switch between layers (GRBL dialects only).
* **Arcs (G2 / G3):** off by default. When on, outlines and rings are fitted with arcs within the *Arc tolerance*; scan lines and images stay lines. The *Curve tolerance* sets how finely Béziers and text are flattened (default 0.05 mm). Not every firmware does arcs well (Marlin needs ARC_SUPPORT), so test with the laser off.

## Connection behaviour and the controller's settings
* **Streaming:** *Count the buffer* (default, fast: several lines are in the controller's buffer) or *Simple* (one line, wait for its `ok`; slow, for firmware that answers wrongly). A running job that waits for an answer for 10 s while the machine stands still is reported in the console and in a window (*Keep waiting* / *Stop the job*); nothing is resumed or sent on its own.
* **USB binding:** the port field takes a port name or a `VID:PID`; *Bind to this port* remembers the USB ids of the chosen port, so the port is found again when its name changes. *Connect again automatically* retries every few seconds after a lost link until you press Disconnect.
* **After connecting:** optionally home the machine (`$H`, `G28`) and/or clear an alarm (`$X`, `M999`; this skips homing, so only when the head is known to be safe). If the firmware reports it was built with single-axis homing (option `H` in `$I`), *Home X* / *Home Y* appear in the Device tab.
* **Firmware ($$) page** (GRBL): reads all settings with their meaning, unit and typical value; critical ones (`$30`, `$31`, `$32`, `$130`–`$132`) are highlighted. A value is checked (range, mask, 0/1) and written only after a confirmation, then read back. *Back up to a file* writes `$n=value` lines with the date and the firmware's `$I`; *Restore from a file* checks every line, shows what differs, writes after a confirmation and compares the answer; *Factory settings ($RST=$)* needs a confirmation and typing `RESET`. For Marlin the page shows the report of `M503`.

## Laser heads
The **Laser** page lists the laser heads of the machine (at least one). A head has a name, a tool number `T`, a colour, a laser type (Diode, CO2, Fiber), wavelength and optical power (0 = the usual for the type), S-value max, frame power and speed, a **focus power**, the **beam spot** and the **pointer offset**. CO2 and fibre heads also keep PWM values (frequency, maximum frequency, pulse width and its range): they are stored with the profile and offered as variables for macros, but they are **not sent to the controller**.
* **Several heads:** each layer has a *Laser head*. The G-code scales the power by the S max of the layer's head and writes `T<n>` before a layer that uses another head (the numbers must match the mapping in your controller; nothing is assumed about how GRBL treats `T`). A job with one head is written exactly as before.
* **Beam spot:** when a fill interval is smaller than the spot of the layer's head the layer panel says neighbouring lines overlap.
* **Focus mode:** *Focus* (control bar and Device tab) switches on a weak constant beam (`M3 S…`, `M5` to end it) to set the height by. A red *FOCUS BEAM ON* button switches it off; it also goes off by itself when a job starts, on Stop, when the link drops and when the program closes. It needs a connection, a focus power above 0 and no running job.
* **Pointer offset:** with an offset set, the Device tab offers *Align to the pointer* (this session only, not saved): the frame is moved back by the offset so the red dot lands where the beam would. *Dry run with the pointer* runs the path of the job shifted by the offset with every `S` set to 0. Jobs and jogging are never moved. The shifted path is checked against the allowed area first.

## Hooks, macros and no-go zones
* **Hooks** (Advanced page, behind the warning): own G-code written into the job at four points: *Layer start*, *Layer end*, *Object start*, *Object end*, each with a switch. The order is layer start, then for every object its start and end, then layer end; the lines carry a `; hook …` comment. `{variables}` are filled in: `{job_name}` (the file name), `{date}` and `{time}` (UTC), `{layer_name}`, `{layer_index}`, `{total_layers}`, `{workpiece_name}`, `{workpiece_index}`, `{total_workpieces}`, `{machine_name}`, `{max_speed}`, `{work_width}`, `{work_height}`, `{tool_number}` and the head's `{wavelength}`, `{optical_power}`, `{pwm_freq}`, `{pwm_max_freq}`, `{pulse_width}`, `{pulse_min}`, `{pulse_max}`. An unknown variable, a brace that does not close, a control character or a line over 100 characters cannot be saved. With object hooks a layer is burnt object by object (and once per pass); without them nothing changes.
* **Macros:** named scripts of own G-code, run from buttons in the Device tab when connected and no job runs. One command per line; comments after `;` or in brackets; `@include(name)` inserts another macro (cycles and more than 8 levels are refused); the variables of the machine can be used. The first run in a session asks for a confirmation. The lines are cleaned like all others before they are sent.
* **No-go zones** (page *No-go zones*): rectangles and circles in design coordinates, each with a switch. They are drawn in red on the work area (*View → Show no-go zones*, remembered). **Every** move that touches an enabled zone, travel moves included, is an error in the job check and blocks Start, Frame and export; the message names the zone and the layer. Touching the edge counts. The program does not route around zones.

## Z axis and rotary attachment
* **Z axis** (page *Hardware*): tick *The machine has a Z axis*, set its travel and, if needed, the direction. Layers then get *Z height* and *Z per pass* (both 0 = Z is not moved): the G-code goes to the height before each pass (`G0 Z…`), stepping by *Z per pass*. The job check refuses heights outside the travel, the Device tab has Z Up / Down buttons that stop at the travel, and Z is read from the status reports. Without the axis none of this is written.
* **Rotary** (page *Rotary*): a chuck (the motor turns the work) or rollers (the work lies on rollers and is turned by them). The model is deliberately simple, so check it on your machine:
  * circumference of the work `C = π × workpiece diameter`; the design may not be higher than that (job check);
  * *Replaces Y* mode: Y stays millimetres on the surface and the controller needs `$101 = microsteps per turn / (π × drive diameter)`, where the drive diameter is the workpiece for a chuck and the roller for rollers (the surface of the work moves like the surface of a roller). The page shows the value and can write it through the firmware editor, after a confirmation;
  * *True A axis* mode: `A` in degrees takes the place of `Y`, `A = Y × 360 / (π × drive diameter)`. It needs firmware with a fourth axis; GRBL 1.1 has three, so there the option is disabled and Y is used. No arcs are written on the A axis;
  * layers are marked *rotary*; rotary and flat layers cannot be in one job; the canvas marks where the circumference repeats.
* **No-go zones with Z:** with a Z axis a zone can be limited to a range of heights (like a box or a cylinder standing on the table); a move at a height outside the range is free. Layers that do not set Z are treated as being at every height.

## Device profiles
The start window selects a **device profile** and remembers it. A profile holds controller, laser type (diode or CO2), units (mm or inches; geometry is always stored in mm), speed units (per second or per minute, converted automatically; the G-code always carries the mm/min feed after `G21`), the default speed of each layer, work area, machine zero, S-value max, travel speed, baud rate, port, jog step and feed, frame power and camera alignment. The configuration window is a resizable window with a list of pages on the left: **General** (name, controller, units, speeds, jog), **Hardware** (work area, machine zero), **Laser** (laser type, S-value max, frame power), **G-code**, **Connection** (serial or TCP; only for controllers with a live link) and **Camera**. Profiles saved by earlier versions are read as they are; the laser settings live in a list of laser heads (one for now). The editor needs at least one profile.

**Templates and the first-run setup.** *Add device…* on the start screen offers generic templates (GRBL diode, GRBL CO2, K40 conversion, Marlin diode) and, optionally, a read of the real values from the controller; a short setup then asks for the name and units and ends with a summary of what is worth checking. The templates are generic starting points, not profiles of particular machines. A profile can be **duplicated**, **exported** and **imported** (a JSON file with a format version; the import is validated strictly, a newer format is refused, and the serial port is never taken from the file).

**Maintenance.** Each device keeps the total working time and your own counters (for example "lens cleaned") with optional limits. The time is counted from the moment jobs really run; a reminder is shown once per session and on the start screen, and a counter can be reset after confirming. Copies and imports start at zero.

**Notes.** Every device has notes in Markdown with a preview (a small built-in renderer; only http / https links). Notes that come from a template are read-only.

## Controllers
| Controller | Support |
|---|---|
| **GRBL** | streaming over serial or TCP: connect, jog, home, unlock, pause / resume, stop, framing, progress; *Read from device* fills work area, S max, travel speed and laser mode |
| **Marlin** | same (laser feature, inline power); reads feed rate |
| **Ruida**, **Trocen** | export HPGL / DXF with one pen or layer per colour, opened in the vendor's software (e.g. RDWorks). Fill becomes hatch lines; raster images are skipped. Direct binary protocols are not implemented |

## Device tab and the control bar
* **Start** is enabled when a laser is connected (Ruida and Trocen export a file instead) and turns into a red **STOP** while the job runs; **Frame** needs a connection and is disabled during a job. The **Home** icon between *Save G-code* and *Frame* does the same as Home in the Device tab and is off in the same cases (no connection, or a job running).
* In the Device tab the eight jog arrows (the four diagonals move X and Y by one step at once), Home, Unlock, Pause and Resume are large icon buttons of one size (the orange square cancels a jog); STOP is a wide red button below them. Jog step, jog feed and frame power sit together above them.

![Device tab](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/27-device-tab.png)

## Console
The Console tab shows everything sent to and received from the controller; status polling is hidden by default.
![Console](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/04-console.png)

## TCP and ser2net
A device can connect over the network, e.g. to `ser2net` on a Raspberry Pi. Set *Connection type* to *Network (TCP)* and enter host and port; the baud rate is configured in ser2net.
```yaml
connection: &laser
  accepter: tcp,3333
  connector: serialdev,/dev/ttyUSB0,115200n81,local
```
On Linux, **Laser → Share over network (ser2net)…** does this for you: it picks the port, baud rate and TCP port, writes ser2net 4.x (YAML) or 3.x configuration, shows the address, and can copy, save or install it and restart the service (asks for admin rights via `pkexec`, backs up the old file). A local-only option suits SSH tunnels; otherwise the port is open to the network.
![ser2net](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/18-ser2net.png)

## Camera
Give the device a camera URL (MJPEG stream or JPEG snapshot, http / https; RTSP is not supported) to get a **Camera view** button. The view can float, open as a separate OS window, or dock as a side tab, and can be rotated 90° / 180° / 270° (a device setting). The picture can be zoomed (+ / − buttons or the mouse wheel, up to 8×), dragged, and returned to the whole picture with *Default zoom*; three icons choose floating, separate window or side tab, and the address is not shown. The **Overlay** button (on the control bar, after *Camera view*, and in the camera window) puts the picture under the design; it is always visible and greyed out until there is a picture.
![Camera tab](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/17-camera-tab.png)

**Camera overlay alignment:** a photo or live camera sits under the design, aligned with four draggable corners (perspective corrected), with opacity, flip and rotate, saved per device.
![Overlay](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/10-camera-overlay.png)

**Several cameras.** A device can have any number of cameras (the *Camera* page): each has a name, an on/off switch, a source (an http / https address, or a camera of this computer where the build has camera support), a rotation and one *active* flag; the active one gives the live picture. A camera can only be switched on when its address is valid. Per camera there are **picture settings** (brightness, contrast, white balance, noise reduction, and for a local camera a preference for the uncompressed YUYV format) and a **manual lens correction** (radial k1–k3 and tangential p1, p2 coefficients, entered by hand; all 0 means no correction). Profiles with a single camera URL from earlier versions become one camera.

**Alignment by reference points.** In the overlay window, *Reference points* holds pairs of a point in the picture (pixels) and its place in the work area (mm). With 3 points the fit is affine, with 4 or more it is a full perspective fit (least squares); *Align by the points* moves the four overlay corners to match, and *Points from the current corners* turns the older four-corner alignment into points. The points are stored with the camera.

Not supported: RTSP streams, and automatic lens calibration (ChArUco boards, markers or dot grids); it would need a heavy computer-vision dependency, so the coefficients are entered by hand.

