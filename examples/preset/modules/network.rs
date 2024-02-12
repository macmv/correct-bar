use correct_bar::bar::{Color, ModuleImpl, Updater};
use crossbeam_channel::Receiver;
use dbus::blocking::Connection;
use networkmanager::{
  devices::{Any, Wired},
  NetworkManager,
};
use parking_lot::Mutex;
use std::sync::Arc;

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

  fn new_inner(primary: Color, secondary: Color) -> Result<Network, ()> {
    let state = NetworkState::new();

    let (tx, rx) = crossbeam_channel::bounded(0);

    Ok(Network { primary, secondary, recv: rx, state: Arc::new(Mutex::new(state)) })
  }
}

impl ModuleImpl for Network {
  fn updater(&self) -> Updater { Updater::Never }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    let s = self.state.lock();

    for (i, c) in s.active_connection().iter().enumerate() {
      if i != 0 {
        ctx.draw_text(", ", self.secondary);
      }
      ctx.draw_text(&c, self.primary);
    }
  }
  fn box_clone(&self) -> Box<dyn ModuleImpl + Send + Sync> { Box::new(self.clone()) }
}

impl NetworkState {
  pub fn new() -> NetworkState { NetworkState { dbus: Connection::new_system().unwrap() } }

  pub fn active_connection(&self) -> Vec<String> {
    let nm = NetworkManager::new(&self.dbus);
    let mut connections = vec![];

    for d in nm.get_devices().unwrap() {
      match d {
        networkmanager::devices::Device::WiFi(w) => {
          if let Ok(id) = w.active_connection().unwrap().id() {
            connections.push(id);
          }
        }
        networkmanager::devices::Device::Ethernet(e) => {
          if e.active_connection().unwrap().id().is_ok() {
            connections.push(e.interface().unwrap());
          }
        }
        _ => {}
      }
    }

    connections
  }
}
