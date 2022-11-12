use crate::{
  bar::{Backend, Bar, Window},
  config::Config,
};
use parking_lot::Mutex;
use std::{sync::Arc, thread};
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
  wm_protocols:        b"WM_PROTOCOLS",
  wm_del_window:       b"WM_DELETE_WINDOW",
  wm_state:            b"_NET_WM_STATE",
  wm_state_above:      b"_NET_WM_STATE_ABOVE",
  wm_state_maxv:       b"_NET_WM_STATE_MAXIMIZED_VERT",
  wm_state_maxh:       b"_NET_WM_STATE_MAXIMIZED_HORZ",
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

pub fn setup(config: &Config) -> Arc<Mutex<Bar>> {
  match setup_inner(config) {
    Ok(bar) => bar,
    Err(e) => {
      println!("{e}");
      std::process::exit(1);
    }
  }
}

fn setup_inner(config: &Config) -> xcb::Result<Arc<Mutex<Bar>>> {
  let (conn, screen_num) = xcb::Connection::connect(None)?;

  let setup = conn.get_setup();
  let screen = setup.roots().nth(screen_num as usize).unwrap().clone();
  let root = screen.root();
  let depth = screen.root_depth();

  let window = conn.generate_id();

  let cookie = conn.send_request_checked(&x::CreateWindow {
    depth:        x::COPY_FROM_PARENT as u8,
    wid:          window,
    parent:       screen.root(),
    x:            config.window.margin_left as i16,
    y:            config.window.margin_top as i16,
    width:        config.window.width as u16,
    height:       config.window.height as u16,
    border_width: 0,
    class:        x::WindowClass::InputOutput,
    visual:       screen.root_visual(),
    // this list must be in same order than `Cw` enum order
    value_list:   &[
      x::Cw::BackPixel(0x222222),
      x::Cw::EventMask(x::EventMask::EXPOSURE | x::EventMask::KEY_PRESS),
    ],
  });
  // We now check if the window creation worked.
  // A cookie can't be cloned; it is moved to the function.
  conn.check_request(cookie)?;

  // Let's change the window title
  let cookie = conn.send_request_checked(&x::ChangeProperty {
    mode: x::PropMode::Replace,
    window,
    property: x::ATOM_WM_NAME,
    r#type: x::ATOM_STRING,
    data: b"Correct Bar",
  });
  conn.check_request(cookie)?;

  let atoms = Atoms::setup(&conn)?;

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
  strut[Strut::TopEndX as usize] = config.window.width - config.window.margin_right;

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

  // We now activate the window close event by sending the following request.
  // If we don't do this we can still close the window by clicking on the "x"
  // button, but the event loop is notified through a connection shutdown error.
  conn.check_request(conn.send_request_checked(&x::ChangeProperty {
    mode: x::PropMode::Replace,
    window,
    property: atoms.wm_protocols,
    r#type: x::ATOM_ATOM,
    data: &[atoms.wm_del_window],
  }))?;

  // Previous request was checked, so a flush is not necessary in this case.
  // Otherwise, here is how to perform a connection flush.
  conn.flush()?;

  let pixmap = conn.generate_id();
  conn.check_request(conn.send_request_checked(&x::CreatePixmap {
    drawable: x::Drawable::Window(window),
    pid: pixmap,
    height: config.window.height as u16,
    width: config.window.width as u16,
    depth,
  }))?;

  let gc = conn.generate_id();
  conn.check_request(conn.send_request_checked(&x::CreateGc {
    drawable:   x::Drawable::Pixmap(pixmap),
    cid:        gc,
    value_list: &[],
  }))?;

  assert_eq!(depth, 24);

  let mut maximized = false;

  let conn = Arc::new(conn);
  let bar = Arc::new(Mutex::new(Bar::from_config(
    &config,
    X11Backend { conn: conn.clone(), window, pixmap, depth, gc },
  )));

  // We enter the main event loop
  let b2 = bar.clone();
  thread::spawn(move || {
    loop {
      match conn.wait_for_event().unwrap() {
        xcb::Event::X(x::Event::Expose(_)) => {
          b2.lock().render();
          println!("Got an expose!");
        }
        xcb::Event::X(x::Event::KeyPress(ev)) => {
          if ev.detail() == 0x3a {
            // The M key was pressed
            // (M only on qwerty keyboards. Keymap support is done
            // with the `xkb` extension and the `xkbcommon-rs` crate)

            // We toggle maximized state, for this we send a message
            // by building a `x::ClientMessageEvent` with the proper
            // atoms and send it to the server.

            let data = x::ClientMessageData::Data32([
              if maximized { 0 } else { 1 },
              atoms.wm_state_maxv.resource_id(),
              atoms.wm_state_maxh.resource_id(),
              0,
              0,
            ]);
            let event = x::ClientMessageEvent::new(window, atoms.wm_state, data);
            let cookie = conn.send_request_checked(&x::SendEvent {
              propagate:   false,
              destination: x::SendEventDest::Window(root),
              event_mask:  x::EventMask::STRUCTURE_NOTIFY,
              event:       &event,
            });
            conn.check_request(cookie).unwrap();

            // Same as before, if we don't check for error, we have to flush
            // the connection.
            // conn.flush()?;

            maximized = !maximized;
          } else if ev.detail() == 0x18 {
            // Q (on qwerty)

            // We exit the event loop (and the program)
            break;
          }
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
  Ok(bar)
}
