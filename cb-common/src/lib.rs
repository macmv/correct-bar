use std::collections::HashMap;

pub struct Gpu<A> {
  instance: wgpu::Instance,
  adapter:  wgpu::Adapter,
  device:   wgpu::Device,
  queue:    wgpu::Queue,

  bars: HashMap<BarId, Bar>,

  app: A,
}

pub trait App {
  type Config;

  fn new(config: Self::Config, device: &wgpu::Device) -> Self;
  fn create_bar(
    &mut self,
    id: BarId,
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    scale: f32,
    width: u32,
    height: u32,
  );
  fn move_mouse(&mut self, id: BarId, pos: Option<(f64, f64)>);
  fn set_scale(&mut self, id: BarId, device: &wgpu::Device, factor: i32);
  fn draw(&mut self, id: BarId, device: &wgpu::Device, queue: &wgpu::Queue, output: &wgpu::Texture);
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct BarId(u32);

pub struct Bar {
  surface:        wgpu::Surface<'static>,
  surface_config: wgpu::SurfaceConfiguration,

  pub scale: f32,
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

    Gpu { instance, adapter, device, queue, bars: HashMap::new(), app }
  }

  pub fn instance(&self) -> &wgpu::Instance { &self.instance }
  pub fn bar(&self, id: BarId) -> Option<&Bar> { self.bars.get(&id) }
  pub fn bar_mut(&mut self, id: BarId) -> Option<&mut Bar> { self.bars.get_mut(&id) }

  pub fn add_surface(
    &mut self,
    id: BarId,
    surface: wgpu::Surface<'static>,
    scale: f32,
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
      present_mode: surface_caps.present_modes[0],
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
    self.app.move_mouse(id, pos);
    self.draw(id);
  }

  pub fn draw(&mut self, id: BarId) {
    let output = self.bars.get(&id).unwrap().surface.get_current_texture().unwrap();

    self.app.draw(id, &self.device, &self.queue, &output.texture);
    output.present();
  }

  pub fn set_scale(&mut self, id: BarId, factor: i32) {
    let Some(bar) = self.bars.get_mut(&id) else { return };

    bar.surface_config.width = bar.surface_config.width * factor as u32;
    bar.surface_config.height = bar.surface_config.height * factor as u32;

    bar.surface.configure(&self.device, &bar.surface_config);
    self.app.set_scale(id, &self.device, factor);
  }
}
