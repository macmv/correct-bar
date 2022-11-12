use super::{Color, DynamicBuffer, Pos, Window};

pub struct RenderContext<'a> {
  window: &'a mut Window,
  buffer: &'a mut DynamicBuffer,

  pub(super) pos:   u32,
  pub(super) width: u32,
}

impl<'a> RenderContext<'a> {
  pub(super) fn new(window: &'a mut Window, buffer: &'a mut DynamicBuffer) -> Self {
    RenderContext { window, buffer, pos: 0, width: 0 }
  }

  pub fn draw_text(&mut self, text: &str, color: Color) {
    let rect =
      self.buffer.draw_text(self.window.font_mut(), Pos { x: self.pos, y: 0 }, text, color);
    self.pos += rect.width;
    let new_width = self.width + rect.width;
    if new_width > self.width {
      self.width = new_width;
    }
  }
}
