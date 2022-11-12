mod backend;
pub mod bar;
pub mod config;

use bar::Updater;
use config::Config;

pub fn run(config: Config) {
  let bar = backend::x11::setup(&config);

  let mut sleep_duration = None;
  let mut sleep_modules = vec![];
  let mut channel_modules = vec![];

  {
    let mut b = bar.lock();
    b.modules.set_from_config(config);

    let mut all_modules = vec![];
    for (key, module) in b.all_modules() {
      all_modules.push(key);
      match module.imp().updater() {
        Updater::Never => {}
        Updater::Every(duration) => {
          if let Some(dur) = sleep_duration {
            if duration < dur {
              sleep_duration = Some(duration);
            }
          }
          sleep_modules.push(key);
        }
        Updater::Channel(recv) => {
          channel_modules.push((key, recv));
        }
      }
    }
    for key in all_modules {
      b.update_module(key);
    }
    b.render();
  }

  if !channel_modules.is_empty() {
    let bar = bar.clone();
    std::thread::spawn(move || loop {
      let mut sel = crossbeam_channel::Select::new();
      channel_modules.iter().for_each(|(_, chan)| {
        sel.recv(chan);
      });
      let idx = sel.ready();
      let mut b = bar.lock();
      b.update_module(channel_modules[idx].0);
      b.render();
    });
  }

  if let Some(dur) = sleep_duration {
    loop {
      std::thread::sleep(dur);
      let mut b = bar.lock();
      for module in &sleep_modules {
        b.update_module(*module);
      }
      b.render();
    }
  } else {
    // Sit around and do nothing
    loop {
      std::thread::park();
    }
  }
}
