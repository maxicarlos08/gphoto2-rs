use std::{
  borrow::Cow,
  ffi,
  fs::File,
  mem::MaybeUninit,
  os::raw::{c_char, c_int},
  sync::Once,
};

static HOOK_LOG_FUNCTION: Once = Once::new();

pub fn char_slice_to_cow(chars: &[c_char]) -> Cow<'_, str> {
  unsafe { String::from_utf8_lossy(ffi::CStr::from_ptr(chars.as_ptr()).to_bytes()) }
}

pub fn chars_to_string(chars: *const c_char) -> String {
  unsafe { String::from_utf8_lossy(ffi::CStr::from_ptr(chars).to_bytes()) }.into_owned()
}

pub trait IntoUnixFd {
  fn into_unix_fd(self) -> c_int;
}

#[cfg(unix)]
impl IntoUnixFd for File {
  fn into_unix_fd(self) -> c_int {
    use std::os::unix::prelude::IntoRawFd;

    self.into_raw_fd()
  }
}

#[cfg(windows)]
impl IntoUnixFd for File {
  fn into_unix_fd(self) -> c_int {
    use std::os::windows::io::IntoRawHandle;

    let handle = self.into_raw_handle();

    unsafe { libc::open_osfhandle(handle as _, 0) }
  }
}

#[cfg(feature = "extended_logs")]
pub fn hook_gp_log() {
  use libgphoto2_sys::GPLogLevel;
  use log::LevelFilter;

  unsafe extern "C" fn log_function(
    level: libgphoto2_sys::GPLogLevel,
    domain: *const std::os::raw::c_char,
    message: *const std::os::raw::c_char,
    _data: *mut ffi::c_void,
  ) {
    let log_level = match level {
      GPLogLevel::GP_LOG_ERROR => log::Level::Error,
      GPLogLevel::GP_LOG_DEBUG => log::Level::Debug,
      GPLogLevel::GP_LOG_VERBOSE => log::Level::Info,
      GPLogLevel::GP_LOG_DATA => log::Level::Trace,
    };

    let target = format!("gphoto2::{}", chars_to_string(domain));

    log::log!(target: &target, log_level, "{}", chars_to_string(message));
  }

  let max_log_level = match log::STATIC_MAX_LEVEL {
    LevelFilter::Debug | LevelFilter::Warn => GPLogLevel::GP_LOG_DEBUG,
    LevelFilter::Error => GPLogLevel::GP_LOG_ERROR,
    LevelFilter::Info => GPLogLevel::GP_LOG_VERBOSE,
    LevelFilter::Trace => GPLogLevel::GP_LOG_DATA,
    LevelFilter::Off => return,
  };

  HOOK_LOG_FUNCTION.call_once(|| unsafe {
    libgphoto2_sys::gp_log_add_func(max_log_level, Some(log_function), std::ptr::null_mut());
  });
}

#[cfg(not(feature = "extended_logs"))]
pub fn hook_gp_context_log_func(context: *mut libgphoto2_sys::GPContext) {
  use log::Level;

  unsafe extern "C" fn log_func(
    _context: *mut libgphoto2_sys::GPContext,
    message: *const c_char,
    log_level: *mut ffi::c_void,
  ) {
    let log_level: Level = std::mem::transmute(log_level);

    log::log!(target: "gphoto2", log_level, "{}", chars_to_string(message));
  }

  HOOK_LOG_FUNCTION.call_once(|| unsafe {
    if log::log_enabled!(log::Level::Error) {
      let log_level_as_ptr = std::mem::transmute(log::Level::Error);

      libgphoto2_sys::gp_context_set_error_func(context, Some(log_func), log_level_as_ptr);

      // `gp_context_message` seems to be used also for error messages.
      libgphoto2_sys::gp_context_set_message_func(context, Some(log_func), log_level_as_ptr);
    }

    if log::log_enabled!(log::Level::Info) {
      libgphoto2_sys::gp_context_set_status_func(
        context,
        Some(log_func),
        std::mem::transmute(log::Level::Info),
      );
    }
  });
}

pub struct UninitBox<T> {
  inner: Box<MaybeUninit<T>>,
}

impl<T> UninitBox<T> {
  pub fn uninit() -> Self {
    Self { inner: Box::new(MaybeUninit::uninit()) }
  }

  pub fn as_mut_ptr(&mut self) -> *mut T {
    self.inner.as_mut_ptr().cast()
  }

  pub unsafe fn assume_init(self) -> Box<T> {
    Box::from_raw(Box::into_raw(self.inner).cast())
  }
}

macro_rules! to_c_string {
  ($v:expr) => {
    ffi::CString::new($v)?.as_ptr().cast::<std::os::raw::c_char>()
  };
}

macro_rules! as_ref {
  ($from:ident $(<$lt:tt>)? -> $to:ty, $self:ident $($rest:tt)*) => {
    as_ref!(@ $from $(<$lt>)?, $to, , $self, $self $($rest)*);
  };

  ($from:ident $(<$lt:tt>)? -> $to:ty, * $self:ident $($rest:tt)*) => {
    as_ref!(@ $from $(<$lt>)?, $to, unsafe, $self, *$self $($rest)*);
  };

  ($from:ident $(<$lt:tt>)? -> $to:ty, ** $self:ident $($rest:tt)*) => {
    as_ref!(@ $from $(<$lt>)?, $to, unsafe, $self, **$self $($rest)*);
  };

  (@ $from:ty, $to:ty, $($unsafe:ident)?, $self:ident, $value:expr) => {
    impl AsRef<$to> for $from {
      fn as_ref(&$self) -> &$to {
        $($unsafe)? { & $value }
      }
    }

    impl AsMut<$to> for $from {
      fn as_mut(&mut $self) -> &mut $to {
        $($unsafe)? { &mut $value }
      }
    }
  };
}

macro_rules! bitflags {
  ($(# $attr:tt)* $name:ident = $target:ident { $($(# $field_attr:tt)* $field:ident: $value:ident,)* }) => {
    $(# $attr)*
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub struct $name(libgphoto2_sys::$target);

    impl From<libgphoto2_sys::$target> for $name {
      fn from(flags: libgphoto2_sys::$target) -> Self {
        Self(flags)
      }
    }

    impl $name {
      $(
        $(# $field_attr)*
        #[inline]
        pub fn $field(&self) -> bool {
          (self.0 & libgphoto2_sys::$target::$value).0 != 0
        }
      )*
    }

    impl std::fmt::Debug for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!($name))
          $(
            .field(stringify!($field), &self.$field())
          )*
          .finish()
      }
    }
  };
}

pub(crate) use {as_ref, bitflags, to_c_string};
