use correct_bar::bar::{Color, ModuleImpl, Updater};
use crossbeam_channel::Receiver;
use libpulse_sys as sys;
use parking_lot::Mutex;
use std::{
  ffi::{c_void, CStr, CString},
  fmt, ptr,
};

#[derive(Clone)]
pub struct Pulse {
  color: Color,
  recv:  Receiver<()>,
}

struct MainLoop {
  pa: *mut sys::pa_mainloop,
}

impl Drop for MainLoop {
  fn drop(&mut self) {
    unsafe {
      sys::pa_mainloop_free(self.pa);
    }
  }
}

impl MainLoop {
  pub fn new() -> Self {
    unsafe {
      let pa = sys::pa_mainloop_new();

      MainLoop { pa }
    }
  }

  pub fn run(&mut self) {
    unsafe {
      sys::pa_mainloop_run(self.pa, ptr::null_mut());
    }
  }
}

struct Context {
  pa: *mut sys::pa_context,
}

impl Drop for Context {
  fn drop(&mut self) {
    unsafe {
      sys::pa_context_unref(self.pa);
    }
  }
}

impl Context {
  pub fn new(mainloop: &mut MainLoop, props: &PropList) -> Self {
    unsafe {
      let pa = sys::pa_context_new_with_proplist(
        sys::pa_mainloop_get_api(mainloop.pa),
        b"correct-bar\0".as_ptr() as *const _,
        props.pa,
      );

      Context { pa }
    }
  }

  // Don't call this twice lol.
  pub fn set_callback(&mut self, custom: impl Fn() + Send + 'static) {
    static CALLBACK: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);
    extern "C" fn cb(_ctx: *mut sys::pa_context, _userdata: *mut c_void) {
      let cb = CALLBACK.lock();
      if let Some(cb) = &*cb {
        cb();
      }
    }

    CALLBACK.lock().replace(Box::new(custom));
    unsafe {
      sys::pa_context_set_state_callback(self.pa, Some(cb), ptr::null_mut());
    }
  }

  pub fn connect(&mut self) {
    unsafe {
      sys::pa_context_connect(
        self.pa,
        std::ptr::null(),
        sys::PA_CONTEXT_NOAUTOSPAWN,
        std::ptr::null(),
      );
    }
  }
}

struct PropList {
  pa: *mut sys::pa_proplist,
}

impl Drop for PropList {
  fn drop(&mut self) {
    unsafe {
      sys::pa_proplist_free(self.pa);
    }
  }
}

impl PropList {
  pub fn new() -> Self {
    unsafe {
      let pa = sys::pa_proplist_new();

      PropList { pa }
    }
  }

  pub fn set(&mut self, k: &str, v: &str) {
    unsafe {
      let k = CString::new(k).unwrap();
      let v = CString::new(v).unwrap();

      sys::pa_proplist_sets(self.pa, k.as_ptr(), v.as_ptr());
    }
  }
}

impl fmt::Debug for PropList {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    unsafe {
      let str = CStr::from_ptr(sys::pa_proplist_to_string(self.pa));

      write!(f, "{}", str.to_string_lossy())?;

      sys::pa_xfree(str.as_ptr() as *mut c_void);
    }
    Ok(())
  }
}

impl Pulse {
  pub fn new(color: Color) -> Self {
    let (tx, rx) = crossbeam_channel::bounded(16);

    println!("spawning thread");
    std::thread::spawn(move || {
      println!("creating main loop");
      let mut l = MainLoop::new();

      let mut props = PropList::new();

      println!("creating context");
      let mut ctx = Context::new(&mut l, &props);

      println!("setting context");
      ctx.set_callback(|| {
        println!("state change!");
      });

      println!("connecting");
      ctx.connect();
      println!("connected");

      tx.send(()).unwrap();

      l.run();
      panic!("loop exitted");
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
