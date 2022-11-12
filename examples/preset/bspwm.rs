use correct_bar::bar::{Color, ModuleImpl, Updater};
use std::{io::prelude::*, os::unix::net::UnixStream};

pub struct BSPWM {
  path: String,
}

impl BSPWM {
  pub fn new() -> Self { BSPWM::new_at("/tmp/bspwm_0_0-socket") }
  pub fn new_at(path: &str) -> Self { BSPWM { path: path.into() } }

  fn open_socket(&self) -> UnixStream { UnixStream::connect(&self.path).unwrap() }
  fn send_immediate(&self, args: &[&str]) -> Result<String, String> {
    let mut socket = self.open_socket();
    for arg in args {
      socket.write(arg.as_bytes()).unwrap();
      socket.write(&[0x00]).unwrap();
    }
    let mut buf = vec![];
    socket.read_to_end(&mut buf).unwrap();
    if buf[0] == 0x07 {
      Err(String::from_utf8(buf[1..].to_vec()).unwrap())
    } else {
      Ok(String::from_utf8(buf).unwrap())
    }
  }
}

impl ModuleImpl for BSPWM {
  fn updater(&self) -> Updater { Updater::Never }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    {
      let res = self.send_immediate(&["query", "-T", "-d"]);
      println!("{res:?}");
    }
    ctx.draw_text("hello", Color::white());
  }
}
