use super::{Color, Pos, Rect};

use rusttype::{Font, Point, Scale};

pub struct Window {
  buf:  WindowBuf,
  font: Font<'static>,
}
struct WindowBuf {
  data:   Vec<u8>,
  width:  u32,
  height: u32,
}

impl Window {
  pub fn new(width: u32, height: u32) -> Self {
    // let font =
    // std::fs::read("/usr/share/fonts/TTF/icomoon-feather.ttf").unwrap();
    let font = std::fs::read("/usr/share/fonts/TTF/iosevka-regular.ttc").unwrap();
    Window {
      buf:  WindowBuf { data: vec![0; (width * height * 4) as usize], width, height },
      font: Font::try_from_vec(font).unwrap(),
    }
  }
  pub fn width(&self) -> u32 { self.buf.width }
  pub fn height(&self) -> u32 { self.buf.height }
  pub fn data(&self) -> &[u8] { &self.buf.data }

  pub fn draw_text(&mut self, pos: Pos, text: &str, color: Color) -> Rect {
    let layout = self.font.layout(text, Scale::uniform(24.0), Point { x: 0.0, y: 0.0 });
    for glyph in layout {
      println!("{:?}", glyph);
      let bounds = glyph.pixel_bounding_box().unwrap();
      let base =
        Pos { x: (pos.x as i32 + bounds.min.x) as u32, y: (pos.y as i32 + bounds.min.y) as u32 };
      glyph.draw(|x, y, coverage| {
        if coverage > 0.0 {
          self.buf.draw_pixel_alpha(base + Pos { x, y }, color, (coverage * 255.0) as u8);
        }
      });
    }
    Rect { pos, width: 0, height: 0 }
  }

  pub fn draw_rect(&mut self, rect: Rect, color: Color) { self.buf.draw_rect(rect, color); }
  pub fn draw_pixel(&mut self, pos: Pos, color: Color) { self.buf.draw_pixel(pos, color); }
}

impl WindowBuf {
  pub fn draw_rect(&mut self, rect: Rect, color: Color) {
    for x in rect.left()..rect.right() {
      for y in rect.top()..rect.bottom() {
        self.draw_pixel(Pos { x, y }, color);
      }
    }
  }

  pub fn draw_pixel(&mut self, pos: Pos, color: Color) {
    // if i + 4 <= buf.len() {
    if pos.x < self.width && pos.y < self.height {
      let i = (pos.y * self.width + pos.x) as usize * 4;
      self.data[i] = color.r;
      self.data[i + 1] = color.g;
      self.data[i + 2] = color.b;
    }
  }

  pub fn get_pixel(&mut self, pos: Pos) -> Color {
    if pos.x < self.width && pos.y < self.height {
      let i = (pos.y * self.width + pos.x) as usize * 4;
      Color { r: self.data[i], g: self.data[i + 1], b: self.data[i + 2] }
    } else {
      Color::black()
    }
  }

  pub fn draw_pixel_alpha(&mut self, pos: Pos, color: Color, alpha: u8) {
    if alpha == 255 {
      self.draw_pixel(pos, color);
    } else {
      let existing = self.get_pixel(pos);
      self.draw_pixel(pos, color.fade(existing, alpha));
    }
  }
}
