//! List of cameras and ports

use crate::{
  helper::{chars_to_cow, uninit},
  try_gp_internal, Result,
};
use std::{borrow::Cow, ffi, marker::PhantomData};

/// List of tuples: (camere model, port)
pub struct CameraList<'a> {
  pub(crate) inner: *mut libgphoto2_sys::CameraList,
  phantom: PhantomData<&'a ffi::c_void>,
}

impl Drop for CameraList<'_> {
  fn drop(&mut self) {
    unsafe {
      libgphoto2_sys::gp_list_unref(self.inner);
    }
  }
}

impl CameraList<'_> {
  pub(crate) fn new() -> Result<Self> {
    let mut list = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_list_new(&mut list))?;

    Ok(Self { inner: list, phantom: PhantomData })
  }

  /// Converts the internal gphoto list to a rust vec
  pub fn to_vec(&self) -> Result<Vec<(Cow<str>, Cow<str>)>> {
    let length = unsafe { libgphoto2_sys::gp_list_count(self.inner) };

    let mut res = Vec::with_capacity(length as usize);

    for list_index in 0..length {
      let (mut name, mut value) = unsafe { (uninit(), uninit()) };

      try_gp_internal!(libgphoto2_sys::gp_list_get_name(self.inner, list_index, &mut name))?;
      try_gp_internal!(libgphoto2_sys::gp_list_get_value(self.inner, list_index, &mut value))?;

      res.push((chars_to_cow(name), chars_to_cow(value)));
    }

    Ok(res)
  }
}
