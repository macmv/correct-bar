mod ctx;
mod module;
mod window;

pub use ctx::RenderContext;
pub use module::{Module, Updater};
pub use window::Window;

pub struct Bar {
  window:  Window,
  backend: Box<dyn Backend + Send + Sync>,

  pub modules: Modules,
}

pub struct Modules {
  left:   Vec<Module>,
  middle: Vec<Module>,
  right:  Vec<Module>,
}

#[derive(Clone, Copy, Debug)]
pub struct Color {
  pub r: u8,
  pub g: u8,
  pub b: u8,
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
    let mut ctx = RenderContext::new(&mut self.window);
    let module = self.modules.by_key(key);
    module.imp().render(&mut ctx);
  }
}

impl Modules {
  pub fn empty() -> Self { Modules { left: vec![], middle: vec![], right: vec![] } }
  pub fn set_from_config(&mut self, config: crate::Config) {
    self.left = config.modules_left;
    self.middle = config.modules_middle;
    self.right = config.modules_right;
  }
  pub fn iter(&self) -> impl Iterator<Item = (ModuleKey, &Module)> {
    self
      .left
      .iter()
      .enumerate()
      .map(|(i, module)| (ModuleKey::Left(i as u32), module))
      .chain(
        self.middle.iter().enumerate().map(|(i, module)| (ModuleKey::Middle(i as u32), module)),
      )
      .chain(self.right.iter().enumerate().map(|(i, module)| (ModuleKey::Right(i as u32), module)))
  }
  pub fn by_key(&self, key: ModuleKey) -> &Module {
    match key {
      ModuleKey::Left(i) => &self.left[i as usize],
      ModuleKey::Middle(i) => &self.middle[i as usize],
      ModuleKey::Right(i) => &self.right[i as usize],
    }
  }
}
