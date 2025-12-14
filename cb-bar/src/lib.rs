use cb_core::{BarId, Render};
use kurbo::{Rect, Size};

pub trait Module {
  fn render(&self, render: &mut Render);
}

pub struct Bar {
  pub left_modules:   Vec<Box<dyn Module>>,
  pub center_modules: Vec<Box<dyn Module>>,
  pub right_modules:  Vec<Box<dyn Module>>,
}

struct BarLayout {
  size: Size,

  left_modules:   Vec<ModuleLayout>,
  center_modules: Vec<ModuleLayout>,
  right_modules:  Vec<ModuleLayout>,
}

struct ModuleLayout {
  module: Box<dyn Module>,
  bounds: Rect,
}

pub struct Config {
  pub bars: Vec<Bar>,
}

struct App {
  bars: Vec<BarLayout>,

  render: cb_core::RenderStore,
}

pub fn run(config: Config) { cb_backend_wayland::setup::<App>(config); }

impl Bar {
  fn into_layout(self) -> BarLayout {
    let mut layout = BarLayout {
      size: Size::new(1920.0, 30.0),

      left_modules:   self
        .left_modules
        .into_iter()
        .map(|m| ModuleLayout { module: m, bounds: Rect::ZERO })
        .collect(),
      center_modules: self
        .center_modules
        .into_iter()
        .map(|m| ModuleLayout { module: m, bounds: Rect::ZERO })
        .collect(),
      right_modules:  self
        .right_modules
        .into_iter()
        .map(|m| ModuleLayout { module: m, bounds: Rect::ZERO })
        .collect(),
    };

    layout.layout();

    layout
  }
}

impl BarLayout {
  fn layout(&mut self) {
    let mut x = 0.0;
    for module in &mut self.left_modules {
      module.bounds = Rect::new(x, 0.0, x + 50.0, self.size.height);
      x += module.bounds.size().width;
    }

    let mut x = 0.0;
    for module in &mut self.center_modules {
      module.bounds = Rect::new(x, 0.0, x + 50.0, self.size.height);
      x += module.bounds.size().width;
    }

    let offset = self.size.width - x / 2.0;
    for module in &mut self.center_modules {
      module.bounds.x0 += offset;
      module.bounds.x1 += offset;
    }

    let mut x = self.size.width;
    for module in self.right_modules.iter_mut().rev() {
      module.bounds = Rect::new(x - 240.0, 0.0, x, self.size.height);
      x -= module.bounds.size().width;
    }
  }

  fn draw(&self, render: &mut Render) {
    for module in &self.left_modules {
      render.set_offset(module.bounds.origin().to_vec2());
      module.module.render(render);
    }
    for module in &self.center_modules {
      render.set_offset(module.bounds.origin().to_vec2());
      module.module.render(render);
    }
    for module in &self.right_modules {
      render.set_offset(module.bounds.origin().to_vec2());
      module.module.render(render);
    }
  }
}

impl cb_core::App for App {
  type Config = Config;

  fn new(config: Config, device: &cb_core::wgpu::Device) -> Self {
    App {
      bars: config.bars.into_iter().map(|b| b.into_layout()).collect(),

      render: cb_core::RenderStore::new(device),
    }
  }

  fn create_bar(
    &mut self,
    id: BarId,
    device: &cb_core::wgpu::Device,
    format: cb_core::wgpu::TextureFormat,
    scale: f32,
    width: u32,
    height: u32,
  ) {
    self.render.create_bar(id, device, format, scale, width, height);
  }

  fn move_mouse(&mut self, id: BarId, pos: Option<(f64, f64)>) { self.render.move_mouse(id, pos); }

  fn draw(
    &mut self,
    id: BarId,
    device: &cb_core::wgpu::Device,
    queue: &cb_core::wgpu::Queue,
    output: &cb_core::wgpu::Texture,
  ) {
    if let Some(mut render) = self.render.for_bar(id) {
      self.bars[0].draw(&mut render);

      render.draw(device, queue, output);
    }
  }

  fn set_scale(&mut self, id: BarId, device: &cb_core::wgpu::Device, factor: i32) {
    self.render.set_scale(id, device, factor);
  }
}
