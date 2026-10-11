🌐 **English** · [Polski](Urzadzenia-i-sterowniki)

# Devices and Controllers

![Device configuration](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/05-device-config.png)

## Axes, work area and the check before a job
* **Machine zero** can be at any of the four corners; **Invert axes** reverses X and/or Y, and **Panel orientation** (native, rotated left, rotated right) turns the work area the way the panel sits in front of you (the area is shown turned and the G-code is mapped to match). Everything goes through one design-to-machine transform, so the jog arrows, the zero marker, frame, G-code, HPGL and DXF agree.
* **Margins** are strips along the edges that the machine cannot use (clamps, a frame); they are hatched on the canvas. **Software limits** are an optional own rectangle inside the usable area (outlined in orange). The jog buttons stop at the edge of the allowed area; a head that stands outside may always move back in.
* **Check before the job:** Start, Frame, Save G-code and the file export first check the job. An object or toolpath (overscan included) outside the usable area or the software limits is an **error**: nothing is sent, framed or exported, and the window offers *Select objects*. Layers that burn at 0 % power, or hold objects that give no toolpath, are **warnings** you can confirm. The G-code writer no longer clips coordinates silently.
* **After connecting** a GRBL controller is read once (`$$`). If laser mode (`$32`) is off a window warns that the laser may fire on rapid moves and offers *Enable laser mode ($32=1)* after a confirmation; if the controller's `$30` differs from the S max of the profile you can *Use the value from the controller*.

## Device profiles
The start window selects a **device profile** and remembers it. A profile holds controller, laser type (diode or CO2), units (mm or inches; geometry is always stored in mm), speed units (per second or per minute, converted automatically; the G-code always carries the mm/min feed after `G21`), the default speed of each layer, work area, machine zero, S-value max, travel speed, baud rate, port, jog step and feed, frame power and camera alignment. The configuration window is a resizable window with a list of pages on the left: **General** (name, controller, units, speeds, jog), **Hardware** (work area, machine zero), **Laser** (laser type, S-value max, frame power), **G-code**, **Connection** (serial or TCP; only for controllers with a live link) and **Camera**. Profiles saved by earlier versions are read as they are; the laser settings live in a list of laser heads (one for now). The editor needs at least one profile.

## Controllers
| Controller | Support |
|---|---|
| **GRBL** | streaming over serial or TCP: connect, jog, home, unlock, pause / resume, stop, framing, progress; *Read from device* fills work area, S max, travel speed and laser mode |
| **Marlin** | same (laser feature, inline power); reads feed rate |
| **Ruida**, **Trocen** | export HPGL / DXF with one pen or layer per colour, opened in the vendor's software (e.g. RDWorks). Fill becomes hatch lines; raster images are skipped. Direct binary protocols are not implemented |

## Device tab and the control bar
* **Start** is enabled when a laser is connected (Ruida and Trocen export a file instead) and turns into a red **STOP** while the job runs; **Frame** needs a connection and is disabled during a job.
* In the Device tab the jog arrows, Home, Unlock, Pause and Resume are large icon buttons of one size (the orange square cancels a jog); STOP is a wide red button below them. Jog step, jog feed and frame power sit together above them.

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
