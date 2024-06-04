use std::sync::Arc;

use parking_lot::Mutex;
use wayland_client::{
  backend::ObjectId,
  protocol::{wl_compositor, wl_output, wl_registry, wl_shm, wl_shm_pool, wl_surface},
  Connection, Dispatch, Proxy, QueueHandle,
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

use crate::{bar::Bar, config::Config};

#[derive(Default)]
struct AppData {
  monitors: Vec<Monitor>,
  shm_pool: Option<wl_shm_pool::WlShmPool>,

  // FIXME: This needs to be per-bar.
  surface: Option<wl_surface::WlSurface>,
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

impl Dispatch<wl_shm_pool::WlShmPool, ()> for AppData {
  fn event(
    state: &mut Self,
    output: &wl_shm_pool::WlShmPool,
    event: wl_shm_pool::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<AppData>,
  ) {
    println!("shm pool event: {:?}", event);
  }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for AppData {
  fn event(
    state: &mut Self,
    output: &wl_compositor::WlCompositor,
    event: wl_compositor::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<AppData>,
  ) {
    println!("compositor event: {:?}", event);
  }
}

impl Dispatch<wl_surface::WlSurface, ()> for AppData {
  fn event(
    state: &mut Self,
    surface: &wl_surface::WlSurface,
    event: wl_surface::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<AppData>,
  ) {
    println!("surface event: {:?}", event);
  }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for AppData {
  fn event(
    state: &mut Self,
    surface: &xdg_wm_base::XdgWmBase,
    event: xdg_wm_base::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<AppData>,
  ) {
    println!("wm_base event: {:?}", event);
  }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for AppData {
  fn event(
    state: &mut Self,
    surface: &xdg_surface::XdgSurface,
    event: xdg_surface::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<AppData>,
  ) {
    println!("xdg_surface event: {:?}", event);
  }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for AppData {
  fn event(
    state: &mut Self,
    surface: &xdg_toplevel::XdgToplevel,
    event: xdg_toplevel::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<AppData>,
  ) {
    println!("xdg_toplevel event: {:?}", event);
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
      } else if interface == wl_compositor::WlCompositor::interface().name {
        println!("compositor found");

        let compositor = registry.bind::<wl_compositor::WlCompositor, _, _>(name, version, qh, ());
        state.surface = Some(compositor.create_surface(qh, ()));
      } else if interface == xdg_wm_base::XdgWmBase::interface().name {
        println!("xdg_wm_base found");

        let wm_base = registry.bind::<xdg_wm_base::XdgWmBase, _, _>(name, version, qh, ());

        let surface = state.surface.take().unwrap();
        let xdg_surface = wm_base.get_xdg_surface(&surface, qh, ());

        xdg_surface.get_toplevel(qh, ());
        xdg_surface.set_window_geometry(50, 50, 100, 100);

        surface.commit();

        println!("created xdg_surface: {:?}", xdg_surface);
      } else if interface == wl_shm::WlShm::interface().name {
        // let pool = registry.bind::<wl_shm::WlShm, _, _>(name, version, qh, ());
        // pool.create_pool(fd, size, qh, udata)

        println!("found an shm pool");
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

  let mut app = AppData::default();

  loop {
    event_queue.roundtrip(&mut app).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));
  }
}
