use cb_core::{Color, Drawable, RenderStore, Text};
use kurbo::{Point, Rect, Size};

pub struct Layout<'a> {
  pub(crate) store: &'a mut RenderStore,
  pub(crate) scale: f64,

  pub(crate) bounds: Rect,
}

pub struct TextLayout {
  origin: Point,
  layout: parley::Layout<peniko::Brush>,
}

impl Layout<'_> {
  pub fn layout_text<'a>(
    &mut self,
    origin: Point,
    text: impl Into<Text<'a>>,
    color: Color,
  ) -> TextLayout {
    let mut layout =
      text.into().layout(&mut self.store, cb_core::encode_color(color).into(), self.scale);

    layout.break_all_lines(None);
    layout.align(None, parley::Alignment::Start, parley::AlignmentOptions::default());

    let layout = TextLayout { origin, layout };

    self.bounds = self.bounds.union(Rect::from_origin_size(origin, layout.size()));

    layout
  }
}

impl Drawable for TextLayout {
  fn draw(&self, ctx: &mut cb_core::Render) { ctx.draw_text_layout(self.origin, &self.layout); }
}

impl TextLayout {
  pub fn size(&self) -> Size {
    Size::new(f64::from(self.layout.full_width()), f64::from(self.layout.height()))
  }
}
