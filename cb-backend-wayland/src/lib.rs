use std::{collections::HashMap, os::fd::AsRawFd, ptr::NonNull};

use cb_common::{BarId, Gpu};
use wayland_client::{
  Connection, Dispatch, Proxy, QueueHandle,
  protocol::{
    wl_callback, wl_compositor, wl_display, wl_output, wl_pointer, wl_registry, wl_seat,
    wl_shm_pool, wl_surface,
  },
};
use wayland_protocols::{
  wp::fractional_scale::v1::client::{wp_fractional_scale_manager_v1, wp_fractional_scale_v1},
  xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base},
};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};
use wgpu::{
  SurfaceTargetUnsafe,
  rwh::{RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle},
};

struct AppData<A> {
  gpu: Gpu<A>,

  display:  Option<wl_display::WlDisplay>,
  monitors: HashMap<BarId, Monitor>,

  compositor:       Option<wl_compositor::WlCompositor>,
  seat:             Option<wl_seat::WlSeat>,
  shell:            Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
  fractional_scale: Option<wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1>,

  pointer_surface: Option<wl_surface::WlSurface>,
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
}

impl<A: cb_common::App + 'static> AppData<A> {
  fn on_change(&mut self, qh: &QueueHandle<AppData<A>>) {
    if let Some(shell) = &self.shell
      && let Some(compositor) = &self.compositor
    {
      for (id, monitor) in &mut self.monitors {
        if monitor.surface.is_none() {
          monitor.surface = Some(compositor.create_surface(qh, *id));

          if let Some(scale) = &self.fractional_scale {
            scale.get_fractional_scale(monitor.surface.as_ref().unwrap(), qh, *id);
          }
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

          layer_surface.set_size(0, 40);
          layer_surface.set_anchor(
            zwlr_layer_surface_v1::Anchor::Top
              | zwlr_layer_surface_v1::Anchor::Left
              | zwlr_layer_surface_v1::Anchor::Right,
          );
          layer_surface.set_margin(0, 0, 0, 0);
          layer_surface.set_exclusive_edge(zwlr_layer_surface_v1::Anchor::Top);
          layer_surface.set_exclusive_zone(30);

          surface.commit();

          monitor.layer_surface = Some(layer_surface);
        }
      }
    }
  }
}

impl<A> Dispatch<wl_output::WlOutput, ()> for AppData<A> {
  fn event(
    state: &mut Self,
    output: &wl_output::WlOutput,
    event: wl_output::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
    let monitor = state.monitors.values_mut().find(|m| &m.output == output).unwrap();
    match event {
      wl_output::Event::Mode { width, height, .. } => {
        monitor.width = width;
        monitor.height = height;
      }
      wl_output::Event::Geometry { x, y, .. } => {
        monitor.x = x;
        monitor.y = y;
      }
      wl_output::Event::Done => {
        println!("monitors: {:?}", state.monitors);
      }
      _ => {}
    }
  }
}

impl<A> Dispatch<wl_shm_pool::WlShmPool, ()> for AppData<A> {
  fn event(
    _state: &mut Self,
    _output: &wl_shm_pool::WlShmPool,
    event: wl_shm_pool::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
    println!("shm pool event: {:?}", event);
  }
}

impl<A> Dispatch<wl_compositor::WlCompositor, ()> for AppData<A> {
  fn event(
    _state: &mut Self,
    _output: &wl_compositor::WlCompositor,
    event: wl_compositor::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
    println!("compositor event: {:?}", event);
  }
}

impl<A: cb_common::App> Dispatch<wl_surface::WlSurface, BarId> for AppData<A> {
  fn event(
    state: &mut Self,
    surface: &wl_surface::WlSurface,
    event: wl_surface::Event,
    id: &BarId,
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
    // Prefer fractional scales when available.
    if state.fractional_scale.is_some() {
      return;
    }

    match event {
      wl_surface::Event::PreferredBufferScale { factor } => {
        let bar = state.gpu.bar_mut(*id).unwrap();
        if bar.scale != factor as f64 {
          bar.scale = factor as f64;
          state.gpu.set_scale(*id, factor as f64);

          surface.set_buffer_scale(factor);
          surface.commit();
          state.gpu.render_bar(*id);
        }
      }
      _ => {
        // println!("surface event: {:?}", event);
      }
    }
  }
}

impl<A> Dispatch<xdg_wm_base::XdgWmBase, ()> for AppData<A> {
  fn event(
    _state: &mut Self,
    _surface: &xdg_wm_base::XdgWmBase,
    event: xdg_wm_base::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
    println!("wm_base event: {:?}", event);
  }
}

