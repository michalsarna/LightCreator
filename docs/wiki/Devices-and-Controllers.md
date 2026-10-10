🌐 **English** · [Polski](Urzadzenia-i-sterowniki)

# Devices and Controllers

![Device configuration](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/05-device-config.png)

## Device profiles
The start window selects a **device profile** and remembers it. A profile holds controller, laser type (diode or CO2), units (mm or inches; geometry is always stored in mm), speed units (per second or per minute, converted automatically; the G-code always carries the mm/min feed after `G21`), the default speed of each layer, work area, machine zero, S-value max, travel speed, baud rate, port, jog step and feed, frame power and camera alignment. The configuration window groups them into **Device**, **Connection** (serial or TCP) and **Camera**. The editor needs at least one profile.

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
