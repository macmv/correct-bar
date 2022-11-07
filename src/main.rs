use correct_bar::{
  config::{Config, WindowConfig},
  module::{Module, Section},
};

fn main() {
  let config = Config {
    window: WindowConfig {
      margin_top: 40,
      margin_left: 10,
      margin_right: 10,
      margin_bottom: 10,
      width: 1920 - 20,
      height: 50,
      ..Default::default()
    },
    modules_left: vec![
      Module::constant(&[Section::new("foo").with_color(0x333333)]),
      Module::constant(&[Section::new("|")]),
    ],
    ..Default::default()
  };
  correct_bar::run(config)
}
