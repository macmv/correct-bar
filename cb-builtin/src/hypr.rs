use parking_lot::Mutex;
use std::{
  cell::{Cell, RefCell},
  io::{BufRead, BufReader, Read, Write},
  os::unix::net::UnixStream,
  path::PathBuf,
  sync::Arc,
};

use cb_bar::{Module, TextLayout};
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
  id:      String,
  text:    TextLayout,
  focused: bool,
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

static STATE: Mutex<HyprState> = Mutex::new(HyprState { workspaces: vec![] });
static UPDATERS: Mutex<UpdateGroup> = Mutex::new(UpdateGroup::new());

#[derive(Clone)]
struct HyprState {
  workspaces: Vec<Workspace>,
}

#[derive(Clone)]
struct Workspace {
  id:      String,
  name:    String,
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
        STATE.lock().focus_workspace(workspace);
        UPDATERS.lock().mark_dirty();
        waker.wake();
      }
      "destroyworkspacev2" => {
        let Some((workspace, _name)) = args.split_once(',') else { continue };
        STATE.lock().destroy_workspace(workspace);
        UPDATERS.lock().mark_dirty();
        waker.wake();
      }
      "focusedmonv2" => {
        let Some((_mon, workspace)) = args.split_once(',') else { continue };
        STATE.lock().focus_workspace(workspace);
        UPDATERS.lock().mark_dirty();
        waker.wake();
      }

      _ => {}
    }
  }
}

impl Connection {
  fn req(&self, req: &str) -> String {
    let mut stream = UnixStream::connect(&self.request).unwrap();

    stream.write_all(req.as_bytes()).unwrap();

    let mut buf = String::new();
    stream.read_to_string(&mut buf).unwrap();

    buf
  }

  fn load_workspaces(&self) -> Vec<Workspace> {
    let mut workspaces = vec![];

    for line in self.req("workspaces").lines() {
      // TODO: Parse json
      if let Some(suffix) = line.strip_prefix("workspace ID ") {
        let (id, _) = suffix.split_once(' ').unwrap();
        workspaces.push(Workspace {
          id:      id.to_string(),
          name:    id.to_string(),
          focused: false,
        });
      }
    }

    workspaces
  }
}

impl HyprState {
  fn setup(&mut self) {
    let c = Connection::from_env();

    for workspace in c.load_workspaces() {
      self.workspaces.push(workspace);
    }

    self.workspaces.sort_by(|a, b| a.name.cmp(&b.name));
  }

  fn destroy_workspace(&mut self, id: &str) { self.workspaces.retain(|w| w.id != id); }

  fn focus_workspace(&mut self, id: &str) {
    let mut found = false;
    for workspace in &mut self.workspaces {
      workspace.focused = workspace.id == id;
      found |= workspace.focused;
    }

    if !found {
      self.workspaces.push(Workspace {
        id:      id.to_string(),
        name:    id.to_string(),
        focused: true,
      });
    }

    self.workspaces.sort_by(|a, b| a.name.cmp(&b.name));
  }
}

impl Module for HyprModule {
  fn updater(&self) -> cb_bar::Updater<'_> {
    if self.render_dirty.get() {
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

    let state = STATE.lock().clone();

    self.workspaces.clear();
    for (i, workspace) in state.workspaces.iter().enumerate() {
      if i != 0 {
        layout.pad(15.0);
      }

      let color = if workspace.focused { self.spec.primary } else { self.spec.secondary };
      self.workspaces.push(WorkspaceLayout {
        id:      workspace.id.clone(),
        text:    layout.layout_text(&workspace.name, color),
        focused: workspace.focused,
      });
    }

    layout.pad(10.0);
  }

  fn on_click(&mut self, cursor: Point) {
    for workspace in &self.workspaces {
      if workspace.text.bounds().inflate(5.0, 0.0).contains(cursor) {
        Connection::from_env().req(&format!("dispatch workspace {}", workspace.id));
        STATE.lock().focus_workspace(&workspace.id);
      }
    }
  }

  fn render(&self, ctx: &mut cb_core::Render) {
    self.render_dirty.set(false);

    for workspace in &self.workspaces {
      ctx.draw_button(
        &workspace.text.bounds().inflate(5.0, 0.0),
        if workspace.focused { self.spec.primary } else { self.spec.secondary },
      );
      ctx.draw(&workspace.text);
    }
  }
}
