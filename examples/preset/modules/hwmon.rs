//! Reads values in `/sys/class/hwmon` to get the temperature of various
//! devices.

use correct_bar::bar::{Color, ModuleImpl, Updater};
use std::{
  cell::RefCell,
  fs,
  fs::File,
  io::{Read, Seek, SeekFrom},
  path::Path,
  time::Duration,
};

#[derive(Debug)]
struct Monitor {
  // The value of the `name` file.
  name: String,

  temps: Vec<(String, File)>,
}

impl Monitor {
  pub fn new(path: &Path) -> Self {
    let name = fs::read_to_string(path.join("name")).unwrap().trim().to_string();
    let mut temps = vec![];
    for entry in fs::read_dir(path).unwrap() {
      let p = entry.unwrap();
      // Every `temp` entry has two files: `temp1_input` and `temp1_label`, where `1`
      // is the number of this sensor. So the easiest way to find these is to just
      // look for the `temp1_label` files, then get the other files from that.

      let path = p.path();
      let name = path.file_name().unwrap().to_str().unwrap();
      if name.starts_with("temp") && name.ends_with("label") {
        let input_path = path.parent().unwrap().join(name.replace("label", "input"));
        temps.push((
          name.to_string(),
          fs::read_to_string(path).unwrap().trim().to_string(),
          File::open(input_path).unwrap(),
        ));
      }
    }
    temps.sort_unstable_by(|(a, _, _), (b, _, _)| a.cmp(b));
    Monitor { name, temps: temps.into_iter().map(|(_, label, file)| (label, file)).collect() }
  }

  pub fn find_all() -> Vec<Monitor> {
    fs::read_dir("/sys/class/hwmon")
      .unwrap()
      .map(|dir| Monitor::new(&dir.unwrap().path()))
      .collect()
  }

  pub fn read(&mut self) -> Vec<(String, f32)> {
    self
      .temps
      .iter_mut()
      .map(|(name, file)| {
        file.seek(SeekFrom::Start(0)).unwrap();

        let mut buf = vec![];
        file.read_to_end(&mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        let milli_degress = s.trim().parse::<u32>().unwrap();

        (name.to_string(), milli_degress as f32 / 1000.0)
      })
      .collect()
  }
}

thread_local! {
  static MONITORS: RefCell<Option<Vec<Monitor>>> = RefCell::new(None);
}

pub struct Temp {
  pub primary:   Color,
  pub secondary: Color,
}
impl ModuleImpl for Temp {
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    MONITORS.with(|s| {
      let mut monitors = s.borrow_mut();
      if monitors.is_none() {
        *monitors = Some(Monitor::find_all());
      }
      let m = monitors.as_mut().unwrap();

      if let Some(mon) = m.iter_mut().filter(|m| m.name == "k10temp").next() {
        let results = mon.read();
        ctx.draw_text(&format!("{:>2.00}", results[0].1), self.primary);
        ctx.draw_text("°", self.secondary);
      }

      /*
      for c in state.components {
        if c.label() == "k10temp Tccd1" {
          ctx.draw_text(&format!("{:>2.00}", c.temperature()), self.primary);
          ctx.draw_text("°", self.secondary);
          break;
        }
      }
      */
    });
  }
}
