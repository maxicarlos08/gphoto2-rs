use std::{borrow::Cow, ffi, os::raw::c_char};

pub fn chars_to_cow<'a>(chars: *const c_char) -> Cow<'a, str> {
  unsafe { String::from_utf8_lossy(ffi::CStr::from_ptr(chars).to_bytes()) }
}

pub fn camera_text_to_str<'a>(text: libgphoto2_sys::CameraText) -> Cow<'a, str> {
  unsafe { String::from_utf8_lossy(ffi::CStr::from_ptr(text.text.as_ptr()).to_bytes()) }
}

macro_rules! to_c_string {
  ($v:expr) => {
    ffi::CString::new($v)?.as_ptr().cast::<std::os::raw::c_char>()
  };
}

macro_rules! as_ref {
  ($from:ident -> $to:ty, $self:ident . $field:ident) => {
    as_ref!(@ $from -> $to, , $self, $self.$field);
  };

  ($from:ident -> $to:ty, * $self:ident . $field:ident) => {
    as_ref!(@ $from -> $to, unsafe, $self, *$self.$field);
  };

  (@ $from:ident -> $to:ty, $($unsafe:ident)?, $self:ident, $value:expr) => {
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

pub(crate) use {as_ref, to_c_string};
