use super::{Color, Pos, Window};

pub struct RenderContext<'a> {
  window: &'a mut Window,

  pub(super) pos:   Pos,
  pub(super) width: u32,
}

impl<'a> RenderContext<'a> {
  pub(super) fn new(window: &'a mut Window, pos: Pos) -> Self {
    RenderContext { window, pos, width: 0 }
  }

  pub fn draw_text(&mut self, text: &str, color: Color) {
    let rect = self.window.draw_text(self.pos, text, color);
    if self.width == 0 {
      self.width = rect.width;
    }
  }
}
