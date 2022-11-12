pub mod desktop;
pub mod laptop;

use correct_bar::bar::Module;

pub fn modules() -> (Vec<Module>, Vec<Module>, Vec<Module>) {
  (
    vec![
      Module::text("foo", correct_bar::bar::Color { r: 255, g: 255, b: 128 }).into(),
      Module::text(" | ", correct_bar::bar::Color { r: 100, g: 100, b: 100 }).into(),
      Module::text("100%", correct_bar::bar::Color { r: 100, g: 255, b: 128 }).into(),
    ],
    vec![
      Module::text("HELLO WORLD", correct_bar::bar::Color { r: 255, g: 255, b: 128 }).into(),
      Module::text(" | ", correct_bar::bar::Color { r: 100, g: 100, b: 100 }).into(),
      Module::text("foo and stuff", correct_bar::bar::Color { r: 100, g: 255, b: 128 }).into(),
    ],
    vec![
      Module::text("mmm things", correct_bar::bar::Color { r: 255, g: 100, b: 128 }).into(),
      Module::text(" | ", correct_bar::bar::Color { r: 100, g: 100, b: 100 }).into(),
      Module::text("time here", correct_bar::bar::Color { r: 100, g: 255, b: 128 }).into(),
    ],
  )
}
