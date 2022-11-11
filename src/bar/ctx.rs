use super::{Color, Window};
use crate::module::Module;

pub struct RenderContext<'a> {
  window: &'a Window,
  module: &'a Module,
}

impl<'a> RenderContext<'a> {
  pub(super) fn new(window: &'a Window, module: &'a Module) -> Self {
    RenderContext { window, module }
  }

  pub fn draw_text(&mut self, text: &str, color: Color) { let _ = (text, color); }
}