impl<A> Dispatch<xdg_surface::XdgSurface, ()> for AppData<A> {
  fn event(
    _state: &mut Self,
    _surface: &xdg_surface::XdgSurface,
    event: xdg_surface::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
    println!("xdg_surface event: {:?}", event);
  }
}

impl<A> Dispatch<xdg_toplevel::XdgToplevel, ()> for AppData<A> {
  fn event(
    _state: &mut Self,
    _surface: &xdg_toplevel::XdgToplevel,
    event: xdg_toplevel::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
    println!("xdg_toplevel event: {:?}", event);
  }
}

impl<A> Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for AppData<A> {
  fn event(
    _state: &mut Self,
    _shell: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
    event: zwlr_layer_shell_v1::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
    println!("layer shell event: {:?}", event);
  }
}

impl<A: cb_common::App> Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, BarId> for AppData<A> {
  fn event(
    state: &mut Self,
    _shell: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    event: zwlr_layer_surface_v1::Event,
    id: &BarId,
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
    match event {
      zwlr_layer_surface_v1::Event::Configure { serial, width, height } => {
        if let Some(monitor) = state.monitors.get_mut(id) {
          monitor.layer_surface.as_ref().unwrap().ack_configure(serial);

          unsafe {
            let raw_display = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
              NonNull::new_unchecked(state.display.as_mut().unwrap().id().as_ptr() as *mut _),
            ));
            let raw_window = RawWindowHandle::Wayland(WaylandWindowHandle::new(
              NonNull::new_unchecked(monitor.surface.as_mut().unwrap().id().as_ptr() as *mut _),
            ));

            let surface = state
              .gpu
              .instance()
              .create_surface_unsafe(SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: raw_display,
                raw_window_handle:  raw_window,
              })
              .expect("create_surface failed");
            state.gpu.add_surface(*id, surface, 1.0, width, height);
            state.gpu.render_bar(*id);
          }

          // TODO: Request a new frame with this:
          // monitor.surface.as_ref().unwrap().frame(qh, *id);
        }
      }
      _ => {}
    }
  }
}

impl<A: cb_common::App> Dispatch<wl_callback::WlCallback, BarId> for AppData<A> {
  fn event(
    state: &mut Self,
    _: &wl_callback::WlCallback,
    _: wl_callback::Event,
    id: &BarId,
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
    println!("drawing!!");
    state.gpu.render_bar(*id);
  }
}

impl<A> Dispatch<wl_seat::WlSeat, ()> for AppData<A> {
  fn event(
    _: &mut Self,
    _: &wl_seat::WlSeat,
    event: wl_seat::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
    println!("seat: {event:?}");
  }
}

impl<A> Dispatch<wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1, ()> for AppData<A> {
  fn event(
    _: &mut Self,
    _: &wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
    _: wp_fractional_scale_manager_v1::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
  }
}

impl<A: cb_common::App> Dispatch<wp_fractional_scale_v1::WpFractionalScaleV1, BarId>
  for AppData<A>
{
  fn event(
    state: &mut Self,
    _: &wp_fractional_scale_v1::WpFractionalScaleV1,
    event: wp_fractional_scale_v1::Event,
    id: &BarId,
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
    match event {
      wp_fractional_scale_v1::Event::PreferredScale { scale } => {
        let scale = scale as f64 / 120.0;

        let Some(monitor) = state.monitors.get_mut(id) else { return };
        let Some(surface) = monitor.surface.as_ref() else { return };

        let bar = state.gpu.bar_mut(*id).unwrap();
        if bar.scale != scale {
          bar.scale = scale;
          state.gpu.set_scale(*id, scale);

          surface.commit();
          state.gpu.render_bar(*id);
        }

        println!("fractional scale: {scale}");
      }
      _ => {}
    }
  }
}

impl<A: cb_common::App> Dispatch<wl_pointer::WlPointer, ()> for AppData<A> {
  fn event(
    state: &mut Self,
    _: &wl_pointer::WlPointer,
    event: wl_pointer::Event,
    _: &(),
    _: &Connection,
    _: &QueueHandle<Self>,
  ) {
    match event {
      wl_pointer::Event::Enter { surface, surface_x, surface_y, .. } => {
        state.pointer_surface = Some(surface);
        if let Some(bar) = state.pointer_bar() {
          state.gpu.move_mouse(bar, Some((surface_x, surface_y)));
        }
      }
      wl_pointer::Event::Leave { .. } => {
        if let Some(bar) = state.pointer_bar() {
          state.gpu.move_mouse(bar, None);
        }

        state.pointer_surface = None;
      }

      wl_pointer::Event::Button { state: s, .. } => {
        if matches!(s.into_result(), Ok(wl_pointer::ButtonState::Pressed)) {
          if let Some(bar) = state.pointer_bar() {
            state.gpu.click_mouse(bar);
          }
        }
      }

      wl_pointer::Event::Motion { surface_x, surface_y, .. } => {
        if let Some(bar) = state.pointer_bar() {
          state.gpu.move_mouse(bar, Some((surface_x, surface_y)));
        }
      }

      _ => {}
    }
  }
}

