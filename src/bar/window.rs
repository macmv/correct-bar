use super::Color;
use crate::math::{Pos, Rect};
use rusttype::{Font, GlyphId, Point, Scale, ScaledGlyph};
use std::{collections::HashMap, fmt};

pub struct Window {
  buf:  Buffer,
  font: FontWithCache,
}

pub struct FontWithCache {
  font:  Font<'static>,
  cache: FontCache,
}
struct FontCache {
  glyphs: HashMap<GlyphId, AlphaBuffer>,
}

pub struct AlphaBuffer {
  data:   Vec<u8>,
  width:  u32,
  height: u32,
}
pub struct Buffer {
  data:   Vec<u8>,
  width:  u32,
  height: u32,
}
/// A buffer that will resize when drawn to. This one will only resize
/// horizontally, because this bar is horizontal.
pub struct DynamicBuffer {
  buf:        Buffer,
  background: Color,
}

impl Window {
  pub fn new(width: u32, height: u32) -> Self {
    // let font =
    // std::fs::read("/usr/share/fonts/TTF/icomoon-feather.ttf").unwrap();
    Window {
      buf:  Buffer::new(width, height),
      font: FontWithCache::load("/usr/share/fonts/TTF/iosevka-regular.ttc"),
    }
  }
  pub fn width(&self) -> u32 { self.buf.width }
  pub fn height(&self) -> u32 { self.buf.height }
  pub fn data(&self) -> &[u8] { &self.buf.data }
  pub fn buffer(&self) -> &Buffer { &self.buf }
  pub fn buffer_mut(&mut self) -> &mut Buffer { &mut self.buf }

  pub fn font(&self) -> &FontWithCache { &self.font }
  pub fn font_mut(&mut self) -> &mut FontWithCache { &mut self.font }
}

impl FontWithCache {
  pub fn load(path: &str) -> Self {
    let font = std::fs::read(path).unwrap();
    FontWithCache { font: Font::try_from_vec(font).unwrap(), cache: FontCache::new() }
  }
}

impl FontCache {
  pub fn new() -> Self { FontCache { glyphs: HashMap::new() } }

  pub fn render(&mut self, glyph: &ScaledGlyph) -> &AlphaBuffer {
    // In order for anti-aliasing to work correctly, we add 1 pixel of extra space
    // around the bounding box.
    if !self.glyphs.contains_key(&glyph.id()) {
      let bounds = glyph.exact_bounding_box().unwrap();
      let mut buf =
        AlphaBuffer::new(bounds.width().ceil() as u32 + 2, bounds.height().ceil() as u32 + 2);
      glyph.clone().positioned(Point { x: 0.0, y: 0.0 }).draw(|x, y, coverage| {
        if coverage > 0.0 {
          buf.draw_pixel(Pos { x: x as i32 + 1, y: y as i32 + 1 }, (coverage * 255.0) as u8);
        }
      });
      self.glyphs.insert(glyph.id(), buf);
    }
    self.glyphs.get(&glyph.id()).unwrap()
  }
}

impl AlphaBuffer {
  pub fn new(width: u32, height: u32) -> Self {
    AlphaBuffer { data: vec![0; (width * height) as usize], width, height }
  }

  pub fn draw_pixel(&mut self, pos: Pos, alpha: u8) {
    if pos.x > 0 && pos.y > 0 && (pos.x as u32) < self.width && (pos.y as u32) < self.height {
      let i = (pos.y as u32 * self.width + pos.x as u32) as usize;
      self.data[i] = alpha;
    }
  }

  pub fn get_pixel(&self, pos: Pos) -> u8 {
    if pos.x > 0 && pos.y > 0 && (pos.x as u32) < self.width && (pos.y as u32) < self.height {
      let i = (pos.y as u32 * self.width + pos.x as u32) as usize;
      self.data[i]
    } else {
      0
    }
  }
}

impl fmt::Debug for DynamicBuffer {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("DynamicBuffer")
      .field("width", &self.buf.width)
      .field("height", &self.buf.height)
      .finish()
  }
}

impl DynamicBuffer {
  pub fn new(height: u32, background: Color) -> Self {
    DynamicBuffer { buf: Buffer::new(0, height), background }
  }

  #[inline]
  pub fn width(&self) -> u32 { self.buf.width }
  #[inline]
  pub fn height(&self) -> u32 { self.buf.height }
  pub fn buffer(&self) -> &Buffer { &self.buf }
  pub fn buffer_mut(&mut self) -> &mut Buffer { &mut self.buf }

  pub fn fill_and_set_background(&mut self, color: Color) {
    self.background = color;
    self.buf.fill(color);
  }

