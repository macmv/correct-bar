use correct_bar::bar::{Color, ModuleImpl, Updater};
use crossbeam_channel::Receiver;

#[derive(Clone)]
pub struct Pulse {
  color: Color,
  recv:  Receiver<()>,
}

impl Pulse {
  pub fn new(color: Color) -> Self {
    let (tx, rx) = crossbeam_channel::bounded(16);

    std::thread::spawn(move || loop {
      // TODO: Watch for pulse events.
      std::thread::sleep(std::time::Duration::from_secs(1));
      tx.send(()).unwrap();
    });

    Pulse { color, recv: rx }
  }
}

impl ModuleImpl for Pulse {
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    ctx.draw_text("fooo", self.color);
    ctx.draw_text("%", self.color);
  }
  fn updater(&self) -> Updater { Updater::Channel(self.recv.clone()) }
  fn box_clone(&self) -> Box<dyn ModuleImpl + Send + Sync> { Box::new(self.clone()) }
}