impl<A: cb_common::App> AppData<A> {
  fn pointer_bar(&mut self) -> Option<BarId> {
    let surface = self.pointer_surface.as_ref()?;

    for (id, m) in &self.monitors {
      if m.surface.as_ref().is_some_and(|m| surface == m) {
        return Some(*id);
      }
    }

    None
  }
}

impl<A: cb_common::App + 'static> Dispatch<wl_registry::WlRegistry, ()> for AppData<A> {
  fn event(
    state: &mut Self,
    registry: &wl_registry::WlRegistry,
    event: wl_registry::Event,
    _: &(),
    _: &Connection,
    qh: &QueueHandle<Self>,
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
          },
        );
      } else if interface == wl_compositor::WlCompositor::interface().name {
        state.compositor = Some(registry.bind(name, version, qh, ()));
      } else if interface == wl_seat::WlSeat::interface().name {
        state.seat = Some(registry.bind(name, version, qh, ()));
        state.seat.as_ref().unwrap().get_pointer(qh, ());
      } else if interface == zwlr_layer_shell_v1::ZwlrLayerShellV1::interface().name {
        state.shell = Some(registry.bind(name, version, qh, ()));
      } else if interface
        == wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1::interface().name
      {
        state.fractional_scale = Some(registry.bind(name, version, qh, ()));
      }

      state.on_change(qh);
    }
  }
}

pub fn setup<A: cb_common::App + 'static>(config: A::Config) {
  let conn = Connection::connect_to_env().unwrap();

  let display = conn.display();
  let mut event_queue = conn.new_event_queue();

  let qh = event_queue.handle();
  display.get_registry(&qh, ());

  let mut app = AppData {
    gpu:              Gpu::<A>::new(config),
    monitors:         HashMap::new(),
    compositor:       None,
    shell:            None,
    seat:             None,
    fractional_scale: None,
    display:          None,
    pointer_surface:  None,
  };
  app.display = Some(display);

  loop {
    event_queue.dispatch_pending(&mut app).unwrap();

    if app.gpu.needs_render() {
      app.gpu.render();
    }

    event_queue.flush().unwrap();

    if let Some(guard) = event_queue.prepare_read() {
      blocking_read(guard, app.gpu.waker.as_deref(), Some(std::time::Duration::from_secs(1)))
        .unwrap();
    }
  }
}

// Copied from `wayland-client::conn::blocking_read`, so we can pass in a
// timeout.
fn blocking_read(
  guard: wayland_backend::client::ReadEventsGuard,
  waker: Option<&cb_common::Waker>,
  timeout: Option<std::time::Duration>,
) -> Result<usize, wayland_backend::client::WaylandError> {
  let mut fds: [libc::pollfd; 2] = unsafe { std::mem::zeroed() };

  fds[0].fd = guard.connection_fd().as_raw_fd();
  fds[0].events = libc::POLLIN | libc::POLLERR;
  let mut n_fds = 1;

  if let Some(waker) = waker {
    fds[1].fd = waker.fd().as_raw_fd();
    fds[1].events = libc::POLLIN | libc::POLLERR;
    n_fds = 2;
  }

  loop {
    let ret = unsafe {
      libc::poll(&mut fds as *mut _, n_fds, timeout.map(|t| t.as_millis() as _).unwrap_or(-1))
    };

    if ret == -1 {
      let err = std::io::Error::last_os_error();
      match err.kind() {
        std::io::ErrorKind::Interrupted => continue,
        _ => return Err(wayland_backend::client::WaylandError::Io(err)),
      }
    }

    break;
  }

  let mut res = Ok(0);

  let mut guard = Some(guard);
  for (i, fd) in fds.iter().enumerate() {
    if fd.revents == 0 {
      continue;
    }

    match i {
      0 => {
        res = match guard.take().unwrap().read() {
          Ok(n) => Ok(n),
          // if we are still "wouldblock", just return 0; the caller will retry.
          Err(wayland_backend::client::WaylandError::Io(e))
            if e.kind() == std::io::ErrorKind::WouldBlock =>
          {
            Ok(0)
          }
          Err(e) => Err(e),
        }
      }

      1 => waker.as_ref().unwrap().clear(),

      _ => unreachable!(),
    }
  }

  res
}
