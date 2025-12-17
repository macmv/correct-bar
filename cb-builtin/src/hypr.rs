use parking_lot::Mutex;
use std::{
  cell::{Cell, RefCell},
  io::{BufRead, BufReader, Read, Write},
  os::unix::net::UnixStream,
  path::PathBuf,
  sync::Arc,
};

use cb_bar::{Animation, Module, TextLayout};
use cb_core::{Color, Waker};
use kurbo::Point;

use crate::{Dirty, UpdateGroup};

#[derive(Clone)]
pub struct Hypr {
  pub primary:   Color,
  pub secondary: Color,
}

struct HyprModule {
  spec:         Hypr,
  workspaces:   Vec<WorkspaceLayout>,
  dirty:        Dirty,
  render_dirty: Cell<bool>,
}

struct WorkspaceLayout {
  id:   u32,
  text: TextLayout,

  focus_animation: Animation,

  focused: bool,
  /// True if this workspace is on the focused monitor.
  active:  bool,
}

impl From<Hypr> for Box<dyn Module> {
  fn from(spec: Hypr) -> Self {
    Box::new(HyprModule {
      spec,
      workspaces: vec![],
      dirty: UPDATERS.lock().add(),
      render_dirty: Cell::new(false),
    })
  }
}

thread_local! {
  static SOCKET: RefCell<Option<Connection>> = RefCell::new(None);
}

struct Connection {
  request: PathBuf,
}

impl Connection {
  pub fn from_env() -> Self {
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
    let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap();

    Connection { request: format!("{runtime}/hypr/{sig}/.socket.sock").into() }
  }
}

static STATE: Mutex<HyprState> = Mutex::new(HyprState { monitors: vec![], workspaces: vec![] });
static UPDATERS: Mutex<UpdateGroup> = Mutex::new(UpdateGroup::new());

struct HyprState {
  monitors:   Vec<Monitor>,
  workspaces: Vec<Workspace>,
}

/// ```json
/// {
///   "id": 0,
///   "name": "DP-1",
///   "description": "Monitor Name",
///   "make": "LG",
///   "model": "Model",
///   "serial": "Serial number",
///   "width": 3840,
///   "height": 2160,
///   "physicalWidth": 610, // in mm
///   "physicalHeight": 350, // in mm
///   "refreshRate": 59.99700, // in Hz
///   "x": 0,
///   "y": 0,
///   "activeWorkspace": {
///     "id": 2,
///     "name": "2"
///   },
///   "specialWorkspace": {
///     "id": 0,
///     "name": ""
///   },
///   "reserved": [0, 30, 0, 0], // reserved space (ie, bars)
///   "scale": 2.00, // UI scale
///   "transform": 0,
///   "focused": true,
///   "dpmsStatus": true,
///   "vrr": false,
///   "solitary": "0",
///   "solitaryBlockedBy": ["WINDOWED", "CANDIDATE"],
///   "activelyTearing": false,
///   "tearingBlockedBy": ["NOT_TORN", "USER", "CANDIDATE"],
///   "directScanoutTo": "0",
///   "directScanoutBlockedBy": ["USER", "CANDIDATE"],
///   "disabled": false,
///   "currentFormat": "XRGB8888",
///   "mirrorOf": "none",
///   "availableModes": ["3840x2160@60.00Hz", "3840x2160@30.00Hz", etc],
///   "colorManagementPreset": "srgb",
///   "sdrBrightness": 1.00,
///   "sdrSaturation": 1.00,
///   "sdrMinLuminance": 0.20,
///   "sdrMaxLuminance": 80
/// }
/// ```
#[derive(serde::Deserialize)]
struct Monitor {
  name:    String,
  #[serde(rename = "activeWorkspace")]
  active:  ActiveWorkspace,
  focused: bool,
}

#[derive(serde::Deserialize)]
struct ActiveWorkspace {
  id: u32,
}

/// ```json
/// {
///   "id": 2,
///   "name": "2",
///   "monitor": "DP-1",
///   "monitorID": 0,
///   "windows": 1,
///   "hasfullscreen": false,
///   "lastwindow": "window id, like 0x55",
///   "lastwindowtitle": "title, like Steam",
///   "ispersistent": false
/// }
/// ```
#[derive(serde::Deserialize)]
struct Workspace {
  id:   u32,
  name: String,

  #[serde(skip)]
  focused: bool,
}

fn spawn_listener(waker: &Arc<Waker>) {
  use std::sync::atomic::*;

  static RUNNING: AtomicBool = AtomicBool::new(false);

  if !RUNNING.swap(true, Ordering::SeqCst) {
    STATE.lock().setup();
    let waker = waker.clone();
    std::thread::spawn(move || listen(waker));
  }
}

