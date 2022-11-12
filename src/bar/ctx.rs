use super::{Color, DynamicBuffer, Pos, Window};

pub struct RenderContext<'a> {
  window: &'a mut Window,
  buffer: &'a mut DynamicBuffer,

  pub(super) width: u32,
}

impl<'a> RenderContext<'a> {
  pub(super) fn new(window: &'a mut Window, buffer: &'a mut DynamicBuffer, pos: Pos) -> Self {
    RenderContext { window, buffer, width: 0 }
  }

  pub fn draw_text(&mut self, text: &str, color: Color) {
    let rect = self.buffer.draw_text(self.window.font_mut(), Pos { x: 0, y: 0 }, text, color);
    if self.width == 0 {
      self.width = rect.width;
    }
  }
}
