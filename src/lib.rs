mod backend;
pub mod bar;
pub mod config;
pub mod math;

use bar::Updater;
use config::Config;

pub fn run(config: Config) {
  let bars = backend::x11::setup(config);

  let mut sleep_duration = None;
  let mut sleep_modules = vec![];
  let mut channel_modules = vec![];

  for (bar_index, bar) in bars.iter().enumerate() {
    let mut b = bar.lock();

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
          } else {
            sleep_duration = Some(duration);
          }
          sleep_modules.push(key);
        }
        Updater::Channel(recv) => {
          channel_modules.push((key, bar_index, recv));
        }
      }
    }
    for key in all_modules {
      b.update_module(key);
    }
    b.render();
  }

  if !channel_modules.is_empty() {
    let bars = bars.clone();
    std::thread::spawn(move || loop {
      let mut sel = crossbeam_channel::Select::new();
      channel_modules.iter().for_each(|(_, _, chan)| {
        sel.recv(chan);
      });
      let idx = sel.ready();
      while let Ok(_) = channel_modules[idx].2.try_recv() {}
      let mut b = bars[channel_modules[idx].1].lock();
      b.update_module(channel_modules[idx].0);
      b.render();
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
