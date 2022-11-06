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
  wm_protocols:  b"WM_PROTOCOLS",
  wm_del_window: b"WM_DELETE_WINDOW",
  wm_state:      b"_NET_WM_STATE",
  wm_state_maxv: b"_NET_WM_STATE_MAXIMIZED_VERT",
  wm_state_maxh: b"_NET_WM_STATE_MAXIMIZED_HORZ",
}

pub fn run() {
  match run_inner() {
    Ok(()) => {}
    Err(e) => println!("{e}"),
  }
}

fn run_inner() -> xcb::Result<()> {
  let (conn, screen_num) = xcb::Connection::connect(None)?;

  let setup = conn.get_setup();
  let screen = setup.roots().nth(screen_num as usize).unwrap();

  let window = conn.generate_id();

  let cookie = conn.send_request_checked(&x::CreateWindow {
    depth:        x::COPY_FROM_PARENT as u8,
    wid:          window,
    parent:       screen.root(),
    x:            0,
    y:            0,
    width:        150,
    height:       150,
    border_width: 0,
    class:        x::WindowClass::InputOutput,
    visual:       screen.root_visual(),
    // this list must be in same order than `Cw` enum order
    value_list:   &[
      x::Cw::BackPixel(screen.white_pixel()),
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
    data: b"My XCB Window",
  });
  conn.check_request(cookie)?;

  conn.send_request(&x::MapWindow { window });

  let atoms = Atoms::setup(&conn)?;

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

  let mut maximized = false;

  // We enter the main event loop
  loop {
    match conn.wait_for_event()? {
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
            destination: x::SendEventDest::Window(screen.root()),
            event_mask:  x::EventMask::STRUCTURE_NOTIFY,
            event:       &event,
          });
          conn.check_request(cookie)?;

          // Same as before, if we don't check for error, we have to flush
          // the connection.
          // conn.flush()?;

          maximized = !maximized;
        } else if ev.detail() == 0x18 {
          // Q (on qwerty)

          // We exit the event loop (and the program)
          break Ok(());
        }
      }
      xcb::Event::X(x::Event::ClientMessage(ev)) => {
        // We have received a message from the server
        if let x::ClientMessageData::Data32([atom, ..]) = ev.data() {
          if atom == atoms.wm_del_window.resource_id() {
            break Ok(());
          }
        }
      }
      _ => {}
    }
  }
}