fn listen(waker: Arc<Waker>) {
  let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
  let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap();

  let mut reader =
    BufReader::new(UnixStream::connect(format!("{runtime}/hypr/{sig}/.socket2.sock")).unwrap());

  let mut line = String::new();
  loop {
    line.clear();
    match reader.read_line(&mut line) {
      Ok(0) => {
        eprintln!("hypr: connection closed");
        break;
      }
      Ok(_) => {}
      Err(e) => {
        eprintln!("hypr: {e}");
        break;
      }
    }

    let Some((ev, args)) = line.split_once(">>") else { continue };

    let ev = ev.trim();
    let args = args.trim();

    match ev {
      "workspacev2" => {
        let Some((workspace, _name)) = args.split_once(',') else { continue };
        let Ok(workspace) = workspace.parse::<u32>() else { continue };
        {
          let mut state = STATE.lock();
          state.focus_workspace(workspace);
        }
        UPDATERS.lock().mark_dirty();
        waker.wake();
      }
      "destroyworkspacev2" => {
        let Some((workspace, _name)) = args.split_once(',') else { continue };
        let Ok(workspace) = workspace.parse::<u32>() else { continue };
        {
          let mut state = STATE.lock();
          state.destroy_workspace(workspace);
        }
        UPDATERS.lock().mark_dirty();
        waker.wake();
      }
      "focusedmonv2" => {
        let Some((mon, workspace)) = args.split_once(',') else { continue };
        let Ok(workspace) = workspace.parse::<u32>() else { continue };
        {
          let mut state = STATE.lock();
          state.focus_monitor(mon);
          state.focus_workspace(workspace);
        }
        UPDATERS.lock().mark_dirty();
        waker.wake();
      }

      _ => {}
    }
  }
}

impl Connection {
  fn req_str(&self, req: &str) -> String {
    let mut stream = UnixStream::connect(&self.request).unwrap();

    stream.write_all(req.as_bytes()).unwrap();

    let mut buf = String::new();
    stream.read_to_string(&mut buf).unwrap();

    buf
  }

  fn req_json<T: serde::de::DeserializeOwned>(&self, req: &str) -> Vec<T> {
    serde_json::from_str(&self.req_str(&format!("j/{req}"))).unwrap()
  }

  pub fn dispatch(&self, req: &str) { self.req_str(&format!("dispatch {req}")); }

  pub fn load_monitors(&self) -> Vec<Monitor> { self.req_json("monitors") }
  pub fn load_workspaces(&self) -> Vec<Workspace> { self.req_json("workspaces") }
}

impl HyprState {
  fn setup(&mut self) {
    let c = Connection::from_env();

    self.monitors = c.load_monitors();
    self.workspaces = c.load_workspaces();

    self.workspaces.sort_by(|a, b| a.name.cmp(&b.name));
  }

  fn destroy_workspace(&mut self, id: u32) { self.workspaces.retain(|w| w.id != id); }

  fn focus_monitor(&mut self, name: &str) {
    for monitor in &mut self.monitors {
      monitor.focused = monitor.name == name;
    }
  }
  fn focus_workspace(&mut self, id: u32) {
    let mut found = false;
    for workspace in &mut self.workspaces {
      workspace.focused = workspace.id == id;
      found |= workspace.focused;
    }

    if !found {
      self.workspaces.clear();
      self.workspaces = Connection::from_env().load_workspaces();

      for workspace in &mut self.workspaces {
        workspace.focused = workspace.id == id;
      }
    }

    self.workspaces.sort_by(|a, b| a.name.cmp(&b.name));

    if let Some(focused) = self.monitors.iter_mut().find(|m| m.focused) {
      focused.active.id = id;
    }
  }
}

impl Module for HyprModule {
  fn updater(&self) -> cb_bar::Updater<'_> {
    if self.render_dirty.get() || self.workspaces.iter().any(|w| w.focus_animation.is_running()) {
      cb_bar::Updater::Animation
    } else {
      cb_bar::Updater::Atomic(self.dirty.get())
    }
  }

  fn on_mouse(&mut self, _: Point) { self.render_dirty.set(true); }

  fn layout(&mut self, layout: &mut cb_bar::Layout) {
    spawn_listener(layout.waker);
    self.dirty.clear();

    layout.pad(10.0);

    let state = STATE.lock();

    self.workspaces.retain(|w| state.workspaces.iter().find(|ws| ws.id == w.id).is_some());

    for (i, workspace) in state.workspaces.iter().enumerate() {
      if i != 0 {
        layout.pad(15.0);
      }

      if self.workspaces.get(i).is_none_or(|w| w.id != workspace.id) {
        self.workspaces.insert(
          i,
          WorkspaceLayout {
            id:              workspace.id,
            text:            TextLayout::empty(),
            focus_animation: Animation::ease_in(0.2),
            focused:         false,
            active:          false,
          },
        );
      }

      self.workspaces[i].text = layout.layout_text(&workspace.name, Color::BLACK);
      self.workspaces[i].focused = workspace.focused;
      self.workspaces[i].focus_animation.run(workspace.focused);
      self.workspaces[i].active =
        state.monitors.iter().find(|m| m.active.id == workspace.id).is_some();
    }

    layout.pad(10.0);
  }

  fn on_click(&mut self, cursor: Point) {
    for workspace in &self.workspaces {
      if workspace.text.bounds().inflate(5.0, 0.0).contains(cursor) {
        Connection::from_env().dispatch(&format!("workspace {}", workspace.id));
        STATE.lock().focus_workspace(workspace.id);
      }
    }
  }

  fn render(&self, ctx: &mut cb_core::Render) {
    self.render_dirty.set(false);

    for workspace in &self.workspaces {
      workspace.focus_animation.advance(ctx.frame_time());

      let target_color = if workspace.focused {
        self.spec.primary
      } else if workspace.active {
        cb_core::oklch(0.6, 0.18, 283.76)
      } else {
        self.spec.secondary
      };

      let color = if workspace.focus_animation.is_running() {
        target_color.lerp(
          self.spec.primary,
          workspace.focus_animation.interpolate(0.0, 1.0) as f32,
          peniko::color::HueDirection::Shorter,
        )
      } else {
        target_color
      };

      ctx.draw_button(&workspace.text.bounds().inflate(5.0, 0.0), color);
      ctx.draw_text_layout(workspace.text.origin, &workspace.text.layout, Some(color.into()));
    }
  }
}
