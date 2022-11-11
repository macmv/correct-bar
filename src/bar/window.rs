use super::{Color, Pos, Rect};
use rusttype::{Font, GlyphId, Point, Scale, ScaledGlyph};
use std::collections::HashMap;

pub struct Window {
  buf:  Buffer,
  font: FontWithCache,
}

struct FontWithCache {
  font:  Font<'static>,
  cache: FontCache,
}
struct FontCache {
  glyphs: HashMap<GlyphId, AlphaBuffer>,
}

struct AlphaBuffer {
  data:   Vec<u8>,
  width:  u32,
  height: u32,
}
struct Buffer {
  data:   Vec<u8>,
  width:  u32,
  height: u32,
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

  pub fn draw_text(&mut self, pos: Pos, text: &str, color: Color) -> Rect {
    let mut width = 0;
    let mut height = 0;
    let layout = self.font.font.layout(text, Scale::uniform(48.0), Point { x: 0.0, y: 0.0 });
    for glyph in layout {
      let bounds = glyph.pixel_bounding_box().unwrap();
      let base = Pos {
        x: (pos.x as i32 + bounds.min.x) as u32,
        y: (pos.y as i32 + bounds.min.y + 20) as u32,
      };
      if bounds.max.x - pos.x as i32 > width {
        width = bounds.max.x - pos.x as i32;
      }
      if bounds.max.y - pos.y as i32 > height {
        height = bounds.max.y - pos.y as i32;
      }
      let buf = self.font.cache.render(glyph.unpositioned());
      self.buf.copy_from_alpha(base, color, buf);
    }
    Rect { pos, width: width as u32, height: height as u32 }
  }

  pub fn draw_rect(&mut self, rect: Rect, color: Color) { self.buf.draw_rect(rect, color); }
  pub fn draw_pixel(&mut self, pos: Pos, color: Color) { self.buf.draw_pixel(pos, color); }
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
    if !self.glyphs.contains_key(&glyph.id()) {
      let bounds = glyph.exact_bounding_box().unwrap();
      let mut buf = AlphaBuffer::new(bounds.width().ceil() as u32, bounds.height().ceil() as u32);
      glyph.clone().positioned(Point { x: 0.0, y: 0.0 }).draw(|x, y, coverage| {
        if coverage > 0.0 {
          buf.draw_pixel(Pos { x, y }, (coverage * 255.0) as u8);
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
    if pos.x < self.width && pos.y < self.height {
      let i = (pos.y * self.width + pos.x) as usize;
      self.data[i] = alpha;
    }
  }

  pub fn get_pixel(&self, pos: Pos) -> u8 {
    if pos.x < self.width && pos.y < self.height {
      let i = (pos.y * self.width + pos.x) as usize;
      self.data[i]
    } else {
      0
    }
  }
}

impl Buffer {
  pub fn new(width: u32, height: u32) -> Self {
    Buffer { data: vec![0; (width * height * 4) as usize], width, height }
  }

  pub fn copy_from(&mut self, pos: Pos, other: &Buffer) {
    for x in 0..other.width {
      for y in 0..other.height {
        let p = Pos { x, y };
        self.draw_pixel(p + pos, other.get_pixel(p));
      }
    }
  }
  pub fn copy_from_alpha(&mut self, pos: Pos, color: Color, other: &AlphaBuffer) {
    for x in 0..other.width {
      for y in 0..other.height {
        let p = Pos { x, y };
        let alpha = other.get_pixel(p);
        self.draw_pixel(p + pos, color.fade(self.get_pixel(p + pos), alpha));
      }
    }
  }

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

  pub fn get_pixel(&self, pos: Pos) -> Color {
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
