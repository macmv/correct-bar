use super::{Color, DynamicBuffer, Pos, Rect, Window};

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

  /// Advances the text drawing by the width of the given text. This can be used
  /// to add a space which is the width of some text.
  pub fn advance_text(&mut self, text: &str) -> Rect {
    let rect = self.buffer.layout_text(self.window.font_mut(), Pos { x: self.pos, y: 0 }, text);
    self.pos += rect.width;
    rect.with_height(self.window.height())
  }

  /// Draws the given text, and advances the cursor by the width of the text.
  /// Returns the rectangle of the drawn text.
  pub fn draw_text(&mut self, text: &str, color: Color) -> Rect {
    let rect =
      self.buffer.draw_text(self.window.font_mut(), Pos { x: self.pos, y: 0 }, text, color);
    self.pos += rect.width;
    let new_width = self.width + rect.width;
    if new_width > self.width {
      self.width = new_width;
    }
    rect.with_height(self.window.height())
  }

  /// Draws the given rectangle. This will not advance the cursor.
  pub fn draw_rect(&mut self, rect: Rect, color: Color) { self.buffer.draw_rect(rect, color); }
}
