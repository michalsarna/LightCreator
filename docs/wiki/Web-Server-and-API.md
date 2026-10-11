🌐 **English** · [Polski](Serwer-WWW-i-API)

# Web Server and API

![Options](https://raw.githubusercontent.com/michalsarna/LightCreator/main/docs/screenshots/36-options-web-server.png)

*Program options → Web server and API* switches on a small web server built into LightCreator. It is **off by default**. It shows the state of the machine, the project and a camera in a browser (also on a phone), and offers an API for other programs.

**Every request needs the access phrase** (set in the same options with the *Save* button next to it; it starts as `LightCreator`, change it before you let other computers in). The browser asks for it (any user name, the phrase is the password field); programs send it as HTTP Basic authentication, e.g. `curl -u any:LightCreator http://localhost:8765/api/status`. A wrong phrase is answered with 401 after a short wait. **There is no encryption**, so the phrase and the data can be read by anyone who can listen on the network, and the phrase is kept in plain text in `app.ron`. By default the server listens only on this computer (`http://localhost:8765`). *Reachable from other computers on the network* makes it listen on every interface: then anyone on the network can see the machine and stop or move it. Use that only on a network you trust, and never open the port to the internet.

## The web page
`http://localhost:<port>/` shows the device and its state, the position, the job's working time over the estimate, the project as a picture, the camera (a list lets you choose which camera is active; it changes the active camera of the device), and the buttons STOP, Home, Cancel jog and the eight jog arrows (one step of the device's jog step).

## The API
All answers are JSON unless said otherwise. Commands need `POST`.

| Request | Does |
|---|---|
| `GET /api/status` | the program, the device (name, controller, connected, state, X, Y), the job (running, done / total lines, elapsed and estimated seconds) and the project (file name, revision, description) |
| `GET /api/project.svg` | the project as SVG |
| `GET /api/cameras` | the cameras: index, name, enabled, active, `url` (the camera's own http / https address, or `null`) and `snapshot` |
| `GET /api/camera.jpg` | the newest picture of the camera view as JPEG (only while the Camera view is open in the program; otherwise 503) |
| `POST /api/stop` | stops the running job (like STOP) |
| `POST /api/home` | homing, only with a connection and no job running |
| `POST /api/cancel-jog` | cancels a jog |
| `POST /api/jog?dir=…` | one jog step: `up`, `down`, `left`, `right`, `up-left`, `up-right`, `down-left`, `down-right`; only with a connection and no job running, inside the allowed area |
| `POST /api/camera/select?index=N` | makes camera N the active one |

The commands follow the rules of the buttons they stand for. Nothing here has been tried against real hardware yet; the server itself is covered by tests.
