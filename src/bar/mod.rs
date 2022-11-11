mod color;
mod ctx;
mod math;
mod module;
mod window;

pub use color::Color;
pub use ctx::RenderContext;
pub use math::{Pos, Rect};
pub use module::{Module, Updater};
pub use window::Window;

pub struct Bar {
  window:  Window,
  backend: Box<dyn Backend + Send + Sync>,

  pub modules: Modules,
}

pub struct Modules {
  left:   Vec<PositionedModule>,
  middle: Vec<PositionedModule>,
  right:  Vec<PositionedModule>,
}

struct PositionedModule {
  module: Module,
  pos:    u32,
  width:  u32,
}

impl From<Module> for PositionedModule {
  fn from(module: Module) -> Self { PositionedModule { module, pos: 0, width: 0 } }
}

pub trait Backend {
  fn render(&self, window: &Window);
}

#[derive(Clone, Copy, Debug)]
pub enum ModuleKey {
  Left(u32),
  Middle(u32),
  Right(u32),
}

impl Bar {
  pub fn new(width: u32, height: u32, backend: impl Backend + Send + Sync + 'static) -> Self {
    Bar {
      window:  Window::new(width, height),
      backend: Box::new(backend),
      modules: Modules::empty(),
    }
  }

  pub fn window(&self) -> &Window { &self.window }
  pub fn window_mut(&mut self) -> &mut Window { &mut self.window }

  pub fn render(&mut self) { self.backend.render(&self.window); }

  pub fn all_modules(&self) -> impl Iterator<Item = (ModuleKey, &Module)> { self.modules.iter() }
  pub fn update_module(&mut self, key: ModuleKey) {
    dbg!(key);
    let module = self.modules.by_key_mut(key);
    let mut ctx = RenderContext::new(&mut self.window, Pos { x: module.pos, y: 20 });
    module.module.imp().render(&mut ctx);
    if ctx.width != module.width {
      module.width = ctx.width;
      self.resize_from(key);
    }
  }

  fn resize_from(&mut self, key: ModuleKey) {
    match key {
      ModuleKey::Left(idx) => {
        let mut pos = self.modules.left[idx as usize].pos;
        for module in self.modules.left.iter_mut().skip(idx as usize) {
          pos += module.reposition(pos);
        }
      }
      ModuleKey::Middle(_) => {
        let mut width = 0;
        for module in self.modules.middle.iter() {
          width += module.width;
        }
        let mut pos = self.window.width() / 2 - width / 2;
        for module in self.modules.middle.iter_mut() {
          pos += module.reposition(pos);
        }
      }
      ModuleKey::Right(idx) => {
        let mut pos = self.modules.left[idx as usize].pos;
        for module in self.modules.left.iter_mut().skip(idx as usize).rev() {
          pos -= module.reposition(pos);
        }
      }
    }
  }
}

impl Modules {
  pub fn empty() -> Self { Modules { left: vec![], middle: vec![], right: vec![] } }
  pub fn set_from_config(&mut self, config: crate::Config) {
    self.left = config.modules_left.into_iter().map(Into::into).collect();
    self.middle = config.modules_middle.into_iter().map(Into::into).collect();
    self.right = config.modules_right.into_iter().map(Into::into).collect();
  }
  pub fn iter(&self) -> impl Iterator<Item = (ModuleKey, &Module)> {
    self
      .left
      .iter()
      .enumerate()
      .map(|(i, module)| (ModuleKey::Left(i as u32), &module.module))
      .chain(
        self
          .middle
          .iter()
          .enumerate()
          .map(|(i, module)| (ModuleKey::Middle(i as u32), &module.module)),
      )
      .chain(
        self
          .right
          .iter()
          .enumerate()
          .map(|(i, module)| (ModuleKey::Right(i as u32), &module.module)),
      )
  }
  fn by_key(&self, key: ModuleKey) -> &PositionedModule {
    match key {
      ModuleKey::Left(i) => &self.left[i as usize],
      ModuleKey::Middle(i) => &self.middle[i as usize],
      ModuleKey::Right(i) => &self.right[i as usize],
    }
  }
  fn by_key_mut(&mut self, key: ModuleKey) -> &mut PositionedModule {
    match key {
      ModuleKey::Left(i) => &mut self.left[i as usize],
      ModuleKey::Middle(i) => &mut self.middle[i as usize],
      ModuleKey::Right(i) => &mut self.right[i as usize],
    }
  }
}

impl PositionedModule {
  pub fn reposition(&mut self, pos: u32) -> u32 {
    self.pos = pos;
    self.width
  }
}
