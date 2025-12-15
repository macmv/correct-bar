use std::collections::HashMap;

use cb_core::{BarId, Render, RenderStore};
use kurbo::{Rect, Size};

mod animation;
mod layout;

pub use animation::Animation;
pub use layout::{Layout, TextLayout};

pub trait Module {
  fn updater(&self) -> Updater { Updater::None }
  fn layout(&mut self, layout: &mut Layout);
  fn render(&self, render: &mut Render);
}

pub enum Updater {
  None,
  Animation,
  Every(std::time::Duration),
}

pub struct Bar {
  pub left_modules:   Vec<Box<dyn Module>>,
  pub center_modules: Vec<Box<dyn Module>>,
  pub right_modules:  Vec<Box<dyn Module>>,
}

struct BarLayout {
  size:        Size,
  scale:       f64,
  last_draw:   std::time::Instant,
  force_dirty: bool,

  left_modules:   Vec<ModuleLayout>,
  center_modules: Vec<ModuleLayout>,
  right_modules:  Vec<ModuleLayout>,
}

struct ModuleLayout {
  module: Box<dyn Module>,
  bounds: Rect,
}

pub struct Config {
  pub make_bar: fn() -> Bar,
}

struct App {
  config: Config,
  bars:   HashMap<BarId, BarLayout>,

  render: cb_core::RenderStore,
}

pub fn run(config: Config) { cb_backend_wayland::setup::<App>(config); }

impl Bar {
  fn into_layout(self, store: &mut RenderStore, scale: f64) -> BarLayout {
    let mut layout = BarLayout {
      size: Size::new(1920.0, 30.0),
      scale,
      last_draw: std::time::Instant::now(),
      force_dirty: true,

      left_modules: self
        .left_modules
        .into_iter()
        .map(|m| ModuleLayout { module: m, bounds: Rect::ZERO })
        .collect(),
      center_modules: self
        .center_modules
        .into_iter()
        .map(|m| ModuleLayout { module: m, bounds: Rect::ZERO })
        .collect(),
      right_modules: self
        .right_modules
        .into_iter()
        .map(|m| ModuleLayout { module: m, bounds: Rect::ZERO })
        .collect(),
    };

    layout.layout(store);

    layout
  }
}

impl BarLayout {
  fn layout(&mut self, store: &mut RenderStore) {
    let mut x = 0.0;
    for module in &mut self.left_modules {
      module.layout(store, self.scale);
      module.bounds.x0 += x;
      module.bounds.x1 += x;
      x += module.bounds.size().width;
    }

    let mut x = 0.0;
    for module in &mut self.center_modules {
      module.layout(store, self.scale);
      x += module.bounds.size().width;
    }

    let offset = self.size.width - x / 2.0;
    for module in &mut self.center_modules {
      module.bounds.x0 += offset;
      module.bounds.x1 += offset;
    }

    let mut x = self.size.width;
    for module in self.right_modules.iter_mut().rev() {
      module.layout(store, self.scale);
      x -= module.bounds.size().width;
      module.bounds.x0 += x;
      module.bounds.x1 += x;
    }
  }

  fn dirty(&self) -> bool {
    if self.force_dirty {
      return true;
    }

    let elapsed = self.last_draw.elapsed();
    self.modules().any(|m| m.dirty(elapsed))
  }

  fn draw(&mut self, render: &mut Render) {
    self.last_draw = std::time::Instant::now();
    self.force_dirty = false;

    for module in self.modules() {
      render.set_offset(module.bounds.origin().to_vec2());
      module.module.render(render);
    }
  }

  fn modules(&self) -> impl Iterator<Item = &ModuleLayout> {
    self.left_modules.iter().chain(self.center_modules.iter()).chain(self.right_modules.iter())
  }
}

impl ModuleLayout {
  fn dirty(&self, elapsed: std::time::Duration) -> bool {
    match self.module.updater() {
      Updater::None => false,
      Updater::Animation => true,
      Updater::Every(interval) => elapsed > interval,
    }
  }

  fn layout(&mut self, store: &mut RenderStore, scale: f64) {
    let mut ctx = Layout { store, scale, bounds: Rect::ZERO };
    self.module.layout(&mut ctx);
    self.bounds = ctx.bounds;
  }
}

impl cb_core::App for App {
  type Config = Config;

  fn new(config: Config, device: &cb_core::wgpu::Device) -> Self {
    App { config, bars: HashMap::new(), render: cb_core::RenderStore::new(device) }
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
    self.bars.insert(id, (self.config.make_bar)().into_layout(&mut self.render, f64::from(scale)));

    self.render.create_bar(id, device, format, scale, width, height);
  }

  fn dirty(&self, id: BarId) -> bool { self.bars.get(&id).unwrap().dirty() }

  fn move_mouse(&mut self, id: BarId, pos: Option<(f64, f64)>) {
    self.bars.get_mut(&id).unwrap().force_dirty = true;
    self.render.move_mouse(id, pos);
  }

  fn draw(
    &mut self,
    id: BarId,
    device: &cb_core::wgpu::Device,
    queue: &cb_core::wgpu::Queue,
    output: &cb_core::wgpu::Texture,
  ) {
    self.bars.get_mut(&id).unwrap().layout(&mut self.render);

    if let Some(mut render) = self.render.for_bar(id) {
      self.bars.get_mut(&id).unwrap().draw(&mut render);

      render.render(device, queue, output);
    }
  }

  fn set_scale(&mut self, id: BarId, device: &cb_core::wgpu::Device, factor: i32) {
    self.render.set_scale(id, device, factor);
    self.bars.get_mut(&id).unwrap().scale = factor as f64;
    self.bars.get_mut(&id).unwrap().layout(&mut self.render);
  }
}
