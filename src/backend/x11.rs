use crate::{
  bar::{Backend, Bar, Window},
  config::Config,
};
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc, thread};
use xcb::{x, Xid};

macro_rules! atoms {
  (
    $(
      $ident:ident: $name:expr,
    )*
  ) => {
    struct Atoms {
      $( $ident: x::Atom, )*
    }

    impl Atoms {
      pub fn setup(conn: &xcb::Connection) -> xcb::Result<Self> {
        $(
          let $ident = conn.send_request(&x::InternAtom { only_if_exists: true, name: $name });
        )*
        Ok(Atoms {
          $( $ident: conn.wait_for_reply($ident)?.atom(), )*
        })
      }
    }
  }
}

atoms! {
  wm_del_window:       b"WM_DELETE_WINDOW",
  wm_state:            b"_NET_WM_STATE",
  wm_state_above:      b"_NET_WM_STATE_ABOVE",
  wm_state_sticky:     b"_NET_WM_STATE_STICKY",
  wm_strut:            b"_NET_WM_STRUT",
  wm_strut_partial:    b"_NET_WM_STRUT_PARTIAL",
  wm_window_type:      b"_NET_WM_WINDOW_TYPE",
  wm_window_type_dock: b"_NET_WM_WINDOW_TYPE_DOCK",
}

#[allow(unused)]
enum Strut {
  Left,
  Right,
  Top,
  Bottom,
  LeftStartY,
  LeftEndY,
  RightStartY,
  RightEndY,
  TopStartX,
  TopEndX,
  BottomStartX,
  BottomEndX,
}

pub struct X11Backend {
  conn:   Arc<xcb::Connection>,
  window: xcb::x::Window,
  pixmap: xcb::x::Pixmap,
  gc:     xcb::x::Gcontext,
  depth:  u8,
}

impl X11Backend {
  pub fn send_check(&self, req: &impl xcb::RequestWithoutReply) -> xcb::Result<()> {
    self.check(self.send(req))
  }
  pub fn send(&self, req: &impl xcb::RequestWithoutReply) -> xcb::VoidCookieChecked {
    self.conn.send_request_checked(req)
  }
  pub fn check(&self, cookie: xcb::VoidCookieChecked) -> xcb::Result<()> {
    self.conn.check_request(cookie)?;
    Ok(())
  }
}

impl Backend for X11Backend {
  fn render(&self, window: &Window) {
    self
      .send_check(&xcb::x::PutImage {
        data:     window.data(),
        gc:       self.gc,
        drawable: x::Drawable::Pixmap(self.pixmap),
        depth:    self.depth,
        width:    window.width() as u16,
        height:   window.height() as u16,
        dst_x:    0,
        dst_y:    0,
        format:   xcb::x::ImageFormat::ZPixmap,
        left_pad: 0,
      })
      .unwrap();

    self
      .send_check(&xcb::x::CopyArea {
        dst_drawable: x::Drawable::Window(self.window),
        dst_x:        0,
        dst_y:        0,
        gc:           self.gc,
        src_drawable: x::Drawable::Pixmap(self.pixmap),
        src_x:        0,
        src_y:        0,
        width:        window.width() as u16,
        height:       window.height() as u16,
      })
      .unwrap();
  }
}

pub fn setup(config: Config) -> Vec<Arc<Mutex<Bar>>> {
  match setup_inner(config) {
    Ok(bar) => bar,
    Err(e) => {
      println!("{e}");
      std::process::exit(1);
    }
  }
}

fn root_windows(conn: &xcb::Connection, screen: &xcb::x::Screen) -> xcb::Result<Vec<x::Window>> {
  let tree = conn.wait_for_reply(conn.send_request(&x::QueryTree { window: screen.root() }))?;

  let mut roots = vec![];
  for child in tree.children() {
    let atom = conn.wait_for_reply(conn.send_request(&x::GetProperty {
      delete:      false,
      window:      *child,
      property:    x::ATOM_WM_CLASS,
      r#type:      x::ATOM_STRING,
      long_offset: 0,
      long_length: 4,
    }))?;
    let classes = std::str::from_utf8(atom.value()).unwrap();
    if classes.split("\0").any(|class| class == "Bspwm") {
      roots.push(*child);
    }
  }

  Ok(roots)
}

