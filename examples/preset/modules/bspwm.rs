use correct_bar::bar::{Color, ModuleImpl, Updater};
use crossbeam_channel::Receiver;
use parking_lot::Mutex;
use std::{
  io::{prelude::*, BufReader},
  os::unix::net::UnixStream,
  sync::Arc,
  thread,
};

#[derive(Clone)]
pub struct BSPWM {
  path:    String,
  channel: Receiver<()>,
  state:   Arc<Mutex<json::WmState>>,
}

fn parse_hex(s: &str) -> u32 { u32::from_str_radix(&s[2..], 16).unwrap() }

impl BSPWM {
  pub fn new() -> Self { BSPWM::new_at("/tmp/bspwm_0_0-socket") }
  pub fn new_at(path: &str) -> Self {
    let state = send_immediate_json::<json::WmState>(path, &["wm", "-d"]).unwrap();
    let state = Arc::new(Mutex::new(state));

    let (tx, rx) = crossbeam_channel::bounded(16);
    let s = state.clone();
    let p = path.to_string();
    thread::spawn(move || {
      let mut socket = BufReader::new(send_blocking(&p, &["subscribe", "desktop"]));
      let mut line = String::new();
      loop {
        socket.read_line(&mut line).unwrap();
        let mut sections = line.trim().split(" ");
        match sections.next() {
          Some("desktop_focus") => {
            let monitor = parse_hex(sections.next().unwrap());
            let desktop = parse_hex(sections.next().unwrap());
            let mut state = s.lock();
            state.focused_monitor_id = monitor;
            state.monitor_mut(monitor).focused_desktop_id = desktop;
          }
          _ => {}
        }
        tx.send(()).unwrap();
        line.clear();
      }
    });
    BSPWM { path: path.into(), channel: rx, state }
  }
}

fn open_socket(path: &str) -> UnixStream { UnixStream::connect(path).unwrap() }
fn send_blocking(path: &str, args: &[&str]) -> UnixStream {
  let mut socket = open_socket(path);
  for arg in args {
    socket.write(arg.as_bytes()).unwrap();
    socket.write(&[0x00]).unwrap();
  }
  socket
}
fn send_immediate(path: &str, args: &[&str]) -> Result<String, String> {
  let mut buf = vec![];
  // try 10 times
  for _ in 0..10 {
    buf.clear();
    let mut socket = send_blocking(path, args);
    match socket.read_to_end(&mut buf) {
      Ok(_) => {
        if !buf.is_empty() && buf[0] == 0x07 {
          return Err(String::from_utf8(buf[1..].to_vec()).unwrap());
        } else {
          return Ok(String::from_utf8(buf).unwrap());
        }
      }
      Err(_) => continue,
    }
  }
  Err("could not connect to bspwm".into())
}

fn send_immediate_json<T: serde::de::DeserializeOwned>(
  path: &str,
  args: &[&str],
) -> Result<T, String> {
  send_immediate(path, args).map(|buf| serde_json::from_str(&buf).unwrap())
}

fn switch_desktop(path: &str, desktop: u32) {
  send_immediate(path, &["desktop", "-f", &desktop.to_string()]).unwrap();
}

impl ModuleImpl for BSPWM {
  fn updater(&self) -> Updater { Updater::Channel(self.channel.clone()) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    let state = self.state.lock();
    let needs_click_regions = ctx.needs_click_regions();
    let mut i = 0;
    for monitor in &state.monitors {
      for desktop in &monitor.desktops {
        if i != 0 {
          ctx.advance_text(" ");
        }
        let is_focused = desktop.id == monitor.focused_desktop_id;
        let rect = ctx.draw_text(
          &desktop.name,
          if is_focused { Color::from_hex(0x00ffff) } else { Color::white() },
        );
        if needs_click_regions {
          let p = self.path.clone();
          let d = desktop.id;
          ctx.add_on_click(rect, move || switch_desktop(&p, d));
        }
        i += 1;
      }
    }
  }
}

#[allow(unused)]
mod json {
  use serde::Deserialize;

  #[derive(Clone, Debug, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct WmState {
    pub focused_monitor_id: u32,
    pub clients_count:      u32,
    pub monitors:           Vec<Monitor>,
    // There's a couple other fields, but I don't care about them (things like switch history) so
    // I'm going to leave them out for now.
  }

  impl WmState {
    pub fn monitor(&self, id: u32) -> &Monitor {
      for monitor in &self.monitors {
        if monitor.id == id {
          return monitor;
        }
      }
      panic!("no monitor with id {id:#x}");
    }
    pub fn monitor_mut(&mut self, id: u32) -> &mut Monitor {
      for monitor in &mut self.monitors {
        if monitor.id == id {
          return monitor;
        }
      }
      panic!("no monitor with id {id:#x}");
    }
  }

  #[derive(Clone, Debug, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Monitor {
    pub name:               String,
    pub id:                 u32,
    pub randr_id:           u32,
    pub wired:              bool,
    pub sticky_count:       u32,
    pub window_gap:         u32,
    pub border_width:       u32,
    pub focused_desktop_id: u32,
    pub padding:            Padding,
    pub rectangle:          Rectangle,
    pub desktops:           Vec<Desktop>,
  }

  #[derive(Clone, Debug, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Desktop {
    pub name:            String,
    pub id:              u32,
    pub layout:          Layout,
    pub user_layout:     Layout,
    pub window_gap:      u32,
    pub border_width:    u32,
    pub focused_node_id: u32,
    pub padding:         Padding,
    pub root:            Option<Node>,
  }

  #[derive(Clone, Debug, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Node {
    pub id:           u32,
    pub split_type:   SplitType,
    pub split_ratio:  f64,
    pub vacant:       bool,
    pub hidden:       bool,
    pub sticky:       bool,
    pub private:      bool,
    pub locked:       bool,
    pub marked:       bool,
    pub rectangle:    Rectangle,
    pub constraints:  Constraints,
    pub first_child:  Option<Box<Node>>,
    pub second_child: Option<Box<Node>>,
    pub client:       Option<Window>,
  }

  #[derive(Clone, Debug, Deserialize)]
  #[serde(rename_all = "snake_case")]
  pub struct Constraints {
    pub min_width:  u32,
    pub min_height: u32,
  }

  #[derive(Clone, Debug, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Window {
    pub class_name:         String,
    pub instance_name:      String,
    pub border_width:       u32,
    pub state:              WindowState,
    pub last_state:         WindowState,
    pub layer:              Layer,
    pub last_layer:         Layer,
    pub urgent:             bool,
    pub shown:              bool,
    pub tiled_rectangle:    Rectangle,
    pub floating_rectangle: Rectangle,
  }

  #[derive(Clone, Debug, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub enum Layout {
    Tiled,
  }

  #[derive(Clone, Debug, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub enum WindowState {
    Tiled,
    Floating,
  }

  #[derive(Clone, Debug, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub enum Layer {
    Normal,
  }

  #[derive(Clone, Debug, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub enum SplitType {
    Vertical,
    Horizontal,
  }

  #[derive(Clone, Debug, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Padding {
    pub top:    u32,
    pub right:  u32,
    pub bottom: u32,
    pub left:   u32,
  }

  #[derive(Clone, Debug, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Rectangle {
    pub x:      i32,
    pub y:      i32,
    pub width:  u32,
    pub height: u32,
  }
}
