use std::sync::Arc;

use cb_core::{Color, Drawable, RenderStore, Text, Waker};
use kurbo::{Point, Rect, Size};

pub struct Layout<'a> {
  pub(crate) store: &'a mut RenderStore,
  pub(crate) scale: f64,

  pub(crate) bounds: Rect,

  pub waker: &'a Arc<Waker>,
}

pub struct TextLayout {
  pub origin: Point,
  pub scale:  f64,
  pub layout: parley::Layout<peniko::Brush>,
}

impl TextLayout {
  pub fn empty() -> Self {
    Self { origin: Point::new(0.0, 0.0), scale: 1.0, layout: parley::Layout::new() }
  }
}

impl Layout<'_> {
  pub fn pad(&mut self, gap: f64) { self.bounds.x1 += gap; }

  pub fn layout_text<'a>(&mut self, text: impl Into<Text<'a>>, color: Color) -> TextLayout {
    self.layout_text_at(Point::new(self.bounds.width(), 8.0), text, color)
  }

  pub fn layout_text_at<'a>(
    &mut self,
    origin: Point,
    text: impl Into<Text<'a>>,
    color: Color,
  ) -> TextLayout {
    let mut layout =
      text.into().layout(&mut self.store, cb_core::encode_color(color).into(), self.scale);

    layout.break_all_lines(None);
    layout.align(None, parley::Alignment::Start, parley::AlignmentOptions::default());

    let layout = TextLayout { origin, scale: self.scale, layout };

    self.bounds = self.bounds.union(layout.bounds());

    layout
  }
}

impl Drawable for TextLayout {
  fn draw(&self, ctx: &mut cb_core::Render) {
    ctx.draw_text_layout(self.origin, &self.layout, None);
  }
}

impl TextLayout {
  pub fn bounds(&self) -> Rect { Rect::from_origin_size(self.origin, self.size()) }

  pub fn size(&self) -> Size {
    Size::new(f64::from(self.layout.full_width()), f64::from(self.layout.height())) / self.scale
  }
}
