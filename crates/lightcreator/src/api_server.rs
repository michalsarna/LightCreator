//! The optional web server and API: a status page, the project as an SVG, the cameras, and a few commands (stop,
//! home, cancel a jog, jog). It is NOT protected in any way (no password, no encryption): by default it listens only
//! on this computer. It is a small HTTP/1.1 server on the standard library: one short thread per request.
use serde_json::Value;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// What the API asks the program to do.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Stop,
    Home,
    CancelJog,
    /// One of the eight directions of the Device tab, by name.
    Jog(Jog),
    SelectCamera(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Jog {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl Jog {
    pub fn parse(s: &str) -> Option<Jog> {
        Some(match s {
            "up" => Jog::Up,
            "down" => Jog::Down,
            "left" => Jog::Left,
            "right" => Jog::Right,
            "up-left" => Jog::UpLeft,
            "up-right" => Jog::UpRight,
            "down-left" => Jog::DownLeft,
            "down-right" => Jog::DownRight,
            _ => return None,
        })
    }
    /// Direction as (x, y) in steps, y down the screen like the arrows of the Device tab.
    pub fn delta(self) -> (f64, f64) {
        match self {
            Jog::Up => (0.0, -1.0),
            Jog::Down => (0.0, 1.0),
            Jog::Left => (-1.0, 0.0),
            Jog::Right => (1.0, 0.0),
            Jog::UpLeft => (-1.0, -1.0),
            Jog::UpRight => (1.0, -1.0),
            Jog::DownLeft => (-1.0, 1.0),
            Jog::DownRight => (1.0, 1.0),
        }
    }
}

/// What the program publishes for the server, and what the server hands back.
#[derive(Default)]
pub struct Shared {
    pub status: Mutex<Value>,
    /// The project as SVG, with the revision it was made for.
    pub svg: Mutex<(u64, Arc<String>)>,
    pub cameras: Mutex<Value>,
    /// The latest picture of the camera view in the program: width, height, RGB.
    pub frame: Mutex<Option<(u32, u32, Arc<Vec<u8>>)>>,
    pub commands: Mutex<VecDeque<Command>>,
}

pub struct Server {
    pub addr: SocketAddr,
    stop: Arc<AtomicBool>,
    pub shared: Arc<Shared>,
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

const MAX_HEAD: usize = 8192;
const MAX_CONNECTIONS: usize = 16;

impl Server {
    /// Listen on `port` (0 = any free one); on every interface only when `lan` is set.
    pub fn start(port: u16, lan: bool) -> Result<Server, String> {
        let ip = if lan { Ipv4Addr::UNSPECIFIED } else { Ipv4Addr::LOCALHOST };
        let listener = TcpListener::bind(SocketAddr::from((ip, port))).map_err(|e| e.to_string())?;
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let addr = listener.local_addr().map_err(|e| e.to_string())?;
        let stop = Arc::new(AtomicBool::new(false));
        let shared = Arc::new(Shared::default());
        let (flag, sh) = (stop.clone(), shared.clone());
        let active = Arc::new(AtomicUsize::new(0));
        std::thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((s, _)) => {
                        if active.load(Ordering::Relaxed) >= MAX_CONNECTIONS {
                            continue;
                        }
                        active.fetch_add(1, Ordering::Relaxed);
                        let (sh, active) = (sh.clone(), active.clone());
                        std::thread::spawn(move || {
                            let _ = handle(s, &sh);
                            active.fetch_sub(1, Ordering::Relaxed);
                        });
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(30)),
                    Err(_) => std::thread::sleep(Duration::from_millis(200)),
                }
            }
        });
        Ok(Server { addr, stop, shared })
    }
}

struct Request {
    method: String,
    path: String,
    query: Vec<(String, String)>,
}

