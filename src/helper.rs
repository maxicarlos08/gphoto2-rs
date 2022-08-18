use std::{borrow::Cow, ffi, mem::MaybeUninit};

pub fn chars_to_cow<'a>(chars: *const i8) -> Cow<'a, str> {
  unsafe { String::from_utf8_lossy(ffi::CStr::from_ptr(chars).to_bytes()) }
}

pub fn camera_text_to_str<'a>(text: libgphoto2_sys::CameraText) -> Cow<'a, str> {
  unsafe { String::from_utf8_lossy(ffi::CStr::from_ptr(text.text.as_ptr()).to_bytes()) }
}

macro_rules! to_c_string {
  ($v: expr, $name:ident) => {
    let $name = ffi::CString::new($v)?;
  };
  ($name:ident) => {
    to_c_string!($name, $name);
  };
}

#[inline]
pub unsafe fn uninit<T>() -> T {
  MaybeUninit::zeroed().assume_init()
}

pub(crate) use to_c_string;
