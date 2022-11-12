use super::{Color, DynamicBuffer, Padding, Pos, Rect, Window};

pub struct RenderContext<'a> {
  padding: Padding,
  window:  &'a mut Window,
  buffer:  &'a mut DynamicBuffer,

  pub(super) pos: u32,
}

impl<'a> RenderContext<'a> {
  pub(super) fn new(
    padding: Padding,
    window: &'a mut Window,
    buffer: &'a mut DynamicBuffer,
  ) -> Self {
    RenderContext { padding, window, buffer, pos: padding.left }
  }

  /// Returns the current cursor.
  pub fn pos(&self) -> Pos { Pos { x: self.pos, y: 0 } }
  /// Returns the height of the bar.
  pub fn height(&self) -> u32 { self.window.height() }

  /// Advances the cursor by the given number of pixels.
  pub fn advance_by(&mut self, pixels: u32) {
    self.pos += pixels;
    if self.pos + self.padding.right > self.buffer.width() {
      self.buffer.resize(self.pos + self.padding.right);
    }
  }

  /// Advances the text drawing by the width of the given text. This can be used
  /// to add a space which is the width of some text.
  pub fn advance_text(&mut self, text: &str) -> Rect {
    let rect = self.buffer.layout_text(self.window.font_mut(), Pos { x: self.pos, y: 0 }, text);
    self.advance_by(rect.width);
    rect.with_height(self.window.height())
  }

  /// Draws the given text, and advances the cursor by the width of the text.
  /// Returns the rectangle of the drawn text.
  pub fn draw_text(&mut self, text: &str, color: Color) -> Rect {
    let rect =
      self.buffer.draw_text(self.window.font_mut(), Pos { x: self.pos, y: 0 }, text, color);
    self.advance_by(rect.width);
    rect.with_height(self.window.height())
  }

  /// Draws the given rectangle. This will not advance the cursor.
  pub fn draw_rect(&mut self, rect: Rect, color: Color) { self.buffer.draw_rect(rect, color); }
}
