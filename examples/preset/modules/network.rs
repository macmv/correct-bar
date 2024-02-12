use correct_bar::bar::{Color, ModuleImpl, Updater};
use crossbeam_channel::{Receiver, Sender};
use dbus::{
  blocking::{stdintf::org_freedesktop_dbus::Properties, Connection},
  channel::MatchingReceiver,
  message::MatchRule,
};
use networkmanager::{
  devices::{Any, Wired},
  NetworkManager,
};
use parking_lot::Mutex;
use std::{sync::Arc, time::Duration};

type Result<T> = std::result::Result<T, dbus::Error>;

#[derive(Clone)]
pub struct Network {
  pub primary:   Color,
  pub secondary: Color,

  state: Arc<Mutex<NetworkState>>,
  recv:  Receiver<()>,
}

struct NetworkState {
  dbus: Connection,
}

impl Network {
  pub fn new(primary: Color, secondary: Color) -> Network {
    Self::new_inner(primary, secondary).unwrap()
  }

  fn new_inner(primary: Color, secondary: Color) -> Result<Network> {
    let state = NetworkState::new_system();

    let (tx, rx) = crossbeam_channel::bounded(0);

    std::thread::spawn(move || {
      let state = NetworkState::new_session();
      state.subscribe_to_events(tx);
    });

    Ok(Network { primary, secondary, recv: rx, state: Arc::new(Mutex::new(state)) })
  }
}

impl ModuleImpl for Network {
  fn updater(&self) -> Updater { Updater::Channel(self.recv.clone()) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    let s = self.state.lock();

    for (i, c) in s.active_connections().unwrap().iter().enumerate() {
      if i != 0 {
        ctx.draw_text(", ", self.secondary);
      }
      ctx.draw_text(&c, self.primary);
    }
  }
  fn box_clone(&self) -> Box<dyn ModuleImpl + Send + Sync> { Box::new(self.clone()) }
}

const TIMEOUT: Duration = Duration::from_secs(5);

impl NetworkState {
  pub fn new_session() -> NetworkState { NetworkState { dbus: Connection::new_session().unwrap() } }
  pub fn new_system() -> NetworkState { NetworkState { dbus: Connection::new_system().unwrap() } }

  pub fn subscribe_to_events(&self, tx: Sender<()>) {
    let mut rule = MatchRule::new();
    rule.msg_type = Some(dbus::MessageType::Signal);
    rule.path = Some("/org/freedesktop/portal/desktop".into());

    let proxy = self.dbus.with_proxy("org.freedesktop.DBus", "/org/freedesktop/DBus", TIMEOUT);
    let _: () = proxy
      .method_call(
        "org.freedesktop.DBus.Monitoring",
        "BecomeMonitor",
        (vec![rule.match_str()], 0u32),
      )
      .unwrap();

    self.dbus.start_receive(
      rule,
      Box::new(move |msg, _| {
        if msg.interface().as_deref() == Some("org.freedesktop.portal.NetworkMonitor") {
          tx.send(()).unwrap();
        }
        true
      }),
    );

    loop {
      self.dbus.process(Duration::from_secs(1)).unwrap();
    }
  }

  pub fn active_connections(&self) -> Result<Vec<String>> {
    let proxy = self.dbus.with_proxy(
      "org.freedesktop.NetworkManager",
      "/org/freedesktop/NetworkManager",
      TIMEOUT,
    );

    let (devices,): (Vec<dbus::Path>,) =
      proxy.method_call("org.freedesktop.NetworkManager", "GetDevices", ())?;

    let mut connections = vec![];

    for dev in devices {
      let proxy = self.dbus.with_proxy("org.freedesktop.NetworkManager", &dev, TIMEOUT);

      let active =
        proxy.get::<dbus::Path>("org.freedesktop.NetworkManager.Device", "ActiveConnection")?;

      let connection = self.dbus.with_proxy("org.freedesktop.NetworkManager", &active, TIMEOUT);

      // The path `active` won't exist if there is no active connection, and this'll
      // return an error.
      let Ok(connection_id) =
        connection.get::<String>("org.freedesktop.NetworkManager.Connection.Active", "Id")
      else {
        continue;
      };

      let ty = proxy.get::<u32>("org.freedesktop.NetworkManager.Device", "DeviceType")?;
      match ty {
        // Ethernet
        1 => {
          // The interface name looks better than the connection ID for ethernet devices.
          let id = proxy.get::<String>("org.freedesktop.NetworkManager.Device", "Interface")?;
          connections.push(id);
        }
        // Wifi
        2 => {
          connections.push(connection_id);
        }

        _ => {}
      }
    }

    Ok(connections)
  }
}
