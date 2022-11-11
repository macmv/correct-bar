use super::{Color, Window};

pub struct RenderContext<'a> {
  window: &'a mut Window,
}

impl<'a> RenderContext<'a> {
  pub(super) fn new(window: &'a mut Window) -> Self { RenderContext { window } }

  pub fn draw_text(&mut self, text: &str, color: Color) {
    self.window.draw_rect(20, 20, 20, 20, color);
    let _ = (text, color);
  }
}
