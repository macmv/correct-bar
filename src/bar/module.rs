use super::{Color, RenderContext};
use crossbeam_channel::Receiver;
use std::{fmt, time::Duration};

pub struct Module {
  imp: Box<dyn ModuleImpl + Send + Sync>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Padding {
  pub left:   u32,
  pub right:  u32,
  pub top:    u32,
  pub bottom: u32,
}
impl Padding {
  pub fn none() -> Self { Padding::default() }
}

pub struct TextModule {
  text:       &'static str,
  background: Option<Color>,
  color:      Color,
}
impl ModuleImpl for TextModule {
  fn background(&self) -> Option<Color> { self.background }
  fn render(&self, ctx: &mut RenderContext) { ctx.draw_text(self.text, self.color); }
  fn updater(&self) -> Updater { Updater::Never }
}

impl TextModule {
  pub fn with_background(mut self, background: Color) -> Self {
    self.background = Some(background);
    self
  }
}

impl fmt::Debug for Module {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.debug_struct("Module").finish() }
}

impl Module {
  pub fn imp(&self) -> &dyn ModuleImpl { &*self.imp }
  pub fn imp_mut(&mut self) -> &mut dyn ModuleImpl { &mut *self.imp }

  pub fn text(text: &'static str, color: Color) -> TextModule {
    TextModule { text, background: None, color }
  }
}

impl<T> From<T> for Module
where
  T: ModuleImpl + Send + Sync + 'static,
{
  fn from(imp: T) -> Self { Module { imp: Box::new(imp) } }
}

pub enum Updater {
  Never,
  Every(Duration),
  Channel(Receiver<()>),
}

pub trait ModuleImpl {
  fn padding_override(&self) -> Option<Padding> { None }
  fn background(&self) -> Option<Color> { None }
  fn render(&self, ctx: &mut RenderContext);
  fn updater(&self) -> Updater;
}
