mod color;
mod ctx;
mod math;
mod module;
mod window;

pub use color::Color;
pub use ctx::RenderContext;
pub use math::{Pos, Rect};
pub use module::{Module, ModuleImpl, Updater};
pub use window::{DynamicBuffer, Window};

use crate::config::Config;

pub struct Bar {
  window:     Window,
  backend:    Box<dyn Backend + Send + Sync>,
  background: Color,

  pub modules: Modules,
}

pub struct Modules {
  left:   Vec<PositionedModule>,
  middle: Vec<PositionedModule>,
  right:  Vec<PositionedModule>,
}

#[derive(Debug)]
struct PositionedModule {
  module: Module,
  pos:    u32,
  buffer: DynamicBuffer,
}

impl PositionedModule {
  pub fn new(module: Module, height: u32, background: Color) -> PositionedModule {
    PositionedModule { module, pos: 0, buffer: DynamicBuffer::new(height, background) }
  }
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
  pub fn from_config(config: &Config, backend: impl Backend + Send + Sync + 'static) -> Self {
    Bar {
      window:     Window::new(config.window.width, config.window.height),
      backend:    Box::new(backend),
      background: config.background,
      modules:    Modules::empty(),
    }
  }

  pub fn window(&self) -> &Window { &self.window }
  pub fn window_mut(&mut self) -> &mut Window { &mut self.window }

  pub fn render(&mut self) { self.backend.render(&self.window); }

  pub fn all_modules(&self) -> impl Iterator<Item = (ModuleKey, &Module)> { self.modules.iter() }
  pub fn update_module(&mut self, key: ModuleKey) {
    println!("updating");
    /*
    let module = self.modules.by_key_mut(key);
    let mut ctx = RenderContext::new(&mut module.buffer, Pos { x: module.pos, y: 20 });
    module.module.imp().render(&mut ctx);
    if ctx.width != module.width {
      module.width = ctx.width;
    */
    self.update_from(key);
    /*
      }
    */
  }

  fn update_all(&mut self) {
    macro_rules! draw_module {
      ( $module:expr ) => {{
        let background = $module.module.imp().background().unwrap_or(self.background);
        $module.buffer.fill_and_set_background(background);
        let mut ctx =
          RenderContext::new(&mut self.window, &mut $module.buffer, Pos { x: $module.pos, y: 20 });
        $module.module.imp().render(&mut ctx);
      }};
    }

    macro_rules! copy_module {
      ( $module:expr ) => {
        self.window.buffer_mut().copy_from(Pos { x: $module.pos, y: 0 }, &$module.buffer.buffer());
      };
    }

    self.modules.left.iter_mut().for_each(|m| draw_module!(m));
    self.modules.middle.iter_mut().for_each(|m| draw_module!(m));
    self.modules.right.iter_mut().for_each(|m| draw_module!(m));

    let mut pos = 0;
    for module in &mut self.modules.left {
      module.pos = pos;
      copy_module!(module);
      pos += module.buffer.width();
    }

    let width: u32 = self.modules.middle.iter().map(|m| m.buffer.width()).sum();
    let mut pos = self.window.width() / 2 - width / 2;
    for module in self.modules.middle.iter_mut() {
      module.pos = pos;
      copy_module!(module);
      pos += module.buffer.width();
    }

    let mut pos = self.window.width();
    for module in self.modules.right.iter_mut().rev() {
      pos -= module.buffer.width();
      module.pos = pos;
      copy_module!(module);
    }
  }

  fn update_from(&mut self, key: ModuleKey) {
    self.update_all();
    /*
    match key {
      ModuleKey::Left(_) => {
        let mut pos = self.modules.left[0].pos;
        for module in self.modules.left.iter_mut() {
          module.pos = pos;
          pos += module.width;
        }
      }
      ModuleKey::Middle(_) => {
        let mut width = 0;
        for module in self.modules.middle.iter() {
          width += module.width;
        }
        let mut pos = self.window.width() / 2 - width / 2;
        for module in self.modules.middle.iter_mut() {
          module.pos = pos;
          pos += module.width;
        }
      }
      ModuleKey::Right(_) => {
        let mut pos = self.modules.right[0].pos;
        if pos < self.window.width() / 2 {
          pos = self.window.width();
        }
        for module in self.modules.right.iter_mut().rev() {
          pos -= module.width;
          module.pos = pos;
        }
      }
    }
    */
  }
}

impl Modules {
  pub fn empty() -> Self { Modules { left: vec![], middle: vec![], right: vec![] } }
  pub fn set_from_config(&mut self, config: crate::Config) {
    self.left = config
      .modules_left
      .into_iter()
      .map(|m| PositionedModule::new(m, config.window.height, config.background))
      .collect();
    self.middle = config
      .modules_middle
      .into_iter()
      .map(|m| PositionedModule::new(m, config.window.height, config.background))
      .collect();
    self.right = config
      .modules_right
      .into_iter()
      .map(|m| PositionedModule::new(m, config.window.height, config.background))
      .collect();
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
