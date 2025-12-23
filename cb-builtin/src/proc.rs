//! Reads from /proc to get the current memory and cpu usage.

use std::{
  cell::RefCell,
  collections::HashMap,
  fs::File,
  io::{BufRead, Seek, SeekFrom},
  time::{Duration, Instant},
};

use cb_bar::{Animation, Module, TextLayout, Updater};
use cb_core::{Color, Render, Text};
use kurbo::{Line, Point};
use peniko::Gradient;

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
  pub average: f64,
  pub cpus:    Vec<f64>,
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
      cpu:    if let Some(last) = &self.last_state {
        let elapsed = self.curr_state.time.duration_since(last.time);
        CpuState {
          average: {
            (self.curr_state.stat.average.total() - last.stat.average.total()) as f64
              / elapsed.as_secs_f64()
              / self.curr_state.stat.cpus.len() as f64
          },
          cpus:    self
            .curr_state
            .stat
            .cpus
            .iter()
            .zip(last.stat.cpus.iter())
            .map(|(curr, last)| (curr.total() - last.total()) as f64 / elapsed.as_secs_f64())
            .collect(),
        }
      } else {
        CpuState { average: 0.0, cpus: vec![0.0; self.curr_state.stat.cpus.len()] }
      },
    }
  }
}

#[derive(Clone)]
pub struct Cpu {
  pub primary:   Color,
  pub secondary: Color,
}
struct CpuModule {
  spec: Cpu,
  text: Option<TextLayout>,

  usage: Vec<f64>,
}

impl From<Cpu> for Box<dyn Module> {
  fn from(spec: Cpu) -> Self { Box::new(CpuModule { spec, text: None, usage: vec![] }) }
}

const MAX_PER_COL: usize = 8;

impl CpuModule {
  fn cols(&self) -> usize { (self.usage.len() + MAX_PER_COL - 1) / MAX_PER_COL }
  fn row_cols(&self) -> (usize, usize) {
    let cols = self.cols();
    let rows = (self.usage.len() + cols - 1) / cols;
    (rows, cols)
  }
}

impl Module for CpuModule {
  fn updater(&self) -> Updater<'_> { Updater::Every(Duration::from_secs(1)) }
  fn layout(&mut self, layout: &mut cb_bar::Layout) {
    SYS.with(|s| {
      let mut sys = s.borrow_mut();
      if sys.is_none() {
        *sys = Some(SystemInfo::new());
      }
      let sys = sys.as_mut().unwrap();
      sys.refresh();
      let state = sys.state();

      let mut text = Text::new();
      text.push(format_args!("{:>2.00}", state.cpu.average), self.spec.primary);
      text.push("%", self.spec.secondary);

      self.usage = state.cpu.cpus.clone();

      layout.pad(10.0 + 6.0 * self.cols() as f64);
      self.text = Some(layout.layout_text(text, self.spec.primary));
      layout.pad(5.0);
    });
  }
  fn render(&self, ctx: &mut Render) {
    if let Some(text) = &self.text {
      ctx.draw(text);

      ctx.stroke(
        &Line::new(
          Point::new(text.bounds().min_x(), text.bounds().max_y().round() + 4.0),
          Point::new(text.bounds().max_x(), text.bounds().max_y().round() + 4.0),
        ),
        self.spec.primary,
      );

      let min_y = text.bounds().y0 - 5.0;
      let max_y = text.bounds().y1 + 5.0;

      if !self.usage.is_empty() {
        let (rows, cols) = self.row_cols();
        let delta = (max_y - min_y) / (rows - 1) as f64;

        let mut cpu = 0;
        'outer: for i in 0..cols {
          for j in 0..rows {
            let y = min_y + j as f64 * delta;

            if cpu >= self.usage.len() {
              break 'outer;
            }

            ctx.stroke(
              &Line::new((5.0 + i as f64 * 6.0, y), (5.0 + i as f64 * 6.0 + 3.0, y)),
              self.spec.secondary.lerp(
                self.spec.primary,
                (self.usage[cpu] / 100.0) as f32,
                peniko::color::HueDirection::Shorter,
              ),
            );
            cpu += 1;
          }
        }
      }
    }
  }
}

#[derive(Clone)]
pub struct Mem {
  pub primary:   Color,
  pub secondary: Color,
}
struct MemModule {
  spec:  Mem,
  hover: Animation,
  text:  Option<TextLayout>,
}

impl From<Mem> for Box<dyn Module> {
  fn from(spec: Mem) -> Self {
    Box::new(MemModule { spec, hover: Animation::ease_out(0.2), text: None })
  }
}

impl Module for MemModule {
  fn updater(&self) -> Updater<'_> {
    if self.hover.is_running() {
      Updater::Animation
    } else {
      Updater::Every(Duration::from_secs(1))
    }
  }

  fn on_hover(&mut self, hover: bool) { self.hover.run(hover); }

  fn layout(&mut self, layout: &mut cb_bar::Layout) {
    layout.pad(5.0);

    SYS.with(|s| {
      let mut sys = s.borrow_mut();
      if sys.is_none() {
        *sys = Some(SystemInfo::new());
      }
      let sys = sys.as_mut().unwrap();
      sys.refresh();
      let state = sys.state();

      let mut text = Text::new();

      text
        .push(format_args!("{:>5.02}", state.memory.used_mb as f64 / 1024_f64), self.spec.primary);
      text.push("G / ", self.spec.secondary);
      text
        .push(format_args!("{:>5.02}", state.memory.total_mb as f64 / 1024_f64), self.spec.primary);
      text.push("G", self.spec.secondary);

      self.text = Some(layout.layout_text(text, self.spec.primary));
    });

    layout.pad(5.0);
  }
  fn render(&self, ctx: &mut Render) {
    self.hover.advance(ctx.frame_time());

    if let Some(text) = &self.text {
      ctx.draw(text);

      let min_x = text.bounds().min_x();
      let max_x = text.bounds().max_x();

      let start = Point::new(
        self.hover.interpolate((min_x + max_x) / 2.0, min_x),
        text.bounds().max_y().round() + 4.0,
      );
      let end = Point::new(
        self.hover.interpolate((min_x + max_x) / 2.0, max_x),
        text.bounds().max_y().round() + 4.0,
      );

      ctx.stroke(
        &Line::new(start, end),
        Gradient::new_linear(start, end).with_stops([
          self.spec.primary,
          self.spec.primary.map_lightness(|l| l + 0.1),
          self.spec.primary,
        ]),
      );
    }
  }
}
