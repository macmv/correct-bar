use std::{
  collections::HashMap,
  ops::{Index, IndexMut},
  sync::Arc,
};

use cb_core::{BarId, Render, RenderStore, Waker};
use kurbo::{Point, Rect, Size};

mod animation;
mod layout;

pub use animation::Animation;
pub use layout::{Layout, TextLayout};

pub trait Module {
  fn updater(&self) -> Updater { Updater::None }
  fn on_hover(&mut self, hover: bool) { let _ = hover; }
  fn on_click(&mut self, cursor: Point) { let _ = cursor; }
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

#[derive(Copy, Clone, PartialEq, Eq)]
enum Side {
  Left,
  Center,
  Right,
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct ModuleKey {
  side:  Side,
  index: usize,
}

struct BarLayout {
  size:        Size,
  scale:       f64,
  last_draw:   std::time::Instant,
  force_dirty: bool,
  hover:       Option<ModuleKey>,

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
  waker:  Arc<cb_core::Waker>,
}

pub fn run(config: Config) { cb_backend_wayland::setup::<App>(config); }

impl Bar {
  fn into_layout(self, scale: f64) -> BarLayout {
    BarLayout {
      size: Size::new(1920.0, 30.0),
      scale,
      last_draw: std::time::Instant::now(),
      force_dirty: true,
      hover: None,

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
    }
  }
}

impl BarLayout {
  fn layout(&mut self, store: &mut RenderStore, waker: &Arc<Waker>) {
    let mut x = 0.0;
    for module in &mut self.left_modules {
      module.layout(store, self.scale, waker);
      module.bounds.x0 += x;
      module.bounds.x1 += x;
      x += module.bounds.size().width;
    }

    let mut x = 0.0;
    for module in &mut self.center_modules {
      module.layout(store, self.scale, waker);
      x += module.bounds.size().width;
    }

    let offset = self.size.width - x / 2.0;
    for module in &mut self.center_modules {
      module.bounds.x0 += offset;
      module.bounds.x1 += offset;
    }

    let mut x = self.size.width;
    for module in self.right_modules.iter_mut().rev() {
      module.layout(store, self.scale, waker);
      x -= module.bounds.size().width;
      module.bounds.x0 += x;
      module.bounds.x1 += x;
    }
  }

  fn layout_dirty(&self) -> bool {
    if self.force_dirty {
      return true;
    }

    let elapsed = self.last_draw.elapsed();
    self.modules().any(|m| m.layout_dirty(elapsed))
  }

  fn render_dirty(&self) -> bool {
    if self.force_dirty {
      return true;
    }

    let elapsed = self.last_draw.elapsed();
    self.modules().any(|m| m.render_dirty(elapsed))
  }

  fn draw(&mut self, render: &mut Render) {
    self.last_draw = render.frame_time();
    self.force_dirty = false;

    for module in self.modules() {
      render.set_offset(module.bounds.origin().to_vec2());
      module.module.render(render);
    }
  }

  fn move_mouse(&mut self, pos: Option<(f64, f64)>) {
    let new_hover = if let Some(pos) = pos {
      let pos = Point::new(pos.0, pos.1);

      self.module_keys().find(|&k| self[k].bounds.contains(pos))
    } else {
      None
    };

    if new_hover != self.hover {
      if let Some(hover) = self.hover {
        self[hover].module.on_hover(false);
      }
      if let Some(hover) = new_hover {
        self[hover].module.on_hover(true);
      }

      self.hover = new_hover;
      self.force_dirty = true;
    }
  }

  fn click_mouse(&mut self, pos: (f64, f64)) {
    let pos = Point::new(pos.0, pos.1);
    let Some(hover) = self.module_keys().find(|&k| self[k].bounds.contains(pos)) else {
      return;
    };

    let m = &mut self[hover];
    m.module.on_click(pos - m.bounds.origin().to_vec2());
  }

  fn module_keys(&self) -> impl Iterator<Item = ModuleKey> {
    (0..self.left_modules.len())
      .map(|i| ModuleKey { side: Side::Left, index: i })
      .chain((0..self.center_modules.len()).map(|i| ModuleKey { side: Side::Center, index: i }))
      .chain((0..self.right_modules.len()).map(|i| ModuleKey { side: Side::Right, index: i }))
  }

