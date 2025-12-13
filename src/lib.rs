mod backend;
pub mod bar;
pub mod config;
pub mod math;

use bar::Updater;
use cb_common::BarId;
use config::Config;

use crate::bar::Module;

struct App {
  left_modules:   Vec<Module>,
  center_modules: Vec<Module>,
  right_modules:  Vec<Module>,

  render: cb_core::RenderStore,
}

impl cb_common::App for App {
  type Config = Config;

  fn new(config: Config, device: &wgpu::Device) -> Self {
    App {
      left_modules:   config.modules_left,
      center_modules: config.modules_middle,
      right_modules:  config.modules_right,

      render: cb_core::RenderStore::new(device),
    }
  }

  fn create_bar(
    &mut self,
    id: BarId,
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    scale: f32,
    width: u32,
    height: u32,
  ) {
    self.render.create_bar(id, device, format, scale, width, height);
  }

  fn draw(
    &mut self,
    id: BarId,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    output: &wgpu::Texture,
  ) {
    if let Some(mut render) = self.render.for_bar(id) {
      render.draw(device, queue, output);
    }
  }
}

pub fn run(config: Config) {
  cb_backend_wayland::setup::<App>(config.clone());

  let bars = backend::wayland::setup(config);

  let mut all_modules = vec![];
  let mut sleep_duration = None;
  let mut sleep_modules = vec![];
  let mut channel_modules = vec![];

  {
    let bar = &bars[0];
    let b = bar.lock();

    for (key, module) in b.all_modules() {
      all_modules.push(key);
      match module.imp().updater() {
        Updater::Never => {}
        Updater::Every(duration) => {
          if let Some(dur) = sleep_duration {
            if duration < dur {
              sleep_duration = Some(duration);
            }
          } else {
            sleep_duration = Some(duration);
          }
          sleep_modules.push(key);
        }
        Updater::Channel(recv) => {
          channel_modules.push((key, recv));
        }
      }
    }
  }
  for bar in &bars {
    let mut b = bar.lock();

    for key in &all_modules {
      b.update_module(*key);
    }
    b.render();
  }

  if !channel_modules.is_empty() {
    let bars = bars.clone();
    std::thread::spawn(move || loop {
      let mut sel = crossbeam_channel::Select::new();
      channel_modules.iter().for_each(|(_, chan)| {
        sel.recv(chan);
      });
      let idx = sel.ready();
      while channel_modules[idx].1.try_recv().is_ok() {}
      for bar in &bars {
        let mut b = bar.lock();
        b.update_module(channel_modules[idx].0);
        b.render();
      }
    });
  }

  if let Some(dur) = sleep_duration {
    loop {
      std::thread::sleep(dur);
      for bar in &bars {
        let mut b = bar.lock();
        for module in &sleep_modules {
          b.update_module(*module);
        }
        b.render();
      }
    }
  } else {
    // Sit around and do nothing
    loop {
      std::thread::park();
    }
  }
}
