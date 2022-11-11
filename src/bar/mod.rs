mod window;

pub use window::Window;

use crate::module::Module;

pub struct Bar {
  window:  Window,
  backend: Box<dyn Backend + Send + Sync>,

  modules_left:   Vec<Module>,
  modules_middle: Vec<Module>,
  modules_right:  Vec<Module>,
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
      window:         Window::new(width, height),
      backend:        Box::new(backend),
      modules_left:   vec![],
      modules_middle: vec![],
      modules_right:  vec![],
    }
  }

  pub fn window(&self) -> &Window { &self.window }
  pub fn window_mut(&mut self) -> &mut Window { &mut self.window }

  pub fn render(&mut self) { self.backend.render(&self.window); }

  pub fn all_modules(&self) -> impl Iterator<Item = (ModuleKey, &Module)> {
    self
      .modules_left
      .iter()
      .enumerate()
      .map(|(i, module)| (ModuleKey::Left(i as u32), module))
      .chain(
        self
          .modules_middle
          .iter()
          .enumerate()
          .map(|(i, module)| (ModuleKey::Middle(i as u32), module)),
      )
      .chain(
        self
          .modules_right
          .iter()
          .enumerate()
          .map(|(i, module)| (ModuleKey::Right(i as u32), module)),
      )
  }
  pub fn update_module(&self, key: ModuleKey) {
    let module = self.module(key);
    module.imp().render();
  }
  pub fn module(&self, key: ModuleKey) -> &Module {
    match key {
      ModuleKey::Left(i) => &self.modules_left[i as usize],
      ModuleKey::Middle(i) => &self.modules_middle[i as usize],
      ModuleKey::Right(i) => &self.modules_right[i as usize],
    }
  }
}
