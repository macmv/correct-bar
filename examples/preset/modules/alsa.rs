// SAFETY: This entire module uses the alsa_sys api directly! Edit with caution!

use alsa_sys as alsa;
use correct_bar::bar::{Color, ModuleImpl, Updater};
use parking_lot::Mutex;
use std::{
  ffi::{CStr, CString},
  fmt,
  sync::Arc,
  time::Duration,
};

#[derive(Clone)]
pub struct ALSA {
  mixer: Arc<Mutex<Mixer<'static>>>,
  elem:  MixerElemID<'static>,
  color: Color,
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
struct MixerElemID<'control> {
  ptr:      *mut alsa::snd_mixer_selem_id_t,
  _phantom: std::marker::PhantomData<&'control Control>,
}

unsafe impl Send for Mixer<'_> {}
unsafe impl Send for MixerElem<'_, '_> {}
unsafe impl Send for MixerElemID<'_> {}
unsafe impl Sync for MixerElemID<'_> {}

impl<'control> Mixer<'control> {
  pub fn new(control: &'control Control) -> Result<Self> {
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
      dbg!(&elem);
      let mut elems = vec![];
      while !elem.is_null() {
        elems.push(MixerElem { ptr: elem, _phantom: Default::default() });

        elem = alsa::snd_mixer_elem_next(elem);
      }

      Ok(elems)
    }
  }

  pub fn get<'mixer>(
    &'mixer self,
    id: &MixerElemID<'control>,
  ) -> Option<MixerElem<'mixer, 'control>> {
    unsafe {
      let elem = alsa::snd_mixer_find_selem(self.ptr, id.ptr);
      if elem.is_null() {
        None
      } else {
        Some(MixerElem { ptr: elem, _phantom: Default::default() })
      }
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
  pub fn id(&self) -> Result<MixerElemID<'control>> {
    unsafe {
      let mut id: *mut alsa::snd_mixer_selem_id_t = std::ptr::null_mut();
      check!(alsa::snd_mixer_selem_id_malloc(&mut id))?;
      alsa::snd_mixer_selem_get_id(self.ptr, id);

      Ok(MixerElemID { ptr: id, _phantom: Default::default() })
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
    let control = Box::leak(Box::new(Control::new_name("Generic")?));

    let mixer = Mixer::new(control)?;
    let mut elem = None;
    for e in mixer.elems()? {
      if e.name() == "Master" {
        elem = Some(e.id()?);
        break;
      }
    }

    Ok(ALSA {
      mixer: Arc::new(Mutex::new(mixer)),
      elem:  elem.expect("could not find control"),
      color: Color { r: 100, g: 255, b: 128 },
    })
  }

  pub fn volume(&self) -> f64 {
    let lock = self.mixer.lock();
    let elem = lock.get(&self.elem).unwrap();
    elem.playback_volume(Channel::FrontLeft).unwrap() as f64
  }
}

impl ModuleImpl for ALSA {
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    // foo
    ctx.draw_text(&format!("{}", self.volume() * 100.0), self.color);
    ctx.draw_text("%", self.color);
  }
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn box_clone(&self) -> Box<dyn ModuleImpl + Send + Sync> { Box::new(self.clone()) }
}
