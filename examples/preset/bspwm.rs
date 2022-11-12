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
  channel: Option<Receiver<()>>,
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
    BSPWMModule { bspwm, channel: Some(rx) }
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
  fn updater(&self) -> Updater { Updater::Channel(self.channel.as_ref().unwrap().clone()) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    {
      let res = self.bspwm.send_immediate::<json::Desktop>(&["query", "-T", "-d"]);
      println!("{res:?}");
    }
    ctx.draw_text("hello", Color::white());
  }
}

#[allow(unused)]
mod json {
  use serde::Deserialize;

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
    id:           u32,
    split_type:   SplitType,
    split_ratio:  f64,
    vacant:       bool,
    hidden:       bool,
    sticky:       bool,
    private:      bool,
    locked:       bool,
    marked:       bool,
    rectangle:    Rectangle,
    constraints:  Constraints,
    first_child:  Option<Box<Node>>,
    second_child: Option<Box<Node>>,
    client:       Option<Window>,
  }

  #[derive(Clone, Debug, Deserialize)]
  #[serde(rename_all = "snake_case")]
  pub struct Constraints {
    min_width:  u32,
    min_height: u32,
  }

  #[derive(Clone, Debug, Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct Window {
    class_name:         String,
    instance_name:      String,
    border_width:       u32,
    state:              WindowState,
    last_state:         WindowState,
    layer:              Layer,
    last_layer:         Layer,
    urgent:             bool,
    shown:              bool,
    tiled_rectangle:    Rectangle,
    floating_rectangle: Rectangle,
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
