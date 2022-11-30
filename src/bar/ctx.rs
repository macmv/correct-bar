use super::{ClickRegion, Color, DynamicBuffer, Padding, Window};
use crate::{
  config::Config,
  math::{Pos, Rect},
};

pub struct RenderContext<'a> {
  padding:       Padding,
  window:        &'a mut Window,
  buffer:        &'a mut DynamicBuffer,
  click_regions: &'a mut Vec<ClickRegion>,

  font_size:  f32,
  font_scale: f32,

  ui_scale: f64,

  pub(super) pos: u32,
}

impl<'a> RenderContext<'a> {
  pub(super) fn new(
    config: &Config,
    padding: Padding,
    window: &'a mut Window,
    buffer: &'a mut DynamicBuffer,
    click_regions: &'a mut Vec<ClickRegion>,
  ) -> Self {
    RenderContext {
      padding,
      window,
      buffer,
      click_regions,
      font_size: config.font_size,
      font_scale: 1.0,
      ui_scale: config.scale,
      pos: padding.left,
    }
  }

  /// Returns the current cursor.
  pub fn pos(&self) -> Pos { Pos { x: self.pos as i32, y: 0 } }
  /// Returns the height of the bar.
  pub fn height(&self) -> u32 { self.window.height() }

  /// Sets the multiplier for the font size. All future `draw_text` and
  /// `advance_text` calls will multiply this number into the font size they
  /// use.
  pub fn set_font_scale(&mut self, scale: f32) { self.font_scale = scale }

  /// Returns the font size multiplied by the current font scale.
  pub fn effective_font_size(&self) -> f32 { self.font_size * self.font_scale }

  /// Returns the padding on this module.
  pub fn padding(&self) -> Padding { self.padding }

  /// Returns `true` if the click regions need to be setup.
  pub fn needs_click_regions(&self) -> bool { self.click_regions.is_empty() }

  /// Calls the given `func` any time `region` is clicked. This should only be
  /// called if `needs_click_regions` is true! This will not be cleared every
  /// time `render` is called.
  pub fn add_on_click(&mut self, region: Rect, func: impl Fn() + Send + Sync + 'static) {
    self.click_regions.push(ClickRegion { region, func: Box::new(func) });
  }

  /// Advances the cursor by the given number of pixels, affected by UI scaling.
  pub fn advance_by(&mut self, pixels: u32) {
    self.advance_by_pixels((pixels as f64 * self.ui_scale) as u32);
  }

  /// Advances the cursor by the given number of pixels. This will not apply UI
  /// scaling.
  pub fn advance_by_pixels(&mut self, pixels: u32) {
    self.pos += pixels;
    if self.pos + self.padding.right > self.buffer.width() {
      self.buffer.resize(self.pos + self.padding.right);
    }
  }

  /// Advances the text drawing by the width of the given text. This can be used
  /// to add a space which is the width of some text.
  pub fn advance_text(&mut self, text: &str) -> Rect {
    let size = self.effective_font_size();
    let pos = Pos { x: self.pos as i32, y: (self.window.height() - self.height() / 5) as i32 };
    let rect = self.buffer.layout_text(self.window.font_mut(), pos, text, size);
    self.advance_by_pixels(rect.width);
    rect.with_y(0).with_height(self.window.height())
  }

  /// Draws the given text, and advances the cursor by the width of the text.
  /// Returns the rectangle of the drawn text.
  pub fn draw_text(&mut self, text: &str, color: Color) -> Rect {
    let size = self.effective_font_size();
    let pos = Pos { x: self.pos as i32, y: (self.window.height() - self.height() / 5) as i32 };
    let rect = self.buffer.draw_text(self.window.font_mut(), pos, text, size, color);
    self.advance_by_pixels(rect.width);
    rect.with_y(0).with_height(self.window.height())
  }

  /// Draws the given rectangle. This will not advance the cursor. The
  /// rectangle's position and size will be multiplied by the scale of the bar.
  /// Use [`draw_pixel_rect`](Self::draw_pixel_rect) to skip the UI scaling.
  pub fn draw_rect(&mut self, rect: Rect, color: Color) {
    self.draw_pixel_rect(rect.scaled_by(self.ui_scale), color);
  }

  /// Draws the given rectangle, without advancing the cursor or applying
  /// scaling to the rectangle.
  pub fn draw_pixel_rect(&mut self, rect: Rect, color: Color) {
    self.buffer.draw_rect(rect, color);
  }

  pub fn draw_triangle(&mut self, a: Pos, b: Pos, c: Pos, color: Color) {
    self.draw_pixel_triangle(
      a.scaled_by(self.ui_scale),
      b.scaled_by(self.ui_scale),
      c.scaled_by(self.ui_scale),
      color,
    );
  }
  pub fn draw_pixel_triangle(&mut self, a: Pos, b: Pos, c: Pos, color: Color) {
    self.buffer.draw_triangle(a, b, c, color);
  }
}
