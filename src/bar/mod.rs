mod color;
mod ctx;
mod module;
mod window;

pub use color::Color;
pub use ctx::RenderContext;
pub use module::{Module, ModuleImpl, Padding, Updater};
pub use window::{DynamicBuffer, Window};

use crate::{
  config::Config,
  math::{Pos, Rect},
};
use std::fmt;

pub struct Bar {
  config:  Config,
  window:  Window,
  backend: Box<dyn Backend + Send + Sync>,

  pub modules: Modules,
}

pub struct Modules {
  left:   Vec<PositionedModule>,
  middle: Vec<PositionedModule>,
  right:  Vec<PositionedModule>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cursor {
  Normal,
  Hand,
}

#[derive(Debug)]
struct PositionedModule {
  module: Module,
  pos:    u32,
  buffer: DynamicBuffer,

  stale_width:   bool,
  stale_content: bool,

  click_regions: Vec<ClickRegion>,
}

struct ClickRegion {
  region: Rect,
  func:   Box<dyn Fn() + Send + Sync>,
}

impl fmt::Debug for ClickRegion {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ClickRegion").field("region", &self.region).finish()
  }
}

impl PositionedModule {
  pub fn new(module: Module, height: u32, background: Color) -> PositionedModule {
    PositionedModule {
      module,
      pos: 0,
      buffer: DynamicBuffer::new(height, background),
      stale_width: true,
      stale_content: true,
      click_regions: vec![],
    }
  }

  pub fn width(&self, _config: &Config) -> u32 { self.buffer.width() }
  pub fn on_click(&self, pos: Pos) {
    let module_pos = Pos { x: pos.x - self.pos as i32, y: pos.y };
    for region in &self.click_regions {
      if module_pos.within(region.region) {
        (region.func)();
      }
    }
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
  pub fn from_config(
    mut config: Config,
    width: u32,
    height: u32,
    backend: impl Backend + Send + Sync + 'static,
  ) -> Self {
    let mut window = Window::new(width, height);
    window.buffer_mut().fill(config.background);
    Bar {
      window,
      backend: Box::new(backend),
      modules: Modules::from_config(&mut config, height),
      config,
    }
  }

  pub fn window(&self) -> &Window { &self.window }
  pub fn window_mut(&mut self) -> &mut Window { &mut self.window }

  pub fn render(&mut self) {
    macro_rules! copy_module {
      ( $module:expr ) => {
        self
          .window
          .buffer_mut()
          .copy_from(Pos { x: $module.pos as i32, y: 0 }, &$module.buffer.buffer());
      };
    }

    macro_rules! copy_modules {
      ( $modules:expr ) => {
        for module in &mut $modules {
          if module.stale_content {
            module.stale_content = false;
            copy_module!(module);
          }
        }
      };
    }

    self.update_stale_positions();
    copy_modules!(self.modules.left);
    copy_modules!(self.modules.middle);
    copy_modules!(self.modules.right);

    self.backend.render(&self.window);
  }

  pub fn click(&mut self, x: u32, y: u32) {
    self.update_stale_positions();
    for module in self.modules() {
      if x >= module.pos && x <= module.pos + module.buffer.width() {
        module.on_click(Pos { x: x as i32, y: y as i32 });
        return;
      }
    }
  }

  pub fn mouse_move(&mut self, x: u32, y: u32) -> Cursor {
    self.update_stale_positions();
    for module in self.modules() {
      if x >= module.pos && x <= module.pos + module.buffer.width() {
        return Cursor::Hand;
      }
    }
    Cursor::Normal
  }

  pub fn all_modules(&self) -> impl Iterator<Item = (ModuleKey, &Module)> { self.modules.iter() }
  pub fn update_module(&mut self, key: ModuleKey) {
    let module = self.modules.by_key_mut(key);
    let background = module.module.imp().background().unwrap_or(self.config.background);
    module.buffer.fill_and_set_background(background);
    let old_width = module.buffer.width();
    let mut ctx = RenderContext::new(
      &self.config,
      module.module.imp().padding_override().unwrap_or(self.config.padding),
      &mut self.window,
      &mut module.buffer,
      &mut module.click_regions,
    );
    module.module.imp().render(&mut ctx);

    module.stale_content = true;
    if module.buffer.width() != old_width {
      module.stale_width = true;
    }
  }

  fn modules(&self) -> impl Iterator<Item = &PositionedModule> {
    self.modules.left.iter().chain(self.modules.middle.iter()).chain(self.modules.right.iter())
  }

  /// Updates all modules that have a stale width.
  fn update_stale_positions(&mut self) {
    if self.modules.left.iter().any(|m| m.stale_width) {
      let mut pos = 0;
      for module in &mut self.modules.left {
        module.pos = pos;
        module.stale_width = false;
        pos += module.width(&self.config);
      }
    }
    if self.modules.middle.iter().any(|m| m.stale_width) {
      let width: u32 = self.modules.middle.iter().map(|m| m.buffer.width()).sum();
      let mut pos = self.window.width() / 2 - width / 2;
      for module in self.modules.middle.iter_mut() {
        module.pos = pos;
        module.stale_width = false;
        pos += module.width(&self.config);
      }
    }
    if self.modules.right.iter().any(|m| m.stale_width) {
      let mut pos = self.window.width();
      for module in self.modules.right.iter_mut().rev() {
        pos -= module.width(&self.config);
        module.pos = pos;
        module.stale_width = false;
      }
    }
  }
}

impl Modules {
  /// Drains the modules from the given config.
  pub fn from_config(config: &mut crate::Config, height: u32) -> Self {
    Modules {
      left:   config
        .modules_left
        .drain(..)
        .map(|m| PositionedModule::new(m, height, config.background))
        .collect(),
      middle: config
        .modules_middle
        .drain(..)
        .map(|m| PositionedModule::new(m, height, config.background))
        .collect(),
      right:  config
        .modules_right
        .drain(..)
        .map(|m| PositionedModule::new(m, height, config.background))
        .collect(),
    }
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
  fn by_key_mut(&mut self, key: ModuleKey) -> &mut PositionedModule {
    match key {
      ModuleKey::Left(i) => &mut self.left[i as usize],
      ModuleKey::Middle(i) => &mut self.middle[i as usize],
      ModuleKey::Right(i) => &mut self.right[i as usize],
    }
  }
}
