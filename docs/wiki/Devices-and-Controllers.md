🌐 **English** · [Polski](Urzadzenia-i-sterowniki)

# Devices and Controllers

![Device configuration](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/05-device-config.png)

## Device profiles
The start window selects a **device profile** and remembers it. A profile holds controller, laser type (diode or CO2), units (mm or inches; geometry is always stored in mm), work area, machine zero, S-value max, travel speed, baud rate, port, jog step and feed, frame power and camera alignment. The configuration window groups them into **Device**, **Connection** (serial or TCP) and **Camera**. The editor needs at least one profile.

## Controllers
| Controller | Support |
|---|---|
| **GRBL** | streaming over serial or TCP: connect, jog, home, unlock, pause / resume, stop, framing, progress; *Read from device* fills work area, S max, travel speed and laser mode |
| **Marlin** | same (laser feature, inline power); reads feed rate |
| **Ruida**, **Trocen** | export HPGL / DXF with one pen or layer per colour, opened in the vendor's software (e.g. RDWorks). Fill becomes hatch lines; raster images are skipped. Direct binary protocols are not implemented |

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
Give the device a camera URL (MJPEG stream or JPEG snapshot, http / https; RTSP is not supported) to get a **Camera view** button. The view can float, open as a separate OS window, or dock as a side tab, and can be rotated 90° / 180° / 270° (a device setting). The **Overlay** button and *View → Show camera overlay* put the picture under the design.
![Camera tab](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/17-camera-tab.png)

**Camera overlay alignment:** a photo or live camera sits under the design, aligned with four draggable corners (perspective corrected), with opacity, flip and rotate, saved per device.
![Overlay](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/10-camera-overlay.png)
