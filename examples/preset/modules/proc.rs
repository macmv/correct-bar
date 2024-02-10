//! Reads from /proc to get the current memory and cpu usage.

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
  /// The time this state was recorded.
  time: Instant,

  meminfo: Meminfo,
  stat:    Stat,
}
struct Files {
  stat:    File,
  meminfo: File,
}

/// The contents of `/proc/meminfo`
#[derive(Clone, Debug)]
#[allow(unused)]
struct Meminfo {
  mem_total_kb: u64,
  mem_free_kb:  u64,
  mem_avail_kb: u64,
}
/// The contents of `/proc/stat`
#[derive(Clone, Debug)]
struct Stat {
  average: CpuStat,
  cpus:    Vec<CpuStat>,
}
#[derive(Clone, Debug)]
#[allow(unused)]
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
#[derive(Clone, Debug, Default)]
pub struct State {
  pub memory: MemoryState,
  pub cpu:    CpuState,
}

#[derive(Clone, Debug, Default)]
pub struct MemoryState {
  pub total_mb: u64,
  pub used_mb:  u64,
  pub avail_mb: u64,
}

#[derive(Clone, Debug, Default)]
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
      time:    Instant::now(),
    }
  }
}

impl Meminfo {
  fn read_from(file: &mut File) -> Self {
    file.seek(SeekFrom::Start(0)).unwrap();
    let reader = std::io::BufReader::new(file);
    let values = reader
      .lines()
      .flat_map(|line| {
        let l = line.unwrap();
        let mut sections = l.split(':');
        let key = sections.next().unwrap();
        let value = sections.next().unwrap().trim();
        value.strip_suffix(" kB").map(|val| (key.to_string(), val.parse::<u64>().unwrap()))
      })
      .collect::<HashMap<String, u64>>();
    Meminfo {
      mem_total_kb: values["MemTotal"],
      mem_free_kb:  values["MemFree"],
      mem_avail_kb: values["MemAvailable"],
    }
  }
}
impl Stat {
  fn read_from(file: &mut File) -> Self {
    file.seek(SeekFrom::Start(0)).unwrap();
    let reader = std::io::BufReader::new(file);
    let mut lines = reader.lines();

    let first_line = lines.next().unwrap();
    let average = CpuStat::parse_from(&first_line.unwrap());

    let mut cpus = vec![];
    for line in lines {
      let l = line.unwrap();
      if l.starts_with("cpu") {
        cpus.push(CpuStat::parse_from(&l))
      } else {
        break;
      }
    }

    Stat { average, cpus }
  }
}

impl CpuStat {
  fn parse_from(s: &str) -> Self {
    let mut sections = s.split(' ');
    // This is the cpu/cpu0/cpu1 section
    let first = sections.next().unwrap();
    // The `cpu` line has a double space.
    if first == "cpu" {
      sections.next();
    }

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

  // Returns the total cpu time. This excludes the `idle` time.
  fn total(&self) -> u32 { self.user + self.nice + self.system + self.irq + self.softirq }
}

impl SystemInfo {
  pub fn new() -> SystemInfo {
    let mut files = Files::new();
    let curr_state = files.read_state();
    SystemInfo { last_update: Instant::now(), last_state: None, curr_state, files }
  }

  fn refresh(&mut self) {
    let now = Instant::now();
    if now.duration_since(self.last_update) > Duration::from_secs(1) {
      self.update();
    }
  }

  fn update(&mut self) {
    let new_state = self.files.read_state();
    self.last_state = Some(std::mem::replace(&mut self.curr_state, new_state));
    self.last_update = Instant::now();
  }

  pub fn state(&self) -> State {
    State {
      memory: MemoryState {
        total_mb: self.curr_state.meminfo.mem_total_kb / 1024,
        used_mb:  (self.curr_state.meminfo.mem_total_kb - self.curr_state.meminfo.mem_avail_kb)
          / 1024,
        avail_mb: self.curr_state.meminfo.mem_avail_kb / 1024,
      },
      // Our readings will be bad for the first lookup, which is fine.
      cpu:    if self.last_state.is_none() {
        CpuState::default()
      } else {
        let elapsed = self.curr_state.time.duration_since(self.last_state.as_ref().unwrap().time);
        CpuState {
          usage: {
            (self.curr_state.stat.average.total()
              - self.last_state.as_ref().unwrap().stat.average.total()) as f64
              / elapsed.as_secs_f64()
              / self.curr_state.stat.cpus.len() as f64
          },
        }
      },
    }
  }
}

#[derive(Clone)]
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
      let sys = sys.as_mut().unwrap();
      sys.refresh();
      let state = sys.state();

      ctx.draw_text(&format!("{:>2.00}", state.cpu.usage), self.primary);
      ctx.draw_text("%", self.secondary);
    });
  }
  fn box_clone(&self) -> Box<dyn ModuleImpl + Send + Sync> { Box::new(self.clone()) }
}

#[derive(Clone)]
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
      let sys = sys.as_mut().unwrap();
      sys.refresh();
      let state = sys.state();

      ctx.draw_text(&format!("{:>5.02}G", state.memory.used_mb as f64 / 1024_f64), self.primary);
      ctx.draw_text(" / ", self.secondary);
      ctx.draw_text(&format!("{:>5.02}G", state.memory.total_mb as f64 / 1024_f64), self.primary);
    });
  }
  fn box_clone(&self) -> Box<dyn ModuleImpl + Send + Sync> { Box::new(self.clone()) }
}
