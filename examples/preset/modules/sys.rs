use correct_bar::bar::{Color, ModuleImpl, Updater};
use std::{
  cell::RefCell,
  collections::HashMap,
  fs::File,
  io::{BufRead, Seek, SeekFrom},
  time::{Duration, Instant},
};

thread_local! {
  static SYS: RefCell<Option<SystemInfo>> = RefCell::new(None);
}

struct SystemInfo {
  last_update: Instant,
  last_state:  Option<ProcState>,
  curr_state:  ProcState,

  files: Files,
}

#[derive(Clone, Debug)]
struct ProcState {
  meminfo: Meminfo,
  stat:    Stat,
}
struct Files {
  stat:    File,
  meminfo: File,
}

/// The contents of `/proc/meminfo`
#[derive(Clone, Debug)]
struct Meminfo {
  mem_total_kb: u32,
  mem_free_kb:  u32,
  mem_avail_kb: u32,
}
/// The contents of `/proc/stat`
#[derive(Clone, Debug)]
struct Stat {
  average: CpuStat,
  cpus:    Vec<CpuStat>,
}
#[derive(Clone, Debug)]
struct CpuStat {
  user:       u32,
  nice:       u32,
  system:     u32,
  idle:       u32,
  iowait:     u32,
  irq:        u32,
  softirq:    u32,
  steal:      u32,
  guest:      u32,
  guest_nice: u32,
}

/// The state of the system. This stores a delta between the state some time
/// ago, and the current state.
#[derive(Clone, Debug)]
pub struct State {
  pub memory: MemoryState,
  pub cpu:    CpuState,
}

#[derive(Clone, Debug)]
pub struct MemoryState {
  pub total_mbs: u32,
  pub used_mbs:  u32,
  pub free_mbs:  u32,
}

#[derive(Clone, Debug)]
pub struct CpuState {
  pub usage: f64,
}

impl Files {
  pub fn new() -> Self {
    Files {
      meminfo: File::open("/proc/meminfo").unwrap(),
      stat:    File::open("/proc/stat").unwrap(),
    }
  }

  pub fn read_state(&mut self) -> ProcState {
    ProcState {
      meminfo: Meminfo::read_from(&mut self.meminfo),
      stat:    Stat::read_from(&mut self.stat),
    }
  }
}

impl Meminfo {
  fn read_from(file: &mut File) -> Self {
    file.seek(SeekFrom::Start(0));
    let mut reader = std::io::BufReader::new(file);
    let values = reader
      .lines()
      .flat_map(|line| {
        let mut sections = line.unwrap().split(":");
        let key = sections.next().unwrap();
        let value = sections.next().unwrap().trim();
        value.strip_suffix(" kB").map(|val| (key.to_string(), val.parse::<u32>().unwrap()))
      })
      .collect::<HashMap<String, u32>>();
    Meminfo {
      mem_total_kb: values["MemTotal"],
      mem_free_kb:  values["MemFree"],
      mem_avail_kb: values["MemAvail"],
    }
  }
}
impl Stat {
  fn read_from(file: &mut File) -> Self {
    file.seek(SeekFrom::Start(0));
    let mut reader = std::io::BufReader::new(file);
    let mut lines = reader.lines();

    let first_line = lines.next().unwrap();
    let average = CpuStat::parse(first_line);

    let mut cpus = vec![];
    for line in lines {
      if line.starts_with("cpu") {
        cpus.push(CpuStat::parse(line));
      } else {
        break;
      }
    }
  }
}

impl CpuStat {
  fn parse_from(s: &str) -> Self {
    let mut sections = s.split(" ");
    let _ = sections.next().unwrap(); // this is the cpu/cpu0/cpu1 section

    CpuStat {
      user:       sections.next().unwrap().parse::<u32>().unwrap(),
      nice:       sections.next().unwrap().parse::<u32>().unwrap(),
      system:     sections.next().unwrap().parse::<u32>().unwrap(),
      idle:       sections.next().unwrap().parse::<u32>().unwrap(),
      iowait:     sections.next().unwrap().parse::<u32>().unwrap(),
      irq:        sections.next().unwrap().parse::<u32>().unwrap(),
      softirq:    sections.next().unwrap().parse::<u32>().unwrap(),
      steal:      sections.next().unwrap().parse::<u32>().unwrap(),
      guest:      sections.next().unwrap().parse::<u32>().unwrap(),
      guest_nice: sections.next().unwrap().parse::<u32>().unwrap(),
    }
  }
}

impl SystemInfo {
  pub fn new() -> SystemInfo {
    let files = Files::new();
    let curr_state = files.read_state();
    SystemInfo { last_update: Instant::now(), last_state: None, curr_state, files }
  }

  fn refresh(&mut self) {
    let now = Instant::now();
    if now.duration_since(self.last_update) > Duration::from_secs(1) {
      self.update();
      self.last_update = now;
    }
  }

  fn update(&mut self) {
    let new_state = self.files.read_state();
    self.last_state = Some(std::mem::replace(&mut self.curr_state, new_state));
  }

  pub fn state(&self) -> State {}
}

pub struct Temp {
  pub primary:   Color,
  pub secondary: Color,
}
impl ModuleImpl for Temp {
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    SYS.with(|s| {
      let mut sys = s.borrow_mut();
      if sys.is_none() {
        *sys = Some(SystemInfo::new());
      }
      let mut sys = sys.unwrap();
      sys.refresh();
      let state = sys.state();

      let temp = 50.0;
      ctx.draw_text(&format!("{:>2.00}", temp), self.primary);
      ctx.draw_text("°", self.secondary);

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

pub struct Cpu {
  pub primary:   Color,
  pub secondary: Color,
}
impl ModuleImpl for Cpu {
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    SYS.with(|s| {
      let mut sys = s.borrow_mut();
      if sys.is_none() {
        *sys = Some(SystemInfo::new());
      }
      let mut sys = sys.unwrap();
      sys.refresh();
      let state = sys.state();

      ctx.draw_text(
        &format!(
          "{:>2.00}",
          state.cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / state.cpus.len() as f32,
        ),
        self.primary,
      );
      ctx.draw_text("%", self.secondary);
    });
  }
}

pub struct Mem {
  pub primary:   Color,
  pub secondary: Color,
}
impl ModuleImpl for Mem {
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    SYS.with(|s| {
      let mut sys = s.borrow_mut();
      if sys.is_none() {
        *sys = Some(SystemInfo::new());
      }
      let mut sys = sys.unwrap();
      sys.refresh();
      let state = sys.state();

      ctx.draw_text(
        &format!("{:>5.02}G", state.used_memory as f64 / (1024 * 1024 * 1024) as f64),
        self.primary,
      );
      ctx.draw_text(" / ", self.secondary);
      ctx.draw_text(
        &format!("{:>5.02}G", state.total_memory as f64 / (1024 * 1024 * 1024) as f64),
        self.primary,
      );
    });
  }
}
