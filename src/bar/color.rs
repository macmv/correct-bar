#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color {
  pub r: u8,
  pub g: u8,
  pub b: u8,
}

impl Color {
  pub const fn black() -> Self { Color { r: 0, g: 0, b: 0 } }
  pub const fn white() -> Self { Color { r: 255, g: 255, b: 255 } }
  pub const fn gray(v: u8) -> Self { Color { r: v, g: v, b: v } }
  pub const fn from_hex(hex: u32) -> Self {
    Color { r: (hex >> 16) as u8, g: (hex >> 8) as u8, b: hex as u8 }
  }

  /// Returns this color faded with the background. An alpha of `255` will
  /// return `self`, and an alpha of `0` will return `background`.
  pub fn fade(self, background: Color, alpha: u8) -> Color {
    fn fade(foreground: u8, background: u8, alpha: u8) -> u8 {
      if foreground > background {
        background + ((foreground - background) as u16 * alpha as u16 / 255) as u8
      } else {
        foreground + ((background - foreground) as u16 * (255 - alpha) as u16 / 255) as u8
      }
    }

    Color {
      r: fade(self.r, background.r, alpha),
      g: fade(self.g, background.g, alpha),
      b: fade(self.b, background.b, alpha),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_hex() {
    assert_eq!(Color::from_hex(0xff8800), Color { r: 0xff, g: 0x88, b: 0x00 });
  }

  #[test]
  fn fade() {
    assert_eq!(Color::white().fade(Color::black(), 128), Color::gray(128));
    assert_eq!(Color::white().fade(Color::black(), 255), Color::gray(255));
    assert_eq!(Color::black().fade(Color::white(), 255), Color::gray(0));
    assert_eq!(Color::black().fade(Color::white(), 128), Color::gray(127));
    assert_eq!(Color::black().fade(Color::white(), 0), Color::gray(255));
  }
}
