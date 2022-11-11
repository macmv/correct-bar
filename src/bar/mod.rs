mod window;

pub use window::Window;

pub struct Bar {
  window:  Window,
  backend: Box<dyn Backend + Send + Sync>,
}

#[derive(Clone, Copy)]
pub struct Color {
  r: u8,
  g: u8,
  b: u8,
}

pub trait Backend {
  fn render(&self, window: &Window);
}

impl Bar {
  pub fn new(width: u32, height: u32, backend: impl Backend + Send + Sync + 'static) -> Self {
    Bar { window: Window::new(width, height), backend: Box::new(backend) }
  }

  pub fn window(&self) -> &Window { &self.window }

  pub fn render(&mut self) { self.backend.render(&self.window); }
}
