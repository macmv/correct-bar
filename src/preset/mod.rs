pub mod desktop;
pub mod laptop;

use correct_bar::module::{Module, Section};

pub fn modules() -> (Vec<Module>, Vec<Module>, Vec<Module>) {
  (
    vec![
      Module::constant(&[Section::new("foo").with_color(0x333333)]),
      Module::constant(&[Section::new("|")]),
    ],
    vec![],
    vec![],
  )
}