fn url_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => {
                match u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    Ok(v) => {
                        out.push(v);
                        i += 3;
                        continue;
                    }
                    Err(_) => out.push(b'%'),
                }
            }
            b'+' => out.push(b' '),
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn parse_request(head: &str) -> Option<Request> {
    let line = head.lines().next()?;
    let mut it = line.split_whitespace();
    let (method, target, version) = (it.next()?, it.next()?, it.next()?);
    if !version.starts_with("HTTP/1.") || !target.starts_with('/') || target.len() > 2000 {
        return None;
    }
    let (path, q) = target.split_once('?').unwrap_or((target, ""));
    let query = q.split('&').filter(|p| !p.is_empty()).map(|p| {
        let (k, v) = p.split_once('=').unwrap_or((p, ""));
        (url_decode(k), url_decode(v))
    }).collect();
    Some(Request { method: method.to_string(), path: url_decode(path), query })
}

fn respond(s: &mut TcpStream, code: u16, ctype: &str, body: &[u8]) -> std::io::Result<()> {
    let reason = match code {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        503 => "Service Unavailable",
        _ => "Error",
    };
    let head = format!("HTTP/1.1 {code} {reason}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n", body.len());
    s.write_all(head.as_bytes())?;
    s.write_all(body)
}

fn json(s: &mut TcpStream, code: u16, v: &Value) -> std::io::Result<()> {
    respond(s, code, "application/json; charset=utf-8", v.to_string().as_bytes())
}

fn handle(mut s: TcpStream, sh: &Shared) -> std::io::Result<()> {
    s.set_read_timeout(Some(Duration::from_secs(5)))?;
    s.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut buf = Vec::with_capacity(1024);
    let mut chunk = [0u8; 1024];
    while !buf.windows(4).any(|w| w == b"\r\n\r\n") {
        let n = s.read(&mut chunk)?;
        if n == 0 || buf.len() + n > MAX_HEAD {
            return respond(&mut s, 400, "text/plain", b"bad request");
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    let head = String::from_utf8_lossy(&buf).to_string();
    let Some(req) = parse_request(&head) else { return respond(&mut s, 400, "text/plain", b"bad request") };
    route(&mut s, &req, sh)
}

fn route(s: &mut TcpStream, req: &Request, sh: &Shared) -> std::io::Result<()> {
    let get = req.method == "GET" || req.method == "HEAD";
    let post = req.method == "POST";
    let q = |k: &str| req.query.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str());
    let push = |c: Command| sh.commands.lock().map(|mut v| if v.len() < 64 { v.push_back(c) }).is_ok();
    match (req.path.as_str(), get, post) {
        ("/", true, _) => respond(s, 200, "text/html; charset=utf-8", PAGE.as_bytes()),
        ("/api/status", true, _) => {
            let v = sh.status.lock().map(|v| v.clone()).unwrap_or(Value::Null);
            json(s, 200, &v)
        }
        ("/api/project.svg", true, _) => {
            let svg = sh.svg.lock().map(|v| v.1.clone()).unwrap_or_default();
            respond(s, 200, "image/svg+xml; charset=utf-8", svg.as_bytes())
        }
        ("/api/cameras", true, _) => {
            let v = sh.cameras.lock().map(|v| v.clone()).unwrap_or(Value::Null);
            json(s, 200, &v)
        }
        ("/api/camera.jpg", true, _) => {
            let frame = sh.frame.lock().ok().and_then(|f| f.clone());
            match frame.and_then(|(w, h, rgb)| encode_jpeg(w, h, &rgb)) {
                Some(jpg) => respond(s, 200, "image/jpeg", &jpg),
                None => respond(s, 503, "text/plain", b"no picture: open the camera view in the program"),
            }
        }
        ("/api/stop", _, true) => {
            push(Command::Stop);
            json(s, 200, &serde_json::json!({"ok": true}))
        }
        ("/api/home", _, true) => {
            push(Command::Home);
            json(s, 200, &serde_json::json!({"ok": true}))
        }
        ("/api/cancel-jog", _, true) => {
            push(Command::CancelJog);
            json(s, 200, &serde_json::json!({"ok": true}))
        }
        ("/api/jog", _, true) => match q("dir").and_then(Jog::parse) {
            Some(d) => {
                push(Command::Jog(d));
                json(s, 200, &serde_json::json!({"ok": true}))
            }
            None => json(s, 400, &serde_json::json!({"error": "dir must be one of up, down, left, right, up-left, up-right, down-left, down-right"})),
        },
        ("/api/camera/select", _, true) => match q("index").and_then(|v| v.parse::<usize>().ok()).filter(|i| *i < 8) {
            Some(i) => {
                push(Command::SelectCamera(i));
                json(s, 200, &serde_json::json!({"ok": true}))
            }
            None => json(s, 400, &serde_json::json!({"error": "index must be a camera number"})),
        },
        (p, _, _) if p.starts_with("/api/") && !(get || post) => respond(s, 405, "text/plain", b"method not allowed"),
        ("/api/stop" | "/api/home" | "/api/cancel-jog" | "/api/jog" | "/api/camera/select", true, _) => respond(s, 405, "text/plain", b"use POST"),
        _ => respond(s, 404, "text/plain", b"not found"),
    }
}

/// An RGB picture as JPEG, or `None` when it cannot be made.
pub fn encode_jpeg(w: u32, h: u32, rgb: &[u8]) -> Option<Vec<u8>> {
    if w == 0 || h == 0 || rgb.len() < (w as usize) * (h as usize) * 3 {
        return None;
    }
    let mut out = vec![];
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 70).encode(&rgb[..(w as usize) * (h as usize) * 3], w, h, image::ExtendedColorType::Rgb8).ok()?;
    Some(out)
}

