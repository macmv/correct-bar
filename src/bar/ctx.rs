use super::{Color, Pos, Rect, Window};

pub struct RenderContext<'a> {
  window: &'a mut Window,

  pos: Pos,
}

impl<'a> RenderContext<'a> {
  pub(super) fn new(window: &'a mut Window, pos: Pos) -> Self { RenderContext { window, pos } }

  pub fn draw_text(&mut self, text: &str, color: Color) {
    self.window.draw_rect(Rect { pos: self.pos, width: 20, height: 20 }, color);
    let _ = (text, color);
  }
}
