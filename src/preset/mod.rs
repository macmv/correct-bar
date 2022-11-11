pub mod desktop;
pub mod laptop;

use correct_bar::bar::Module;

pub fn modules() -> (Vec<Module>, Vec<Module>, Vec<Module>) {
  (
    vec![
      Module::text("foo", correct_bar::bar::Color { r: 255, g: 255, b: 128 }),
      // Module::text("|", correct_bar::bar::Color { r: 100, g: 100, b: 100 }),
    ],
    vec![],
    vec![],
  )
}
