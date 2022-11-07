use crossbeam_channel::Receiver;
use std::time::Duration;

pub struct Module {
  imp: Box<dyn ModuleImpl>,
}

impl Module {
  pub fn imp(&self) -> &dyn ModuleImpl { &*self.imp }
  pub fn imp_mut(&mut self) -> &mut dyn ModuleImpl { &mut *self.imp }
}

impl<T> From<T> for Module
where
  T: ModuleImpl + 'static,
{
  fn from(imp: T) -> Self { Module { imp: Box::new(imp) } }
}

pub enum Updater {
  Never,
  Every(Duration),
  Channel(Receiver<()>),
}

pub trait ModuleImpl {
  fn render(&mut self) -> String;
  fn updater(&self) -> Updater;
}

impl<F> ModuleImpl for F
where
  F: FnMut() -> String,
{
  fn render(&mut self) -> String { self() }
  fn updater(&self) -> Updater { Updater::Never }
}
