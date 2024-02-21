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

unsafe impl Send for Context {}

impl Drop for Context {
  fn drop(&mut self) {
    unsafe {
      sys::pa_context_unref(self.pa);
    }
  }
}

impl Clone for Context {
  fn clone(&self) -> Self {
    unsafe {
      sys::pa_context_ref(self.pa);
    }
    Context { pa: self.pa }
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

  pub fn get_state(&self) -> ContextState {
    unsafe { ContextState::from_sys(sys::pa_context_get_state(self.pa)) }
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ContextState {
  Unconnected,
  Connecting,
  Authorizing,
  SettingName,
  Ready,
  Failed,
  Terminated,
  Unknown,
}

impl ContextState {
  pub fn from_sys(s: sys::pa_context_state_t) -> Self {
    match s {
      sys::PA_CONTEXT_UNCONNECTED => ContextState::Unconnected,
      sys::PA_CONTEXT_CONNECTING => ContextState::Connecting,
      sys::PA_CONTEXT_AUTHORIZING => ContextState::Authorizing,
      sys::PA_CONTEXT_SETTING_NAME => ContextState::SettingName,
      sys::PA_CONTEXT_READY => ContextState::Ready,
      sys::PA_CONTEXT_FAILED => ContextState::Failed,
      sys::PA_CONTEXT_TERMINATED => ContextState::Terminated,

      #[allow(unreachable_patterns)]
      _ => ContextState::Unknown,
    }
  }
}

impl Pulse {
  pub fn new(color: Color) -> Self {
    let (tx, rx) = crossbeam_channel::bounded(16);

    std::thread::spawn(move || {
      let mut l = MainLoop::new();

      let props = PropList::new();

      let mut ctx = Context::new(&mut l, &props);

      ctx.set_callback({
        let ctx = ctx.clone();
        move || {
          if ctx.get_state() == ContextState::Ready {
            println!("ready");
          }
        }
      });

      ctx.connect();

      tx.send(()).unwrap();

      l.run();
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
