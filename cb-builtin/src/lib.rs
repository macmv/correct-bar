use std::sync::{
  Arc, Weak,
  atomic::{AtomicBool, Ordering},
};

macro_rules! feature_mod {
  ($mod:ident, $feature:literal) => {
    #[cfg(feature = $feature)]
    pub mod $mod;

    #[cfg(feature = $feature)]
    pub use $mod::*;
  };
}

feature_mod!(clock, "clock");
feature_mod!(hwmon, "hwmon");
feature_mod!(hypr, "hypr");
feature_mod!(proc, "proc");
feature_mod!(pulse, "pulse");

struct UpdateGroup {
  dirty: Vec<Weak<AtomicBool>>,
}

struct Dirty {
  flag: Arc<AtomicBool>,
}

impl UpdateGroup {
  pub const fn new() -> Self { UpdateGroup { dirty: Vec::new() } }

  pub fn add(&mut self) -> Dirty {
    let dirty = Dirty { flag: Arc::new(AtomicBool::new(false)) };
    self.dirty.push(Arc::downgrade(&dirty.flag));
    dirty
  }

  pub fn mark_dirty(&mut self) {
    self.dirty.retain_mut(|w| {
      if let Some(a) = w.upgrade() {
        a.store(true, Ordering::SeqCst);
        true
      } else {
        false
      }
    });
  }
}

impl Dirty {
  pub fn clear(&self) { self.flag.store(false, Ordering::SeqCst); }

  pub fn get(&self) -> &Arc<AtomicBool> { &self.flag }
}
