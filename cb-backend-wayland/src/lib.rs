use std::{collections::HashMap, ptr::NonNull};

use cb_common::{BarId, Gpu};
use wayland_client::{
  Connection, Dispatch, Proxy, QueueHandle,
  protocol::{wl_compositor, wl_display, wl_output, wl_registry, wl_shm_pool, wl_surface},
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};
use wgpu::{
  SurfaceTargetUnsafe,
  rwh::{RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle},
};

#[derive(Default)]
struct AppData {
  display:   Option<wl_display::WlDisplay>,
  monitors:  HashMap<BarId, Monitor>,
  _shm_pool: Option<wl_shm_pool::WlShmPool>,

  compositor: Option<wl_compositor::WlCompositor>,
  shell:      Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
}

#[derive(Debug)]
struct Monitor {
  output: wl_output::WlOutput,

  surface:       Option<wl_surface::WlSurface>,
  layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,

  // Logical position.
  x: i32,
  y: i32,

  // Logical width/height.
  width:  i32,
  height: i32,

  // Physical scale factor. TODO: f32?
  scale: i32,
}

impl AppData {
  fn on_change(&mut self, qh: &QueueHandle<AppData>) {
    if let Some(shell) = &self.shell
      && let Some(compositor) = &self.compositor
    {
      for (id, monitor) in &mut self.monitors {
        if monitor.surface.is_none() {
          monitor.surface = Some(compositor.create_surface(qh, ()));
        }

        if monitor.layer_surface.is_none() {
          let surface = monitor.surface.as_ref().unwrap();

          let layer_surface = shell.get_layer_surface(
            surface,
            Some(&monitor.output),
            zwlr_layer_shell_v1::Layer::Background,
            "foo".into(),
            qh,
            *id,
          );

          layer_surface.set_size(0, 20);
          layer_surface.set_anchor(
            zwlr_layer_surface_v1::Anchor::Top
              | zwlr_layer_surface_v1::Anchor::Left
              | zwlr_layer_surface_v1::Anchor::Right,
          );
          layer_surface.set_margin(0, 0, 0, 0);
          layer_surface.set_exclusive_edge(zwlr_layer_surface_v1::Anchor::Top);
          layer_surface.set_exclusive_zone(20);

          surface.commit();

          monitor.layer_surface = Some(layer_surface);
        }
      }
    }
  }
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
    let monitor = state.monitors.values_mut().find(|m| &m.output == output).unwrap();
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
    _state: &mut Self,
    _output: &wl_shm_pool::WlShmPool,
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
    _state: &mut Self,
    _output: &wl_compositor::WlCompositor,
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
    _state: &mut Self,
    _surface: &wl_surface::WlSurface,
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
    _state: &mut Self,
    _surface: &xdg_wm_base::XdgWmBase,
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
    _state: &mut Self,
    _surface: &xdg_surface::XdgSurface,
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
    _state: &mut Self,
    _surface: &xdg_toplevel::XdgToplevel,
    event: xdg_toplevel::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<AppData>,
  ) {
    println!("xdg_toplevel event: {:?}", event);
  }
}

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for AppData {
  fn event(
    _state: &mut Self,
    _shell: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
    event: zwlr_layer_shell_v1::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<AppData>,
  ) {
    println!("layer shell event: {:?}", event);
  }
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, BarId> for AppData {
  fn event(
    state: &mut Self,
    _shell: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    event: zwlr_layer_surface_v1::Event,
    id: &BarId,
    _: &Connection,
    _: &QueueHandle<AppData>,
  ) {
    match event {
      zwlr_layer_surface_v1::Event::Configure { serial, width, height } => {
        if let Some(monitor) = state.monitors.get_mut(id) {
          monitor.width = width as i32;
          monitor.height = height as i32;
          monitor.layer_surface.as_ref().unwrap().ack_configure(serial);

          unsafe {
            let raw_display = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
              NonNull::new_unchecked(state.display.as_mut().unwrap().id().as_ptr() as *mut _),
            ));
            let raw_window = RawWindowHandle::Wayland(WaylandWindowHandle::new(
              NonNull::new_unchecked(monitor.surface.as_mut().unwrap().id().as_ptr() as *mut _),
            ));

            let mut gpu = Gpu::new();
            let surface = gpu
              .instance()
              .create_surface_unsafe(SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: raw_display,
                raw_window_handle:  raw_window,
              })
              .expect("create_surface failed");
            gpu.add_surface(*id, surface, width, height);
          }
        }
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
        let id = BarId::new(name);

        state.monitors.insert(
          id,
          Monitor {
            output:        registry.bind(name, version, qh, ()),
            surface:       None,
            layer_surface: None,
            x:             0,
            y:             0,
            width:         0,
            height:        0,
            scale:         1,
          },
        );
      } else if interface == wl_compositor::WlCompositor::interface().name {
        state.compositor = Some(registry.bind(name, version, qh, ()));
      } else if interface == zwlr_layer_shell_v1::ZwlrLayerShellV1::interface().name {
        state.shell = Some(registry.bind(name, version, qh, ()));
      }

      state.on_change(qh);
    }
  }
}

pub fn setup() {
  let conn = Connection::connect_to_env().unwrap();

  let display = conn.display();
  let mut event_queue = conn.new_event_queue();

  let qh = event_queue.handle();
  display.get_registry(&qh, ());

  let mut app = AppData::default();
  app.display = Some(display);

  loop {
    event_queue.roundtrip(&mut app).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));
  }
}