/// The page of the web server: status, the project, the camera and the commands.
const PAGE: &str = r##"<!doctype html>
<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>LightCreator</title>
<style>
body{font-family:system-ui,sans-serif;margin:0;background:#1d1f23;color:#e6e6e6}
header{padding:8px 14px;background:#2a2d33;display:flex;gap:16px;align-items:center;flex-wrap:wrap}
main{display:flex;flex-wrap:wrap;gap:14px;padding:14px}
section{background:#2a2d33;border-radius:8px;padding:10px;flex:1 1 360px}
h2{margin:0 0 8px;font-size:14px;color:#aab}
img.p{width:100%;background:#fff;border-radius:4px}
button{font-size:15px;padding:10px 14px;border:0;border-radius:6px;background:#3d4350;color:#fff;cursor:pointer}
button:hover{background:#4d5568}
button.stop{background:#c03030;font-weight:bold}
.pad{display:grid;grid-template-columns:repeat(3,56px);gap:6px;justify-content:center;margin:8px 0}
.pad button{padding:12px 0}
#state{font-weight:bold}
select{font-size:15px;padding:6px}
.warn{color:#e0a030;font-size:12px}
.bar{background:#3d4350;border-radius:4px;height:10px;overflow:hidden}
.bar i{display:block;height:100%;background:#2ea04f;width:0}
</style></head><body>
<header><b>LightCreator</b><span id="dev"></span><span id="state"></span><span id="pos"></span><span id="time"></span></header>
<main>
<section><h2>Project</h2><img class="p" id="proj" alt="project"><div class="bar"><i id="bar"></i></div></section>
<section><h2>Camera</h2><select id="cam"></select><div id="camview" style="margin-top:8px"></div></section>
<section><h2>Control</h2>
<button class="stop" onclick="cmd('stop')">STOP</button> <button onclick="cmd('home')">Home</button> <button onclick="cmd('cancel-jog')">Cancel jog</button>
<div class="pad" id="pad"></div>
<div class="warn">No password or encryption: use only on a network you trust.</div></section>
</main>
<script>
const dirs=[["up-left","↖"],["up","↑"],["up-right","↗"],["left","←"],["","■"],["right","→"],["down-left","↙"],["down","↓"],["down-right","↘"]];
const pad=document.getElementById('pad');
for(const [d,t] of dirs){const b=document.createElement('button');b.textContent=t;if(d)b.onclick=()=>fetch('/api/jog?dir='+d,{method:'POST'});else b.onclick=()=>cmd('cancel-jog');pad.appendChild(b);}
function cmd(c){fetch('/api/'+c,{method:'POST'});}
const f=s=>{s=Math.max(0,Math.floor(s||0));const p=n=>String(n).padStart(2,'0');return p(Math.floor(s/3600))+':'+p(Math.floor(s/60)%60)+':'+p(s%60)};
let rev=-1,camsig='';
async function tick(){
 try{
  const st=await (await fetch('/api/status')).json();
  document.getElementById('dev').textContent=(st.device&&st.device.name)||'';
  document.getElementById('state').textContent=st.device?(st.device.connected?st.device.state:'not connected'):'';
  document.getElementById('pos').textContent=st.device&&st.device.connected?('X '+st.device.x.toFixed(2)+'  Y '+st.device.y.toFixed(2)):'';
  document.getElementById('time').textContent=st.job?(f(st.job.elapsed_s)+' / '+f(st.job.estimate_s)):'';
  document.getElementById('bar').style.width=st.job&&st.job.total?Math.round(100*st.job.done/st.job.total)+'%':'0';
  if(st.project&&st.project.revision!==rev){rev=st.project.revision;document.getElementById('proj').src='/api/project.svg?r='+rev;}
  const cams=await (await fetch('/api/cameras')).json();
  const sig=JSON.stringify(cams);
  if(sig!==camsig){camsig=sig;const sel=document.getElementById('cam');sel.innerHTML='';
   (cams.cameras||[]).forEach(c=>{const o=document.createElement('option');o.value=c.index;o.textContent=c.name+(c.enabled?'':' (off)');if(c.active)o.selected=true;sel.appendChild(o);});
   sel.onchange=()=>fetch('/api/camera/select?index='+sel.value,{method:'POST'});
   const v=document.getElementById('camview');const a=(cams.cameras||[]).find(c=>c.active);
   v.innerHTML='';
   if(a&&a.url){const i=document.createElement('img');i.className='p';i.src=a.url;v.appendChild(i);}
   else if(a){const i=document.createElement('img');i.className='p';i.id='snap';v.appendChild(i);}
  }
  const snap=document.getElementById('snap');if(snap)snap.src='/api/camera.jpg?t='+Date.now();
 }catch(e){document.getElementById('state').textContent='no connection to the program';}
}
tick();setInterval(tick,1000);
</script></body></html>"##;

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(addr: SocketAddr, raw: &str) -> (u16, String) {
        let mut s = TcpStream::connect(addr).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        s.write_all(raw.as_bytes()).unwrap();
        let mut out = vec![];
        let _ = s.read_to_end(&mut out);
        let text = String::from_utf8_lossy(&out).to_string();
        let code = text.split_whitespace().nth(1).and_then(|c| c.parse().ok()).unwrap_or(0);
        (code, text.split("\r\n\r\n").nth(1).unwrap_or("").to_string())
    }

    fn get(addr: SocketAddr, path: &str) -> (u16, String) {
        ask(addr, &format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n"))
    }

    fn post(addr: SocketAddr, path: &str) -> (u16, String) {
        ask(addr, &format!("POST {path} HTTP/1.1\r\nHost: x\r\nContent-Length: 0\r\n\r\n"))
    }

    #[test]
    fn the_server_answers_status_svg_cameras_and_the_page() {
        let srv = Server::start(0, false).unwrap();
        assert!(srv.addr.ip().is_loopback(), "only this computer unless asked");
        *srv.shared.status.lock().unwrap() = serde_json::json!({"device": {"name": "Laser", "connected": true}});
        *srv.shared.svg.lock().unwrap() = (3, Arc::new("<svg/>".into()));
        *srv.shared.cameras.lock().unwrap() = serde_json::json!({"cameras": [{"index": 0, "name": "A", "url": "http://a/x", "active": true, "enabled": true}]});
        let (c, b) = get(srv.addr, "/api/status");
        assert_eq!(c, 200);
        assert_eq!(serde_json::from_str::<Value>(&b).unwrap()["device"]["name"], "Laser");
        assert_eq!(get(srv.addr, "/api/project.svg"), (200, "<svg/>".to_string()));
        assert!(get(srv.addr, "/api/cameras").1.contains("http://a/x"));
        let (c, page) = get(srv.addr, "/");
        assert!(c == 200 && page.contains("LightCreator") && page.contains("STOP"));
        assert_eq!(get(srv.addr, "/nope").0, 404);
        assert_eq!(get(srv.addr, "/api/camera.jpg").0, 503, "no picture yet");
    }

    #[test]
    fn commands_are_queued_by_post_only_and_checked() {
        let srv = Server::start(0, false).unwrap();
        assert_eq!(get(srv.addr, "/api/stop").0, 405, "a command needs POST");
        assert_eq!(post(srv.addr, "/api/stop").0, 200);
        assert_eq!(post(srv.addr, "/api/home").0, 200);
        assert_eq!(post(srv.addr, "/api/cancel-jog").0, 200);
        assert_eq!(post(srv.addr, "/api/jog?dir=up-left").0, 200);
        assert_eq!(post(srv.addr, "/api/jog?dir=sideways").0, 400);
        assert_eq!(post(srv.addr, "/api/jog").0, 400);
        assert_eq!(post(srv.addr, "/api/camera/select?index=2").0, 200);
        assert_eq!(post(srv.addr, "/api/camera/select?index=99").0, 400);
        assert_eq!(post(srv.addr, "/api/camera/select?index=-1").0, 400);
        let cmds: Vec<Command> = srv.shared.commands.lock().unwrap().drain(..).collect();
        assert_eq!(cmds, vec![Command::Stop, Command::Home, Command::CancelJog, Command::Jog(Jog::UpLeft), Command::SelectCamera(2)]);
    }

    #[test]
    fn bad_requests_are_refused_and_the_server_keeps_running() {
        let srv = Server::start(0, false).unwrap();
        assert_eq!(ask(srv.addr, "garbage\r\n\r\n").0, 400);
        assert_eq!(ask(srv.addr, "GET relative HTTP/1.1\r\n\r\n").0, 400);
        assert_eq!(ask(srv.addr, "GET / SPDY/3\r\n\r\n").0, 400);
        let huge = format!("GET /{} HTTP/1.1\r\n\r\n", "a".repeat(20_000));
        assert_eq!(ask(srv.addr, &huge).0, 400);
        assert_eq!(get(srv.addr, "/api/status").0, 200, "still alive");
        // The queue of commands is bounded.
        for _ in 0..200 {
            post(srv.addr, "/api/stop");
        }
        assert!(srv.shared.commands.lock().unwrap().len() <= 64);
    }

    #[test]
    fn a_port_in_use_is_reported_and_dropping_the_server_frees_it() {
        let a = Server::start(0, false).unwrap();
        let port = a.addr.port();
        assert!(Server::start(port, false).is_err());
        drop(a);
        std::thread::sleep(Duration::from_millis(150));
        assert!(Server::start(port, false).is_ok());
    }

    #[test]
    fn urls_and_pictures() {
        assert_eq!(url_decode("a%20b+c%zz%4"), "a b c%zz%4");
        let r = parse_request("POST /api/jog?dir=up%2Dleft&x HTTP/1.1\r\nHost: h\r\n\r\n").unwrap();
        assert_eq!((r.method.as_str(), r.path.as_str(), r.query[0].1.as_str()), ("POST", "/api/jog", "up-left"));
        assert!(encode_jpeg(2, 2, &[200u8; 12]).is_some_and(|j| j.starts_with(&[0xFF, 0xD8])));
        assert!(encode_jpeg(0, 2, &[]).is_none() && encode_jpeg(4, 4, &[0u8; 5]).is_none());
        for d in ["up", "down", "left", "right", "up-left", "up-right", "down-left", "down-right"] {
            assert!(Jog::parse(d).is_some());
        }
    }
}
