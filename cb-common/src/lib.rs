use std::{
  collections::HashMap,
  os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd},
  sync::Arc,
};

pub struct Gpu<A> {
  instance: wgpu::Instance,
  adapter:  wgpu::Adapter,
  device:   wgpu::Device,
  queue:    wgpu::Queue,

  bars: HashMap<BarId, Bar>,

  app:    A,
  cursor: Option<(f64, f64)>,

  pub waker: Option<Arc<Waker>>,
}

pub struct Waker {
  fd: OwnedFd,
}
impl Waker {
  pub fn new() -> Self {
    unsafe {
      let fd = libc::eventfd(0, 0);

      if fd < 0 {
        panic!("eventfd");
      }

      let flags = libc::fcntl(fd, libc::F_GETFL);
      libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);

      Waker { fd: OwnedFd::from_raw_fd(fd) }
    }
  }

  pub fn fd(&self) -> BorrowedFd<'_> { unsafe { BorrowedFd::borrow_raw(self.fd.as_raw_fd()) } }

  pub fn wake(&self) {
    unsafe {
      libc::write(self.fd.as_raw_fd(), &1_u64 as *const _ as *const libc::c_void, 8);
    }
  }

  pub fn clear(&self) {
    unsafe {
      libc::read(self.fd.as_raw_fd(), &mut 0_u64 as *mut _ as *mut libc::c_void, 8);
    }
  }
}

pub trait App {
  type Config;

  fn new(config: Self::Config, device: &wgpu::Device) -> Self;
  fn waker(&self) -> Option<Arc<Waker>>;
  fn create_bar(
    &mut self,
    id: BarId,
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    scale: f64,
    width: u32,
    height: u32,
  );
  fn dirty(&self, id: BarId) -> bool;
  fn move_mouse(&mut self, id: BarId, pos: Option<(f64, f64)>);
  fn click_mouse(&mut self, id: BarId, pos: (f64, f64));
  fn set_scale(&mut self, id: BarId, device: &wgpu::Device, factor: f64);
  fn draw(&mut self, id: BarId, device: &wgpu::Device, queue: &wgpu::Queue, output: &wgpu::Texture);
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct BarId(u32);

pub struct Bar {
  surface:        wgpu::Surface<'static>,
  surface_config: wgpu::SurfaceConfiguration,

  pub scale: f64,
}

impl BarId {
  pub fn new(id: u32) -> Self { BarId(id) }
}

impl<A: App> Gpu<A> {
  pub fn new(config: A::Config) -> Self {
    let instance = wgpu::Instance::new(&Default::default());

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference:       wgpu::PowerPreference::default(),
      compatible_surface:     None,
      force_fallback_adapter: false,
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();

    let app = A::new(config, &device);
    let waker = app.waker();

    Gpu { instance, adapter, device, queue, bars: HashMap::new(), app, cursor: None, waker }
  }

  pub fn instance(&self) -> &wgpu::Instance { &self.instance }
  pub fn bar(&self, id: BarId) -> Option<&Bar> { self.bars.get(&id) }
  pub fn bar_mut(&mut self, id: BarId) -> Option<&mut Bar> { self.bars.get_mut(&id) }

  pub fn add_surface(
    &mut self,
    id: BarId,
    surface: wgpu::Surface<'static>,
    scale: f64,
    width: u32,
    height: u32,
  ) {
    let surface_caps = surface.get_capabilities(&self.adapter);
    // Shader code in this tutorial assumes an sRGB surface texture. Using a
    // different one will result in all the colors coming out darker. If you
    // want to support non sRGB surfaces, you'll need to account for that when
    // drawing to the frame.
    let surface_format = surface_caps
      .formats
      .iter()
      .find(|f| !f.is_srgb())
      .copied()
      .expect("sRGB surface not supported");

    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width,
      height,
      present_mode: wgpu::PresentMode::AutoVsync,
      alpha_mode: if surface_caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::PreMultiplied) {
        wgpu::CompositeAlphaMode::PreMultiplied
      } else {
        wgpu::CompositeAlphaMode::Opaque
      },
      view_formats: vec![],
      desired_maximum_frame_latency: 2,
    };

    surface.configure(&self.device, &config);

    self.bars.insert(id, Bar { surface, surface_config: config, scale });
    self.app.create_bar(id, &self.device, surface_format, scale, width, height);
  }

  pub fn move_mouse(&mut self, id: BarId, pos: Option<(f64, f64)>) {
    self.cursor = pos;
    self.app.move_mouse(id, pos);
  }
  pub fn click_mouse(&mut self, id: BarId) {
    if let Some(pos) = self.cursor {
      self.app.click_mouse(id, pos);
    }
  }

  pub fn needs_render(&self) -> bool { self.bars.keys().any(|id| self.app.dirty(*id)) }

  pub fn render(&mut self) {
    for (&id, bar) in &mut self.bars {
      if self.app.dirty(id) {
        let output = bar.surface.get_current_texture().unwrap();

        self.app.draw(id, &self.device, &self.queue, &output.texture);
        output.present();
      }
    }
  }

  pub fn render_bar(&mut self, id: BarId) {
    let output = self.bars.get(&id).unwrap().surface.get_current_texture().unwrap();

    self.app.draw(id, &self.device, &self.queue, &output.texture);
    output.present();
  }

  pub fn set_scale(&mut self, id: BarId, factor: f64) {
    let Some(bar) = self.bars.get_mut(&id) else { return };

    bar.surface_config.width = bar.surface_config.width * factor as u32;
    bar.surface_config.height = bar.surface_config.height * factor as u32;

    bar.surface.configure(&self.device, &bar.surface_config);
    self.app.set_scale(id, &self.device, factor);
  }
}
