🌐 **English** · [Polski](Serwer-WWW-i-API)

# Web Server and API

![Options](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/36-options-web-server.png)

**Settings → Program options… → Web server and API** switches on a small web server built into LightCreator. It is **off by default**. It shows the state of the machine, the project and a camera in a browser (also on a phone), and offers an API for other programs.

**Every request needs the access phrase** (set in the same options with the *Save* button next to it; it starts as `LightCreator`, change it before you let other computers in). The browser asks for it (any user name, the phrase is the password field); programs send it as HTTP Basic authentication, e.g. `curl -u any:LightCreator http://localhost:8765/api/status`. A wrong phrase is answered with 401 after a short wait. **There is no encryption**, so the phrase and the data can be read by anyone who can listen on the network, and the phrase is kept in `app.ron` as base64 (not readable at a glance, but base64 is an encoding, not encryption). By default the server listens only on this computer (`http://localhost:8765`). *Reachable from other computers on the network* makes it listen on every interface: then anyone on the network can see the machine and stop or move it. Use that only on a network you trust, and never open the port to the internet.

## The web page
`http://localhost:<port>/` has a bar with the device, its state, the position, the job's working time over the estimate and a progress bar, and a **Theme** list: the page starts in the colour scheme of the program (all 14 schemes are offered; your choice is kept in the browser).

On a computer the page has two columns: on the left two thirds the **camera** (a list picks the active camera, which changes the active camera of the device; with no camera there is a big crossed-out camera), on the right third the **control panel** (STOP, Home, Cancel jog and the eight jog arrows, one step of the device's jog step) and below it the **project** as a picture. Double-click the picture (or press the full-screen button) to enlarge it over the whole window, with buttons to zoom in and out, move and reset the view; the mouse wheel, dragging and two-finger pinching work too, Esc closes it.

The camera picture is turned like in the program: the *Camera rotation* of the device settings is the default (0°, 90°, 180°, 270° clockwise), and a list above the picture lets you choose another turn, which the browser remembers per camera. `GET /api/cameras` gives each camera's `rotation` and `rotated` (how much of it the snapshot already has).

On a phone the page is one column: the status, the camera, the project and, at the bottom, the control panel.

## The API
All answers are JSON unless said otherwise. Commands need `POST`.

| Request | Does |
|---|---|
| `GET /api/status` | the program, the device (name, controller, connected, state, X, Y), the job (running, done / total lines, elapsed and estimated seconds) and the project (file name, revision, description) |
| `GET /api/project.svg` | the project as SVG |
| `GET /api/cameras` | the cameras: index, name, enabled, active, `rotation`, `rotated`, `url` (the camera's own http / https address, or `null`) and `snapshot` |
| `GET /api/camera.jpg` | the newest picture of the camera view as JPEG (only while the Camera view is open in the program; otherwise 503) |
| `POST /api/stop` | stops the running job (like STOP) |
| `POST /api/home` | homing, only with a connection and no job running |
| `POST /api/cancel-jog` | cancels a jog |
| `POST /api/jog?dir=…` | one jog step: `up`, `down`, `left`, `right`, `up-left`, `up-right`, `down-left`, `down-right`; only with a connection and no job running, inside the allowed area |
| `POST /api/camera/select?index=N` | makes camera N the active one |

The commands follow the rules of the buttons they stand for. Nothing here has been tried against real hardware yet; the server itself is covered by tests.
