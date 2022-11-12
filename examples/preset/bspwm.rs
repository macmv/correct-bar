use correct_bar::bar::{Color, ModuleImpl, Updater};
use crossbeam_channel::Receiver;
use std::{
  io::{prelude::*, BufReader},
  os::unix::net::UnixStream,
  thread,
};

#[derive(Clone)]
pub struct BSPWMModule {
  bspwm:   BSPWM,
  channel: Receiver<()>,
}

#[derive(Clone)]
struct BSPWM {
  path: String,
}

impl BSPWMModule {
  pub fn new() -> Self { BSPWMModule::new_at("/tmp/bspwm_0_0-socket") }
  pub fn new_at(path: &str) -> Self {
    let bspwm = BSPWM { path: path.into() };

    let (tx, rx) = crossbeam_channel::bounded(16);
    let b = bspwm.clone();
    thread::spawn(move || {
      let mut socket = BufReader::new(b.send_blocking(&["subscribe", "desktop"]));
      let mut line = String::new();
      loop {
        socket.read_line(&mut line).unwrap();
        println!("{line}");
        tx.send(()).unwrap();
        line.clear();
      }
    });
    BSPWMModule { bspwm, channel: rx }
  }
}

impl BSPWM {
  fn open_socket(&self) -> UnixStream { UnixStream::connect(&self.path).unwrap() }
  fn send_blocking(&self, args: &[&str]) -> UnixStream {
    let mut socket = self.open_socket();
    for arg in args {
      socket.write(arg.as_bytes()).unwrap();
      socket.write(&[0x00]).unwrap();
    }
    socket
  }
  fn send_immediate<T: serde::de::DeserializeOwned>(&self, args: &[&str]) -> Result<T, String> {
    let mut socket = self.send_blocking(args);
    let mut buf = vec![];
    socket.read_to_end(&mut buf).unwrap();
    if buf[0] == 0x07 {
      Err(String::from_utf8(buf[1..].to_vec()).unwrap())
    } else {
      Ok(serde_json::from_str(std::str::from_utf8(&buf).unwrap()).unwrap())
    }
  }
}

impl ModuleImpl for BSPWMModule {
  fn updater(&self) -> Updater { Updater::Channel(self.channel.clone()) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    let state = self.bspwm.send_immediate::<json::WmState>(&["wm", "-d"]).unwrap();
    let mut i = 0;
    for monitor in &state.monitors {
      for desktop in &monitor.desktops {
        if i != 0 {
          ctx.advance_text(" ");
        }
        let is_focused = desktop.id == monitor.focused_desktop_id;
        ctx.draw_text(
          &desktop.name,
          if is_focused { Color::from_hex(0x00ffff) } else { Color::white() },
        );
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
