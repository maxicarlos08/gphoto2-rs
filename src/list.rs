use crate::{helper::chars_to_cow, try_gp_internal, Result};
use std::{borrow::Cow, mem::MaybeUninit};

pub struct CameraList {
  inner: *mut libgphoto2_sys::CameraList,
}

impl Drop for CameraList {
  fn drop(&mut self) {
    unsafe {
      libgphoto2_sys::gp_list_unref(self.inner);
    }
  }
}

impl From<*mut libgphoto2_sys::CameraList> for CameraList {
  fn from(inner: *mut libgphoto2_sys::CameraList) -> Self {
    Self { inner }
  }
}

impl CameraList {
  pub fn to_vec(&self) -> Result<Vec<(Cow<str>, Cow<str>)>> {
    let length = unsafe { libgphoto2_sys::gp_list_count(self.inner) };

    let mut res = Vec::with_capacity(length as usize);

    for list_index in 0..length {
      let (mut name, mut value) =
        unsafe { (MaybeUninit::zeroed().assume_init(), MaybeUninit::zeroed().assume_init()) };

      try_gp_internal!(libgphoto2_sys::gp_list_get_name(self.inner, list_index, &mut name))?;
      try_gp_internal!(libgphoto2_sys::gp_list_get_value(self.inner, list_index, &mut value))?;

      res.push((chars_to_cow(name), chars_to_cow(value)));
    }

    Ok(res)
  }
}
