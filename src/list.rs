//! List of cameras and ports

use crate::{helper::chars_to_cow, try_gp_internal, InnerPtr, Result};
use std::{borrow::Cow, ffi, marker::PhantomData};

/// List of string tuples
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

impl<'a> InnerPtr<'a, libgphoto2_sys::CameraList> for CameraList<'a> {
  unsafe fn inner_mut_ptr(&'a self) -> &'a *mut libgphoto2_sys::CameraList {
    &self.inner
  }
}

impl CameraList<'_> {
  pub(crate) fn new() -> Result<Self> {
    try_gp_internal!(gp_list_new(&out list));

    Ok(Self { inner: list, phantom: PhantomData })
  }

  /// Converts the internal gphoto list to a rust vec
  pub fn to_vec(&self) -> Result<Vec<(Cow<str>, Cow<str>)>> {
    let length = unsafe { libgphoto2_sys::gp_list_count(self.inner) };

    let mut res = Vec::with_capacity(length as usize);

    for list_index in 0..length {
      try_gp_internal!(gp_list_get_name(self.inner, list_index, &out name));
      try_gp_internal!(gp_list_get_value(self.inner, list_index, &out value));

      res.push((chars_to_cow(name), chars_to_cow(value)));
    }

    Ok(res)
  }
}
