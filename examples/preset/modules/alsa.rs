// SAFETY: This entire module uses the alsa_sys api directly! Edit with caution!

use alsa_sys as alsa;
use correct_bar::bar::{Color, ModuleImpl, Updater};
use crossbeam_channel::Receiver;
use parking_lot::Mutex;
use std::{
  ffi::{CStr, CString},
  fmt,
  sync::Arc,
};

#[derive(Clone)]
pub struct ALSA {
  control: Arc<Mutex<Control>>,
  elem:    MixerElemID,
  color:   Color,
  recv:    Receiver<()>,
}

macro_rules! check {
  ( $expr:expr) => {{
    let res = $expr;
    if res < 0 {
      Err(ALSAError { code: res })
    } else {
      Ok(res)
    }
  }};
}

struct ALSAError {
  code: i32,
}

type Result<T> = std::result::Result<T, ALSAError>;

impl fmt::Debug for ALSAError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = unsafe { alsa::snd_strerror(self.code) };
    if s.is_null() {
      write!(f, "ALSA error: unknown error code {}", self.code)
    } else {
      write!(f, "ALSA error: {}", unsafe { CStr::from_ptr(s) }.to_str().unwrap())
    }
  }
}

struct Control {
  ptr:  *mut alsa::snd_ctl_t,
  name: String,
}

unsafe impl Send for Control {}

impl Control {
  pub fn new_name(name: &str) -> Result<Self> {
    unsafe {
      let name = CString::new(name).unwrap();
      let id = check!(alsa::snd_card_get_index(name.as_ptr()))?;

      Self::new(id)
    }
  }
  pub fn new(id: i32) -> Result<Self> {
    unsafe {
      let name = format!("hw:{id}");
      let name_cstr = CString::new(name.clone()).unwrap();

      let mut ctl: *mut alsa::snd_ctl_t = std::ptr::null_mut();
      check!(alsa::snd_ctl_open(&mut ctl, name_cstr.as_ptr(), 1))?;

      Ok(Control { ptr: ctl, name })
    }
  }

  pub fn name(&self) -> &str { &self.name }
}

impl Drop for Control {
  fn drop(&mut self) {
    unsafe {
      alsa::snd_ctl_close(self.ptr);
    }
  }
}

struct Mixer<'control> {
  ptr:      *mut alsa::snd_mixer_t,
  _phantom: std::marker::PhantomData<&'control Control>,
}
struct MixerElem<'mixer, 'control> {
  ptr:      *mut alsa::snd_mixer_elem_t,
  _phantom: std::marker::PhantomData<&'mixer Mixer<'control>>,
}
#[derive(Clone)]
struct MixerElemID {
  ptr: *mut alsa::snd_mixer_selem_id_t,
}

unsafe impl Send for Mixer<'_> {}
unsafe impl Send for MixerElem<'_, '_> {}
unsafe impl Send for MixerElemID {}
unsafe impl Sync for MixerElemID {}

impl<'control> Mixer<'control> {
  pub fn new(control: &Control) -> Result<Self> {
    unsafe {
      let mut mixer: *mut alsa::snd_mixer_t = std::ptr::null_mut();
      check!(alsa::snd_mixer_open(&mut mixer, 1))?;

      let name = CString::new(control.name()).unwrap();
      check!(alsa::snd_mixer_attach(mixer, name.as_ptr()))?;
      check!(alsa::snd_mixer_selem_register(mixer, std::ptr::null_mut(), std::ptr::null_mut()))?;
      check!(alsa::snd_mixer_load(mixer))?;

      Ok(Mixer { ptr: mixer, _phantom: Default::default() })
    }
  }

  pub fn elems<'mixer>(&'mixer self) -> Result<Vec<MixerElem<'mixer, 'control>>> {
    unsafe {
      let mut elem = alsa::snd_mixer_first_elem(self.ptr);
      let mut elems = vec![];
      while !elem.is_null() {
        elems.push(MixerElem { ptr: elem, _phantom: Default::default() });

        elem = alsa::snd_mixer_elem_next(elem);
      }

      Ok(elems)
    }
  }

  pub fn get<'mixer>(&'mixer self, id: &MixerElemID) -> Option<MixerElem<'mixer, 'control>> {
    unsafe {
      let elem = alsa::snd_mixer_find_selem(self.ptr, id.ptr);
      if elem.is_null() {
        None
      } else {
        Some(MixerElem { ptr: elem, _phantom: Default::default() })
      }
    }
  }

  pub fn handle_callbacks(&self) {
    unsafe {
      alsa::snd_mixer_handle_events(self.ptr);
    }
  }
}

