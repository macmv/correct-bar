pub mod desktop;
pub mod laptop;
pub mod modules;

use correct_bar::{
  bar::{Color, Module, ModuleImpl, Padding, Updater},
  math::Rect,
};

#[derive(Clone)]
struct SepModule;

impl ModuleImpl for SepModule {
  fn padding_override(&self) -> Option<Padding> { Some(Padding::none()) }
  fn updater(&self) -> Updater { Updater::Never }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    ctx.draw_rect(Rect { pos: ctx.pos(), width: 2, height: ctx.height() }, SEP);
    ctx.advance_by(2);
  }
  fn box_clone(&self) -> Box<dyn ModuleImpl + Send + Sync> { Box::new(self.clone()) }
}

const SEP: Color = Color::from_hex(0x222222);

pub fn modules(include_bspwm: bool) -> (Vec<Module>, Vec<Module>, Vec<Module>) {
  (
    if include_bspwm {
      vec![
        modules::BSPWM::new().into(),
        SepModule.into(),
        Module::text("100%", Color { r: 100, g: 255, b: 128 }).into(),
      ]
    } else {
      vec![Module::text("100%", Color { r: 100, g: 255, b: 128 }).into()]
    },
    vec![
      Module::text("HELLO WORLD", Color { r: 255, g: 255, b: 128 }).into(),
      SepModule.into(),
      Module::text("foo and stuff", Color { r: 100, g: 255, b: 128 }).into(),
    ],
    vec![
      Module::text("mmm things", Color::from_hex(0x8866ff)).into(),
      SepModule.into(),
      modules::Cpu { primary: Color::from_hex(0xff2200), secondary: SEP }.into(),
      SepModule.into(),
      modules::Temp { primary: Color::from_hex(0xff6600), secondary: SEP }.into(),
      SepModule.into(),
      modules::Mem { primary: Color::from_hex(0xffcc00), secondary: SEP }.into(),
      SepModule.into(),
      modules::Clock { primary: Color::white(), secondary: SEP }.into(),
    ],
  )
}