fn setup_window(
  atoms: &Atoms,
  conn: &Arc<xcb::Connection>,
  config: Config,
  screen: &x::Screen,
  root_window: x::Window,
  x: i16,
  y: i16,
  width: u16,
) -> xcb::Result<(x::Window, Bar)> {
  let window = conn.generate_id();
  let depth = screen.root_depth();

  conn.check_request(conn.send_request_checked(&x::CreateWindow {
    depth:        x::COPY_FROM_PARENT as u8,
    wid:          window,
    parent:       screen.root(),
    x:            x + config.window.margin_left as i16,
    y:            x + config.window.margin_top as i16,
    width:        width as u16,
    height:       config.window.height as u16,
    border_width: 0,
    class:        x::WindowClass::InputOutput,
    visual:       screen.root_visual(),
    // this list must be in same order than `Cw` enum order
    value_list:   &[
      x::Cw::BackPixel(0x222222),
      x::Cw::EventMask(x::EventMask::EXPOSURE | x::EventMask::BUTTON_PRESS),
    ],
  }))?;

  conn.check_request(conn.send_request_checked(&x::ConfigureWindow {
    window,
    value_list: &[
      x::ConfigWindow::Sibling(root_window),
      x::ConfigWindow::StackMode(x::StackMode::Above),
    ],
  }))?;

  conn.check_request(conn.send_request_checked(&x::ChangeProperty {
    mode: x::PropMode::Replace,
    window,
    property: x::ATOM_WM_NAME,
    r#type: x::ATOM_STRING,
    data: b"Correct Bar",
  }))?;

  // Need to send these requests before mapping the window!

  conn.check_request(conn.send_request_checked(&x::ChangeProperty {
    mode: x::PropMode::Replace,
    window,
    property: atoms.wm_window_type,
    r#type: x::ATOM_ATOM,
    data: &[atoms.wm_window_type_dock],
  }))?;

  let cookie = conn.send_request_checked(&x::ChangeProperty {
    mode: x::PropMode::Append,
    window,
    property: atoms.wm_state,
    r#type: x::ATOM_ATOM,
    data: &[atoms.wm_state_above, atoms.wm_state_sticky],
  });
  conn.check_request(cookie)?;

  let mut strut = [0_u32; 12];
  strut[Strut::Top as usize] =
    config.window.margin_top + config.window.height + config.window.margin_bottom;
  strut[Strut::TopStartX as usize] = config.window.margin_left;
  strut[Strut::TopEndX as usize] = width as u32 - config.window.margin_right;

  let cookie = conn.send_request_checked(&x::ChangeProperty {
    mode: x::PropMode::Replace,
    window,
    property: atoms.wm_strut,
    r#type: x::ATOM_CARDINAL,
    data: &strut[..4],
  });
  conn.check_request(cookie)?;

  let cookie = conn.send_request_checked(&x::ChangeProperty {
    mode: x::PropMode::Replace,
    window,
    property: atoms.wm_strut_partial,
    r#type: x::ATOM_CARDINAL,
    data: &strut,
  });
  conn.check_request(cookie)?;

  conn.send_request(&x::MapWindow { window });

  // Previous request was checked, so a flush is not necessary in this case.
  // Otherwise, here is how to perform a connection flush.
  conn.flush()?;

  let pixmap = conn.generate_id();
  conn.check_request(conn.send_request_checked(&x::CreatePixmap {
    drawable: x::Drawable::Window(window),
    pid: pixmap,
    height: config.window.height as u16,
    width: width as u16,
    depth,
  }))?;

  let gc = conn.generate_id();
  conn.check_request(conn.send_request_checked(&x::CreateGc {
    drawable:   x::Drawable::Pixmap(pixmap),
    cid:        gc,
    value_list: &[],
  }))?;

  assert_eq!(depth, 24);

  let height = config.window.height;
  Ok((
    window,
    Bar::from_config(
      config,
      width.into(),
      height,
      X11Backend { conn: conn.clone(), window, pixmap, depth, gc },
    ),
  ))
}

fn setup_inner(config: Config) -> xcb::Result<Vec<Arc<Mutex<Bar>>>> {
  let (conn, screen_num) = xcb::Connection::connect(None)?;

  let conn = Arc::new(conn);

  let setup = conn.get_setup();
  let screen = setup.roots().nth(screen_num as usize).unwrap().clone();

  let atoms = Atoms::setup(&conn)?;

  // We setup a bar for every root. Sometimes we get a fake geom here, which isn't
  // really a monitor, so we make sure it's real by checking that it's X/Y > 0.
  let mut bars = vec![];
  let mut bars_map = HashMap::new();
  for root in root_windows(&conn, screen)? {
    let geom = conn
      .wait_for_reply(conn.send_request(&x::GetGeometry { drawable: x::Drawable::Window(root) }))?;

    if geom.x() < 0 || geom.y() < 0 {
      continue;
    }

    let mut config = config.clone();
    config.apply_scaling_for_width(geom.width().into());
    let (window, bar) =
      setup_window(&atoms, &conn, config, screen, root, geom.x(), geom.y(), geom.width())?;
    let bar = Arc::new(Mutex::new(bar));

    bars_map.insert(window.resource_id(), bar.clone());
    bars.push(bar);
  }

  // We enter the main event loop
  thread::spawn(move || {
    loop {
      match conn.wait_for_event().unwrap() {
        xcb::Event::X(x::Event::Expose(ev)) => {
          let bar = &bars_map[&ev.window().resource_id()];
          bar.lock().render()
        }
        xcb::Event::X(x::Event::ButtonPress(ev)) => {
          let bar = &bars_map[&ev.child().resource_id()];
          bar.lock().click(ev.event_x().try_into().unwrap(), ev.event_y().try_into().unwrap())
        }
        xcb::Event::X(x::Event::ClientMessage(ev)) => {
          // We have received a message from the server
          if let x::ClientMessageData::Data32([atom, ..]) = ev.data() {
            if atom == atoms.wm_del_window.resource_id() {
              break;
            }
          }
        }
        _ => {}
      }
    }
  });
  Ok(bars)
}