  pub fn resize(&mut self, width: u32) {
    if width > self.buf.width {
      let mut buffer = Buffer::new(width, self.buf.height);
      buffer.copy_from(Pos { x: 0, y: 0 }, &self.buf);
      buffer.draw_rect(
        Rect {
          pos:    Pos { x: self.buf.width as i32, y: 0 },
          width:  width - self.buf.width,
          height: self.buf.height,
        },
        self.background,
      );
      std::mem::swap(&mut self.buf, &mut buffer);
    }
  }

  pub fn layout_text(
    &mut self,
    font: &mut FontWithCache,
    pos: Pos,
    text: &str,
    font_size: f32,
  ) -> Rect {
    let scale = Scale::uniform(font_size);
    let mut last_glyph = None;
    let mut caret = 0.0;
    for c in text.chars() {
      let glyph = font.font.glyph(c).scaled(scale);
      if let Some(last) = last_glyph {
        caret += font.font.pair_kerning(scale, last, glyph.id());
      }
      let glyph = glyph.positioned(Point { x: caret, y: 0.0 });
      caret += glyph.unpositioned().h_metrics().advance_width;
      last_glyph = Some(glyph.id());
    }

    Rect { pos, width: caret.ceil() as u32, height: 0 }
  }

  pub fn draw_text(
    &mut self,
    font: &mut FontWithCache,
    pos: Pos,
    text: &str,
    font_size: f32,
    color: Color,
  ) -> Rect {
    let rect = self.layout_text(font, pos, text, font_size);

    self.resize(pos.x as u32 + rect.width);

    let scale = Scale::uniform(font_size);
    let mut last_glyph = None;
    let mut caret = 0.0;
    for c in text.chars() {
      let glyph = font.font.glyph(c).scaled(scale);
      if let Some(last) = last_glyph {
        caret += font.font.pair_kerning(scale, last, glyph.id());
      }
      let glyph = glyph.positioned(Point { x: caret, y: 0.0 });
      caret += glyph.unpositioned().h_metrics().advance_width;
      last_glyph = Some(glyph.id());

      if let Some(bounds) = glyph.pixel_bounding_box() {
        let base = Pos { x: pos.x + bounds.min.x, y: pos.y + bounds.min.y };
        let buf = font.cache.render(glyph.unpositioned());
        self.buf.copy_from_alpha(base, color, buf);
      }
    }

    Rect { pos, width: caret as u32, height: 0 }
  }
  pub fn draw_rect(&mut self, rect: Rect, color: Color) {
    if rect.right() < 0 {
      return;
    }
    self.resize(rect.right() as u32);
    self.buf.draw_rect(rect, color);
  }
}

impl Buffer {
  pub fn new(width: u32, height: u32) -> Self {
    Buffer { data: vec![0; (width * height * 4) as usize], width, height }
  }

  pub fn copy_from(&mut self, pos: Pos, other: &Buffer) {
    for x in 0..other.width {
      for y in 0..other.height {
        let p = Pos { x: x as i32, y: y as i32 };
        self.draw_pixel(p + pos, other.get_pixel(p));
      }
    }
  }
  pub fn copy_from_alpha(&mut self, pos: Pos, color: Color, other: &AlphaBuffer) {
    for x in 0..other.width {
      for y in 0..other.height {
        let p = Pos { x: x as i32, y: y as i32 };
        let alpha = other.get_pixel(p);
        self.draw_pixel(p + pos, color.fade(self.get_pixel(p + pos), alpha));
      }
    }
  }

  pub fn fill(&mut self, color: Color) {
    for y in 0..self.height {
      for x in 0..self.width {
        self.draw_pixel(Pos { x: x as i32, y: y as i32 }, color);
      }
    }
  }
  pub fn draw_rect(&mut self, rect: Rect, color: Color) {
    for y in rect.top()..rect.bottom() {
      for x in rect.left()..rect.right() {
        self.draw_pixel(Pos { x, y }, color);
      }
    }
  }

  pub fn draw_pixel(&mut self, pos: Pos, color: Color) {
    // if i + 4 <= buf.len() {
    if pos.x > 0 && pos.y > 0 && (pos.x as u32) < self.width && (pos.y as u32) < self.height {
      let i = (pos.y as u32 * self.width + pos.x as u32) as usize * 4;
      self.data[i] = color.b;
      self.data[i + 1] = color.g;
      self.data[i + 2] = color.r;
    }
  }

  pub fn get_pixel(&self, pos: Pos) -> Color {
    if pos.x > 0 && pos.y > 0 && (pos.x as u32) < self.width && (pos.y as u32) < self.height {
      let i = (pos.y as u32 * self.width + pos.x as u32) as usize * 4;
      Color { b: self.data[i], g: self.data[i + 1], r: self.data[i + 2] }
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