  fn modules(&self) -> impl Iterator<Item = &ModuleLayout> {
    self.left_modules.iter().chain(self.center_modules.iter()).chain(self.right_modules.iter())
  }
}

impl Index<ModuleKey> for BarLayout {
  type Output = ModuleLayout;

  fn index(&self, index: ModuleKey) -> &Self::Output {
    match index.side {
      Side::Left => &self.left_modules[index.index],
      Side::Center => &self.center_modules[index.index],
      Side::Right => &self.right_modules[index.index],
    }
  }
}

impl IndexMut<ModuleKey> for BarLayout {
  fn index_mut(&mut self, index: ModuleKey) -> &mut Self::Output {
    match index.side {
      Side::Left => &mut self.left_modules[index.index],
      Side::Center => &mut self.center_modules[index.index],
      Side::Right => &mut self.right_modules[index.index],
    }
  }
}

impl ModuleLayout {
  fn layout_dirty(&self, elapsed: std::time::Duration) -> bool {
    match self.module.updater() {
      Updater::None => false,
      Updater::Animation => false,
      Updater::Every(interval) => elapsed > interval,
    }
  }
  fn render_dirty(&self, elapsed: std::time::Duration) -> bool {
    match self.module.updater() {
      Updater::None => false,
      Updater::Animation => true,
      Updater::Every(interval) => elapsed > interval,
    }
  }

  fn layout(&mut self, store: &mut RenderStore, scale: f64, waker: &Arc<Waker>) {
    let mut ctx = Layout { store, scale, bounds: Rect::ZERO, waker };
    self.module.layout(&mut ctx);
    self.bounds = ctx.bounds;
  }
}

impl cb_core::App for App {
  type Config = Config;

  fn new(config: Config, device: &cb_core::wgpu::Device) -> Self {
    App {
      config,
      bars: HashMap::new(),
      render: cb_core::RenderStore::new(device),
      waker: Arc::new(Waker::new()),
    }
  }

  fn waker(&self) -> Option<Arc<Waker>> { Some(self.waker.clone()) }

  fn create_bar(
    &mut self,
    id: BarId,
    device: &cb_core::wgpu::Device,
    format: cb_core::wgpu::TextureFormat,
    scale: f32,
    width: u32,
    height: u32,
  ) {
    let mut layout = (self.config.make_bar)().into_layout(f64::from(scale));
    layout.layout(&mut self.render, &self.waker);
    self.bars.insert(id, layout);

    self.render.create_bar(id, device, format, scale, width, height);
  }

  fn dirty(&self, id: BarId) -> bool { self.bars.get(&id).unwrap().render_dirty() }

  fn move_mouse(&mut self, id: BarId, pos: Option<(f64, f64)>) {
    self.bars.get_mut(&id).unwrap().move_mouse(pos);

    self.render.move_mouse(id, pos);
  }

  fn click_mouse(&mut self, id: BarId, pos: (f64, f64)) {
    self.bars.get_mut(&id).unwrap().click_mouse(pos);
  }

  fn draw(
    &mut self,
    id: BarId,
    device: &cb_core::wgpu::Device,
    queue: &cb_core::wgpu::Queue,
    output: &cb_core::wgpu::Texture,
  ) {
    if self.bars.get(&id).unwrap().layout_dirty() {
      self.bars.get_mut(&id).unwrap().layout(&mut self.render, &self.waker);
    }

    if let Some(mut render) = self.render.for_bar(id) {
      self.bars.get_mut(&id).unwrap().draw(&mut render);

      render.render(device, queue, output);
    }
  }

  fn set_scale(&mut self, id: BarId, device: &cb_core::wgpu::Device, factor: i32) {
    self.render.set_scale(id, device, factor);
    self.bars.get_mut(&id).unwrap().scale = factor as f64;
    self.bars.get_mut(&id).unwrap().layout(&mut self.render, &self.waker);
  }
}
