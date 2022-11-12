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
  fn send_immediate(&self, args: &[&str]) -> Result<String, String> {
    let mut socket = self.send_blocking(args);
    let mut buf = vec![];
    socket.read_to_end(&mut buf).unwrap();
    if buf[0] == 0x07 {
      Err(String::from_utf8(buf[1..].to_vec()).unwrap())
    } else {
      Ok(String::from_utf8(buf).unwrap())
    }
  }
}

impl ModuleImpl for BSPWMModule {
  fn updater(&self) -> Updater { Updater::Channel(self.channel.as_ref().unwrap().clone()) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    /*
    {
      let res = self.bspwm.send_immediate(&["query", "-T", "-d"]);
      println!("{res:?}");
    }
    */
    ctx.draw_text("hello", Color::white());
  }
}