#[repr(i32)]
#[allow(unused)]
enum Channel {
  Unknown   = -1,
  FrontLeft = 0,
  FrontRight,
  RearLeft,
  RearRight,
  FrontCenter,
  Woofer,
  SideLeft,
  SideRight,
  RearCenter,
}

impl<'control> MixerElem<'_, 'control> {
  pub fn name(&self) -> String {
    unsafe {
      let ptr = alsa::snd_mixer_selem_get_name(self.ptr);
      if ptr.is_null() {
        panic!("got null ptr from mixer name");
      }
      CStr::from_ptr(ptr).to_str().unwrap().to_string()
    }
  }
  pub fn id(&self) -> Result<MixerElemID> {
    unsafe {
      let mut id: *mut alsa::snd_mixer_selem_id_t = std::ptr::null_mut();
      check!(alsa::snd_mixer_selem_id_malloc(&mut id))?;
      alsa::snd_mixer_selem_get_id(self.ptr, id);

      Ok(MixerElemID { ptr: id })
    }
  }

  #[allow(unused)]
  pub fn has_playback_channel(&self, channel: Channel) -> bool {
    unsafe { alsa::snd_mixer_selem_has_playback_channel(self.ptr, channel as i32) != 0 }
  }

  pub fn playback_volume(&self, channel: Channel) -> Option<i64> {
    unsafe {
      let mut volume = 0;
      let res = alsa::snd_mixer_selem_get_playback_volume(self.ptr, channel as i32, &mut volume);
      if res < 0 {
        None
      } else {
        Some(volume)
      }
    }
  }

  // Don't call this twice lol.
  pub fn set_callback(&self, custom: impl Fn() + Send + 'static) {
    static CALLBACK: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);
    unsafe extern "C" fn callback(_elem: *mut alsa::snd_mixer_elem_t, _mask: u32) -> i32 {
      let cb = CALLBACK.lock();
      if let Some(cb) = &*cb {
        cb();
      }
      0
    }
    unsafe {
      alsa::snd_mixer_elem_set_callback(self.ptr, Some(callback));
    }
    *CALLBACK.lock() = Some(Box::new(custom));
  }
}

impl fmt::Debug for MixerElem<'_, '_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("MixerElem").field("name", &self.name()).finish()
  }
}

impl Drop for Mixer<'_> {
  fn drop(&mut self) {
    unsafe {
      alsa::snd_mixer_close(self.ptr);
    }
  }
}

impl ALSA {
  pub fn new() -> Self { Self::new_inner().unwrap() }

  fn new_inner() -> Result<Self> {
    let control = Control::new_name("Generic")?;

    let (tx, rx) = crossbeam_channel::bounded(16);
    let elem = {
      let mixer = Mixer::new(&control)?;
      let mut elem = None;
      for e in mixer.elems()? {
        if e.name() == "Master" {
          e.set_callback(move || {
            tx.send(()).unwrap();
          });
          elem = Some(e.id()?);
          break;
        }
      }
      std::thread::spawn(move || loop {
        mixer.handle_callbacks();
      });
      elem
    };

    Ok(ALSA {
      control: Arc::new(Mutex::new(control)),
      elem:    elem.expect("could not find control"),
      color:   Color { r: 100, g: 255, b: 128 },
      recv:    rx,
    })
  }

  pub fn volume(&self) -> f64 {
    let control = self.control.lock();
    let mixer = Mixer::new(&control).unwrap();
    let elem = mixer.get(&self.elem).unwrap();
    elem.playback_volume(Channel::FrontLeft).unwrap() as f64 / 87.0
  }
}

impl ModuleImpl for ALSA {
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    // foo
    ctx.draw_text(&format!("{}", (self.volume() * 100.0).round()), self.color);
    ctx.draw_text("%", self.color);
  }
  fn updater(&self) -> Updater { Updater::Channel(self.recv.clone()) }
  fn box_clone(&self) -> Box<dyn ModuleImpl + Send + Sync> { Box::new(self.clone()) }
}
