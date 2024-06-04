use std::sync::Arc;

use parking_lot::Mutex;
use wayland_client::{
  backend::ObjectId,
  protocol::{wl_output, wl_registry},
  Connection, Dispatch, Proxy, QueueHandle,
};

use crate::{bar::Bar, config::Config};

struct AppData {
  monitors: Vec<Monitor>,
}

#[derive(Debug)]
struct Monitor {
  id: ObjectId,

  // Logical position.
  x: i32,
  y: i32,

  // Logical width/height.
  width:  i32,
  height: i32,

  // Physical scale factor. TODO: f32?
  scale: i32,
}

impl Default for Monitor {
  fn default() -> Self { Self { id: ObjectId::null(), x: 0, y: 0, width: 0, height: 0, scale: 0 } }
}

impl Dispatch<wl_output::WlOutput, ()> for AppData {
  fn event(
    state: &mut Self,
    output: &wl_output::WlOutput,
    event: wl_output::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<AppData>,
  ) {
    let monitor = state.monitors.iter_mut().find(|m| m.id == output.id()).unwrap();
    match event {
      wl_output::Event::Mode { width, height, .. } => {
        monitor.width = width;
        monitor.height = height;
        if monitor.scale != 0 {
          // The geometry sends logical coordinates, and mode sends physical size. So only
          // divide the size here.
          monitor.width /= monitor.scale;
          monitor.height /= monitor.scale;
        }
      }
      wl_output::Event::Geometry { x, y, .. } => {
        monitor.x = x;
        monitor.y = y;
      }
      wl_output::Event::Scale { factor } => {
        monitor.scale = factor;
        if monitor.width != 0 {
          monitor.width /= factor;
          monitor.height /= factor;
        }
      }
      wl_output::Event::Done => {
        println!("monitors: {:?}", state.monitors);
      }
      _ => {}
    }
  }
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
  fn event(
    state: &mut Self,
    registry: &wl_registry::WlRegistry,
    event: wl_registry::Event,
    _: &(),
    _: &Connection,
    qh: &QueueHandle<AppData>,
  ) {
    if let wl_registry::Event::Global { name, interface, version } = event {
      if interface == wl_output::WlOutput::interface().name {
        let output = registry.bind::<wl_output::WlOutput, _, _>(name, version, qh, ());
        state.monitors.push(Monitor { id: output.id(), ..Default::default() });
      }
    }
  }
}

pub fn setup(config: Config) -> Vec<Arc<Mutex<Bar>>> {
  let conn = Connection::connect_to_env().unwrap();

  let display = conn.display();
  let mut event_queue = conn.new_event_queue();

  let qh = event_queue.handle();
  display.get_registry(&qh, ());

  let mut app = AppData { monitors: vec![] };

  loop {
    event_queue.roundtrip(&mut app).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));
  }
}
