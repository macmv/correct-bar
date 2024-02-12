use std::cell::RefCell;

use correct_bar::bar::{Color, ModuleImpl, Updater};
use dbus::blocking::Connection;
use networkmanager::{
  devices::{Any, Wired},
  NetworkManager,
};

#[derive(Clone)]
pub struct Network {
  pub primary:   Color,
  pub secondary: Color,
}

struct NetworkState {
  dbus: Connection,
}

thread_local! {
  static NETWORK: RefCell<Option<NetworkState>> = RefCell::new(None);
}

impl ModuleImpl for Network {
  fn updater(&self) -> Updater { Updater::Never }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    NETWORK.with(|n| {
      let mut network = n.borrow_mut();
      if network.is_none() {
        *network = Some(NetworkState::new());
      }
      let n = network.as_mut().unwrap();

      for (i, c) in n.active_connection().iter().enumerate() {
        if i != 0 {
          ctx.draw_text(", ", self.secondary);
        }
        ctx.draw_text(&c, self.primary);
      }

      dbg!(n.active_connection());
    })
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
